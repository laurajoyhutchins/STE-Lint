use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use ste_glossary::{Glossary, TechnicalTerm};

const BUILTIN_PROFILE_DATA: &[(&str, &str)] = &[
    (
        "software-core",
        include_str!("../../../profiles/software-core.json"),
    ),
    ("git", include_str!("../../../profiles/git.json")),
    ("github", include_str!("../../../profiles/github.json")),
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileMetadata {
    pub id: String,
    pub version: u32,
    pub description: String,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminologyProfile {
    pub profile: ProfileMetadata,
    pub terms: Vec<TechnicalTerm>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectConfig {
    #[serde(default)]
    profiles: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EffectiveGlossaryReport {
    pub profiles: Vec<ProfileMetadata>,
    pub project_glossary: Option<String>,
    pub terms: Vec<TechnicalTerm>,
}

pub struct EffectiveGlossary {
    pub glossary: Option<Glossary>,
    pub report: EffectiveGlossaryReport,
}

pub fn builtin_profiles() -> Result<Vec<TerminologyProfile>, String> {
    BUILTIN_PROFILE_DATA
        .iter()
        .map(|(id, json)| parse_builtin_profile(id, json))
        .collect()
}

pub fn builtin_profile(id: &str) -> Result<TerminologyProfile, String> {
    let Some((expected_id, json)) = BUILTIN_PROFILE_DATA
        .iter()
        .find(|(candidate, _)| *candidate == id)
    else {
        return Err(format!("unknown terminology profile '{id}'"));
    };
    parse_builtin_profile(expected_id, json)
}

pub fn resolve_effective_glossary(path: &Path) -> Result<EffectiveGlossary, String> {
    let config_path = project_file(path, ".ste/config.json");
    let selected_ids = match config_path {
        Some(ref candidate) => parse_project_config(candidate)?.profiles,
        None => Vec::new(),
    };

    let mut seen_profiles = HashSet::new();
    let mut selected_profiles = Vec::new();
    let mut glossaries = Vec::new();
    for id in selected_ids {
        if !seen_profiles.insert(id.clone()) {
            return Err(format!("duplicate terminology profile '{id}'"));
        }
        let profile = builtin_profile(&id)?;
        let glossary = Glossary {
            terms: profile.terms.clone(),
        };
        validate_glossary(&glossary, &format!("built-in terminology profile '{id}'"))?;
        selected_profiles.push(profile.profile);
        glossaries.push(glossary);
    }

    let project_glossary_path = project_file(path, ".ste/terms.json");
    if let Some(candidate) = project_glossary_path.as_ref() {
        let glossary = parse_glossary(candidate)?;
        validate_glossary(
            &glossary,
            &format!("project glossary {}", candidate.display()),
        )?;
        glossaries.push(glossary);
    }

    let glossary = if glossaries.is_empty() {
        None
    } else {
        Some(Glossary::compose(&glossaries).map_err(|diagnostics| {
            format!(
                "effective technical glossary failed validation: {}",
                diagnostic_codes(&diagnostics)
            )
        })?)
    };

    Ok(EffectiveGlossary {
        report: EffectiveGlossaryReport {
            profiles: selected_profiles,
            project_glossary: project_glossary_path.map(|path| path.display().to_string()),
            terms: glossary
                .as_ref()
                .map(|glossary| glossary.terms.clone())
                .unwrap_or_default(),
        },
        glossary,
    })
}

fn parse_builtin_profile(expected_id: &str, json: &str) -> Result<TerminologyProfile, String> {
    let profile: TerminologyProfile = serde_json::from_str(json).map_err(|error| {
        format!("invalid built-in terminology profile '{expected_id}': {error}")
    })?;
    if profile.profile.id != expected_id {
        return Err(format!(
            "built-in terminology profile identity mismatch: expected '{expected_id}', found '{}'",
            profile.profile.id
        ));
    }
    Ok(profile)
}

fn parse_project_config(path: &Path) -> Result<ProjectConfig, String> {
    let text = fs::read_to_string(path).map_err(|error| {
        format!(
            "could not read STE project config {}: {error}",
            path.display()
        )
    })?;
    serde_json::from_str(&text)
        .map_err(|error| format!("invalid STE project config {}: {error}", path.display()))
}

fn parse_glossary(path: &Path) -> Result<Glossary, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("could not read glossary {}: {error}", path.display()))?;
    Glossary::from_json(&text)
        .map_err(|error| format!("invalid glossary {}: {error}", path.display()))
}

fn validate_glossary(glossary: &Glossary, label: &str) -> Result<(), String> {
    let diagnostics = glossary.validate();
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{label} failed validation: {}",
            diagnostic_codes(&diagnostics)
        ))
    }
}

fn diagnostic_codes(diagnostics: &[ste_core::Diagnostic]) -> String {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn project_file(path: &Path, relative: &str) -> Option<PathBuf> {
    let start = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or_else(|| Path::new("."))
    };

    start
        .ancestors()
        .map(|ancestor| ancestor.join(relative))
        .find(|candidate| candidate.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_profiles_have_stable_distinct_identities() {
        let profiles = builtin_profiles().unwrap();
        assert_eq!(
            profiles
                .iter()
                .map(|profile| profile.profile.id.as_str())
                .collect::<Vec<_>>(),
            vec!["software-core", "git", "github"]
        );

        let glossaries = profiles
            .iter()
            .map(|profile| Glossary {
                terms: profile.terms.clone(),
            })
            .collect::<Vec<_>>();
        assert!(Glossary::compose(&glossaries).is_ok());
    }
}