use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageStatus {
    Implemented,
    Partial,
    ContextRequired,
    NotImplemented,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleCoverage {
    pub id: String,
    pub status: CoverageStatus,
    pub diagnostic_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleCoverageManifest {
    pub standard: String,
    pub issue: u8,
    pub total_rules: usize,
    pub full_compliance_claimed: bool,
    pub coverage_model: String,
    pub rules: Vec<RuleCoverage>,
}

impl RuleCoverageManifest {
    pub fn embedded() -> Result<Self, serde_json::Error> {
        serde_json::from_str(include_str!("../../../data/rules.json"))
    }

    pub fn status_counts(&self) -> BTreeMap<CoverageStatus, usize> {
        let mut counts = BTreeMap::new();
        for rule in &self.rules {
            *counts.entry(rule.status).or_insert(0) += 1;
        }
        counts
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn embedded_manifest_tracks_the_complete_issue_nine_rule_id_set() {
        let manifest = RuleCoverageManifest::embedded().unwrap();
        assert_eq!(manifest.standard, "ASD-STE100");
        assert_eq!(manifest.issue, 9);
        assert_eq!(manifest.total_rules, 53);
        assert_eq!(manifest.rules.len(), 53);
        assert!(!manifest.full_compliance_claimed);

        let ids = manifest
            .rules
            .iter()
            .map(|rule| rule.id.as_str())
            .collect::<HashSet<_>>();
        assert_eq!(ids.len(), 53);
        for expected in ["1.1", "1.14", "2.1", "3.7", "4.5", "5.5", "6.6", "7.3", "8.7", "9.4"] {
            assert!(ids.contains(expected), "missing Issue 9 rule {expected}");
        }
    }

    #[test]
    fn representative_statuses_remain_conservative() {
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
        assert_eq!(status("5.3"), CoverageStatus::ContextRequired);
        assert_eq!(status("9.3"), CoverageStatus::NotImplemented);
    }
}
