use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Serialize};
use ste_core::{Diagnostic, Severity, Span};

pub const TERMINOLOGY_SCHEMA_V2: &str = "ste-terminology/v2";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TermRole {
    Noun,
    Verb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TermStatus {
    Approved,
    Deprecated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AliasKind {
    Abbreviation,
    Acronym,
    ShortForm,
    Synonym,
    Legacy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceSupport {
    Admission,
    Definition,
    Role,
    Forms,
    Alias,
    Status,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminologySource {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewed_on: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TermSourceRef {
    pub source: String,
    pub supports: Vec<SourceSupport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminologyForm {
    pub text: String,
    pub roles: Vec<TermRole>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminologyAlias {
    pub text: String,
    pub kind: AliasKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminologyTerm {
    pub id: String,
    pub canonical: String,
    pub roles: Vec<TermRole>,
    pub definition: String,
    #[serde(default)]
    pub forms: Vec<TerminologyForm>,
    #[serde(default)]
    pub aliases: Vec<TerminologyAlias>,
    #[serde(default)]
    pub sources: Vec<TermSourceRef>,
    pub status: TermStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replacement: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminologyDocument {
    pub schema: String,
    pub domain: String,
    #[serde(default)]
    pub sources: BTreeMap<String, TerminologySource>,
    pub terms: Vec<TerminologyTerm>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileMetadata {
    pub id: String,
    pub version: u32,
    pub domain: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminologyProfile {
    pub schema: String,
    pub profile: ProfileMetadata,
    #[serde(default)]
    pub sources: BTreeMap<String, TerminologySource>,
    pub terms: Vec<TerminologyTerm>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GlossaryIdentityKind {
    Canonical,
    Alias,
    Form,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TechnicalTerm {
    pub id: String,
    pub canonical: String,
    pub roles: Vec<TermRole>,
    pub definition: String,
    pub domain: String,
    pub forms: Vec<TerminologyForm>,
    pub aliases: Vec<TerminologyAlias>,
    pub sources: Vec<TermSourceRef>,
    pub source_catalog: BTreeMap<String, TerminologySource>,
    pub status: TermStatus,
    pub replacement: Option<String>,
    pub examples: Vec<String>,
}

impl TechnicalTerm {
    pub fn has_role(&self, role: TermRole) -> bool {
        self.roles.contains(&role)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IdentityRecord {
    term_index: usize,
    kind: GlossaryIdentityKind,
    roles: Vec<TermRole>,
    alias_kind: Option<AliasKind>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlossaryIdentityMatch<'a> {
    pub term: &'a TechnicalTerm,
    pub identity_kind: GlossaryIdentityKind,
    pub roles: &'a [TermRole],
    pub alias_kind: Option<AliasKind>,
}

#[derive(Debug, Clone)]
pub struct Glossary {
    terms: Vec<TechnicalTerm>,
    identities: HashMap<String, IdentityRecord>,
    max_identity_words: usize,
    diagnostics: Vec<Diagnostic>,
}

impl Glossary {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let value: serde_json::Value = serde_json::from_str(json)?;
        if value.get("schema").and_then(|value| value.as_str()) == Some(TERMINOLOGY_SCHEMA_V2) {
            let document: TerminologyDocument = serde_json::from_value(value)?;
            Ok(Self::from_document(document))
        } else {
            let legacy: LegacyGlossary = serde_json::from_value(value)?;
            Ok(Self::from_legacy(legacy))
        }
    }

    pub fn from_document(document: TerminologyDocument) -> Self {
        Self::compile(
            Some(document.schema.as_str()),
            &document.domain,
            document.sources,
            document.terms,
        )
    }

    pub fn from_profile(profile: &TerminologyProfile) -> Self {
        Self::compile(
            Some(profile.schema.as_str()),
            &profile.profile.domain,
            profile.sources.clone(),
            profile.terms.clone(),
        )
    }

    fn compile(
        schema: Option<&str>,
        domain: &str,
        source_catalog: BTreeMap<String, TerminologySource>,
        terms: Vec<TerminologyTerm>,
    ) -> Self {
        let mut diagnostics = Vec::new();
        if let Some(schema) = schema
            && schema != TERMINOLOGY_SCHEMA_V2
        {
            diagnostics.push(simple_diagnostic(
                "TERM-SCHEMA-001",
                "Technical glossary uses an unsupported terminology schema.",
                serde_json::json!({"schema": schema, "supported": TERMINOLOGY_SCHEMA_V2}),
            ));
        }

        let compiled_terms = terms
            .into_iter()
            .map(|term| TechnicalTerm {
                id: term.id,
                canonical: term.canonical,
                roles: term.roles,
                definition: term.definition,
                domain: domain.to_owned(),
                forms: term.forms,
                aliases: term.aliases,
                sources: term.sources,
                source_catalog: source_catalog.clone(),
                status: term.status,
                replacement: term.replacement,
                examples: term.examples,
            })
            .collect::<Vec<_>>();
        diagnostics.extend(validate_terms(&compiled_terms));
        let (identities, max_identity_words, identity_diagnostics) = build_identity_index(&compiled_terms);
        diagnostics.extend(identity_diagnostics);

        Self {
            terms: compiled_terms,
            identities,
            max_identity_words,
            diagnostics,
        }
    }

    fn from_legacy(legacy: LegacyGlossary) -> Self {
        let mut sources = BTreeMap::new();
        let mut terms = Vec::new();
        for (index, term) in legacy.terms.into_iter().enumerate() {
            let roles = match term.kind {
                LegacyTechnicalTermKind::TechnicalNoun => vec![TermRole::Noun],
                LegacyTechnicalTermKind::TechnicalVerb => vec![TermRole::Verb],
                LegacyTechnicalTermKind::TechnicalNounAndVerb => vec![TermRole::Noun, TermRole::Verb],
            };
            let source_refs = term
                .provenance
                .iter()
                .enumerate()
                .map(|(source_index, title)| {
                    let id = format!("legacy-{index}-{source_index}");
                    sources.insert(
                        id.clone(),
                        TerminologySource {
                            title: title.clone(),
                            url: None,
                            reviewed_on: None,
                        },
                    );
                    TermSourceRef {
                        source: id,
                        supports: vec![
                            SourceSupport::Admission,
                            SourceSupport::Definition,
                            SourceSupport::Role,
                            SourceSupport::Forms,
                            SourceSupport::Alias,
                            SourceSupport::Status,
                        ],
                    }
                })
                .collect();
            terms.push(TerminologyTerm {
                id: stable_id(&term.term),
                canonical: term.term,
                roles: roles.clone(),
                definition: term.definition,
                forms: term
                    .forms
                    .into_iter()
                    .map(|text| TerminologyForm {
                        text,
                        roles: roles.clone(),
                    })
                    .collect(),
                aliases: term
                    .aliases
                    .into_iter()
                    .map(|text| TerminologyAlias {
                        text,
                        kind: AliasKind::Synonym,
                    })
                    .collect(),
                sources: source_refs,
                status: term.status,
                replacement: None,
                examples: term.examples,
            });
        }
        Self::compile(None, "legacy", sources, terms)
    }

    pub fn compose(glossaries: &[Glossary]) -> Result<Self, Vec<Diagnostic>> {
        let terms = glossaries
            .iter()
            .flat_map(|glossary| glossary.terms.iter().cloned())
            .collect::<Vec<_>>();
        let mut diagnostics = glossaries
            .iter()
            .flat_map(|glossary| glossary.diagnostics.iter().cloned())
            .collect::<Vec<_>>();
        diagnostics.extend(validate_terms(&terms));
        let (identities, max_identity_words, identity_diagnostics) = build_identity_index(&terms);
        diagnostics.extend(identity_diagnostics);
        deduplicate_diagnostics(&mut diagnostics);
        if diagnostics.is_empty() {
            Ok(Self {
                terms,
                identities,
                max_identity_words,
                diagnostics: Vec::new(),
            })
        } else {
            Err(diagnostics)
        }
    }

    pub fn terms(&self) -> &[TechnicalTerm] {
        &self.terms
    }

    pub fn max_identity_words(&self) -> usize {
        self.max_identity_words
    }

    pub fn lookup_identity(&self, value: &str) -> Option<GlossaryIdentityMatch<'_>> {
        let record = self.identities.get(&normalize_identity(value))?;
        Some(GlossaryIdentityMatch {
            term: &self.terms[record.term_index],
            identity_kind: record.kind,
            roles: &record.roles,
            alias_kind: record.alias_kind,
        })
    }

    pub fn lookup_term(&self, value: &str) -> Option<&TechnicalTerm> {
        self.lookup_identity(value).map(|matched| matched.term)
    }

    pub fn contains_term(&self, value: &str) -> bool {
        self.lookup_identity(value).is_some()
    }

    pub fn validate(&self) -> Vec<Diagnostic> {
        self.diagnostics.clone()
    }
}

fn build_identity_index(
    terms: &[TechnicalTerm],
) -> (HashMap<String, IdentityRecord>, usize, Vec<Diagnostic>) {
    let mut identities = HashMap::new();
    let mut diagnostics = Vec::new();
    let mut max_words = 1;

    for (term_index, term) in terms.iter().enumerate() {
        let mut add_identity = |text: &str,
                                kind: GlossaryIdentityKind,
                                roles: Vec<TermRole>,
                                alias_kind: Option<AliasKind>| {
            let normalized = normalize_identity(text);
            max_words = max_words.max(normalized.split_whitespace().count());
            if let Some(first) = identities.get(&normalized) {
                let first_term: &TechnicalTerm = &terms[first.term_index];
                diagnostics.push(identity_conflict(
                    &first_term.canonical,
                    first.kind,
                    &term.canonical,
                    kind,
                    &normalized,
                ));
                return;
            }
            identities.insert(
                normalized,
                IdentityRecord {
                    term_index,
                    kind,
                    roles,
                    alias_kind,
                },
            );
        };

        add_identity(
            &term.canonical,
            GlossaryIdentityKind::Canonical,
            term.roles.clone(),
            None,
        );
        for alias in &term.aliases {
            add_identity(
                &alias.text,
                GlossaryIdentityKind::Alias,
                term.roles.clone(),
                Some(alias.kind),
            );
        }
        for form in &term.forms {
            add_identity(
                &form.text,
                GlossaryIdentityKind::Form,
                form.roles.clone(),
                None,
            );
        }
    }

    (identities, max_words, diagnostics)
}

fn validate_terms(terms: &[TechnicalTerm]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut ids = HashMap::new();
    let mut canonicals = HashMap::new();
    let all_ids = terms
        .iter()
        .map(|term| term.id.as_str())
        .collect::<HashSet<_>>();

    for term in terms {
        if let Some(first) = ids.insert(term.id.clone(), term.canonical.clone()) {
            diagnostics.push(simple_diagnostic(
                "TERM-STABLE-ID-001",
                "Technical glossary contains a duplicate stable term ID.",
                serde_json::json!({"id": term.id, "first": first, "second": term.canonical}),
            ));
        }
        let canonical = normalize_identity(&term.canonical);
        if let Some(first) = canonicals.insert(canonical.clone(), term.canonical.clone()) {
            diagnostics.push(simple_diagnostic(
                "TERM-DUP-001",
                "Technical glossary contains duplicate term identities.",
                serde_json::json!({"first": first, "second": term.canonical, "normalized": canonical}),
            ));
        }
        if term.roles.is_empty() {
            diagnostics.push(simple_diagnostic(
                "TERM-ROLE-001",
                "Technical term must declare at least one grammatical role.",
                serde_json::json!({"term_id": term.id}),
            ));
        }
        for form in &term.forms {
            if form.roles.is_empty()
                || form
                    .roles
                    .iter()
                    .any(|role| !term.roles.contains(role))
            {
                diagnostics.push(simple_diagnostic(
                    "TERM-FORM-001",
                    "Technical term form roles must be non-empty and governed by the term roles.",
                    serde_json::json!({"term_id": term.id, "form": form.text, "roles": form.roles}),
                ));
            }
        }
        for source in &term.sources {
            if !term.source_catalog.contains_key(&source.source) {
                diagnostics.push(simple_diagnostic(
                    "TERM-SOURCE-001",
                    "Technical term references an unknown terminology source.",
                    serde_json::json!({"term_id": term.id, "source": source.source}),
                ));
            }
        }
        if let Some(replacement) = &term.replacement {
            if term.status != TermStatus::Deprecated || !all_ids.contains(replacement.as_str()) {
                diagnostics.push(simple_diagnostic(
                    "TERM-REPLACEMENT-001",
                    "Technical term replacement must name an existing term and may only be set on a deprecated term.",
                    serde_json::json!({"term_id": term.id, "replacement": replacement}),
                ));
            }
        }
    }

    diagnostics
}

fn identity_conflict(
    first_term: &str,
    first_kind: GlossaryIdentityKind,
    second_term: &str,
    second_kind: GlossaryIdentityKind,
    normalized: &str,
) -> Diagnostic {
    simple_diagnostic(
        "TERM-ID-CONFLICT-001",
        "Technical glossary contains a conflicting canonical term, alias, or form identity.",
        serde_json::json!({
            "first": {"term": first_term, "identity_kind": first_kind},
            "second": {"term": second_term, "identity_kind": second_kind},
            "normalized": normalized,
        }),
    )
}

fn simple_diagnostic(code: &str, message: &str, evidence: serde_json::Value) -> Diagnostic {
    Diagnostic {
        code: code.into(),
        severity: Severity::Error,
        message: message.into(),
        span: Span { start: 0, end: 0 },
        rules: Vec::new(),
        evidence: Some(evidence),
        autofix: None,
    }
}

fn deduplicate_diagnostics(diagnostics: &mut Vec<Diagnostic>) {
    let mut seen = HashSet::new();
    diagnostics.retain(|diagnostic| {
        seen.insert((
            diagnostic.code.clone(),
            diagnostic.evidence.as_ref().map(ToString::to_string),
        ))
    });
}

fn normalize_identity(value: &str) -> String {
    value
        .split_ascii_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn stable_id(value: &str) -> String {
    let mut id = String::new();
    let mut needs_dash = false;
    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            if needs_dash && !id.is_empty() {
                id.push('-');
            }
            id.push(character);
            needs_dash = false;
        } else {
            needs_dash = true;
        }
    }
    id.trim_matches('-').to_owned()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LegacyTechnicalTermKind {
    TechnicalNoun,
    TechnicalVerb,
    TechnicalNounAndVerb,
}

#[derive(Debug, Clone, Deserialize)]
struct LegacyTechnicalTerm {
    term: String,
    kind: LegacyTechnicalTermKind,
    definition: String,
    #[allow(dead_code)]
    domain: String,
    #[allow(dead_code)]
    preferred: bool,
    #[serde(default)]
    forms: Vec<String>,
    aliases: Vec<String>,
    examples: Vec<String>,
    provenance: Vec<String>,
    status: TermStatus,
}

#[derive(Debug, Clone, Deserialize)]
struct LegacyGlossary {
    terms: Vec<LegacyTechnicalTerm>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_glossary_compiles_into_one_runtime_model() {
        let glossary =
            Glossary::from_json(include_str!("../../../fixtures/glossary/valid.json")).unwrap();
        let matched = glossary.lookup_identity("busway").unwrap();
        assert_eq!(matched.term.status, TermStatus::Approved);
        assert!(matched.term.has_role(TermRole::Noun));
        assert!(glossary.validate().is_empty());
    }

    #[test]
    fn structured_form_retains_role_evidence() {
        let glossary = Glossary::from_json(
            r#"{
              "schema":"ste-terminology/v2",
              "domain":"software",
              "sources":{"spec":{"title":"Specification"}},
              "terms":[{
                "id":"retry",
                "canonical":"retry",
                "roles":["noun","verb"],
                "definition":"A repeated attempt.",
                "forms":[{"text":"retries","roles":["noun","verb"]}],
                "aliases":[],
                "sources":[{"source":"spec","supports":["admission","definition","role","forms","status"]}],
                "status":"approved"
              }]
            }"#,
        )
        .unwrap();
        let matched = glossary.lookup_identity("RETRIES").unwrap();
        assert_eq!(matched.identity_kind, GlossaryIdentityKind::Form);
        assert_eq!(matched.roles, &[TermRole::Noun, TermRole::Verb]);
    }

    #[test]
    fn aliases_retain_alias_kind() {
        let glossary = Glossary::from_json(
            r#"{
              "schema":"ste-terminology/v2",
              "domain":"github",
              "sources":{"spec":{"title":"Specification"}},
              "terms":[{
                "id":"pull-request",
                "canonical":"pull request",
                "roles":["noun"],
                "definition":"A proposed change.",
                "forms":[],
                "aliases":[{"text":"PR","kind":"abbreviation"}],
                "sources":[{"source":"spec","supports":["admission","definition","role","alias","status"]}],
                "status":"approved"
              }]
            }"#,
        )
        .unwrap();
        let matched = glossary.lookup_identity("pr").unwrap();
        assert_eq!(matched.identity_kind, GlossaryIdentityKind::Alias);
        assert_eq!(matched.alias_kind, Some(AliasKind::Abbreviation));
    }

    #[test]
    fn duplicate_identity_is_rejected_with_stable_code() {
        let glossary =
            Glossary::from_json(include_str!("../../../fixtures/glossary/duplicate.json")).unwrap();
        let diagnostics = glossary.validate();
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == "TERM-DUP-001"));
    }
}
