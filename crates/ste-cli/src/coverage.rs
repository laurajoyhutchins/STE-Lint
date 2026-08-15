use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CoverageStatus {
    Implemented,
    Partial,
    ContextRequired,
    NotImplemented,
}

impl CoverageStatus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Implemented => "implemented",
            Self::Partial => "partial",
            Self::ContextRequired => "context_required",
            Self::NotImplemented => "not_implemented",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RuleCoverage {
    pub id: String,
    pub status: CoverageStatus,
    pub diagnostic_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RuleCoverageManifest {
    pub standard: String,
    pub issue: u8,
    pub total_rules: usize,
    pub full_compliance_claimed: bool,
    pub coverage_model: String,
    pub rules: Vec<RuleCoverage>,
}

impl RuleCoverageManifest {
    pub(crate) fn embedded() -> Result<Self, String> {
        let manifest: Self = serde_json::from_str(include_str!("../../../data/rules.json"))
            .map_err(|error| format!("embedded rule coverage manifest is invalid JSON: {error}"))?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub(crate) fn status_counts(&self) -> BTreeMap<CoverageStatus, usize> {
        let mut counts = BTreeMap::new();
        for rule in &self.rules {
            *counts.entry(rule.status).or_insert(0) += 1;
        }
        counts
    }

    fn validate(&self) -> Result<(), String> {
        if self.standard != "ASD-STE100" || self.issue != 9 {
            return Err("rule coverage manifest must describe ASD-STE100 Issue 9".into());
        }
        if self.full_compliance_claimed {
            return Err(
                "rule coverage manifest must not claim full compliance while rules remain incomplete"
                    .into(),
            );
        }
        if self.total_rules != 53 || self.rules.len() != self.total_rules {
            return Err(format!(
                "rule coverage manifest cardinality mismatch: total_rules={}, entries={}",
                self.total_rules,
                self.rules.len()
            ));
        }
        let ids = self
            .rules
            .iter()
            .map(|rule| rule.id.as_str())
            .collect::<HashSet<_>>();
        if ids.len() != self.rules.len() {
            return Err("rule coverage manifest contains duplicate rule ids".into());
        }
        for expected in [
            "1.1", "1.14", "2.1", "2.2", "3.1", "3.7", "4.1", "4.5", "5.1", "5.5", "6.1", "6.6",
            "7.1", "7.3", "8.1", "8.7", "9.1", "9.4",
        ] {
            if !ids.contains(expected) {
                return Err(format!("rule coverage manifest is missing rule {expected}"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_manifest_tracks_complete_issue_nine_coverage() {
        let manifest = RuleCoverageManifest::embedded().unwrap();
        assert_eq!(manifest.total_rules, 53);
        assert_eq!(manifest.rules.len(), 53);
        assert!(!manifest.full_compliance_claimed);
    }

    #[test]
    fn representative_statuses_are_conservative() {
        let manifest = RuleCoverageManifest::embedded().unwrap();
        let status = |id: &str| {
            manifest
                .rules
                .iter()
                .find(|rule| rule.id == id)
                .map(|rule| rule.status)
                .unwrap()
        };
        assert_eq!(status("8.5"), CoverageStatus::Implemented);
        assert_eq!(status("8.7"), CoverageStatus::Implemented);
        assert_eq!(status("3.4"), CoverageStatus::Partial);
        assert_eq!(status("5.3"), CoverageStatus::Partial);
        assert_eq!(status("9.3"), CoverageStatus::Partial);
    }
}
