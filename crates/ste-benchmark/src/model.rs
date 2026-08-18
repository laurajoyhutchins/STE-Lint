use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use ste_core::{Outcome, Severity, Span};
use ste_lint::LintMode;

pub const BENCHMARK_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BenchmarkError {
    ManifestInvalid(String),
}

impl BenchmarkError {
    fn invalid(message: impl Into<String>) -> Self {
        Self::ManifestInvalid(message.into())
    }
}

impl Display for BenchmarkError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ManifestInvalid(message) => write!(formatter, "manifest_invalid: {message}"),
        }
    }
}

impl Error for BenchmarkError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SteClaimKind {
    ExplicitAsdSte100,
    ExplicitSte,
    QualifiedAsdSte100,
    Inferred,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationState {
    Unknown,
    RuleVerified,
    KnownViolation,
    ManuallyAdjudicated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cohort {
    DeclaredSteDeep,
    DeclaredSteBroad,
    ClaimNoneControl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RightsPolicy {
    ManifestOnly,
    LocalOnly,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceIdentity {
    pub sha256: String,
    pub byte_size: u64,
    pub physical_pages: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SteClaimEvidence {
    pub physical_page: u32,
    pub method: String,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SteClaim {
    pub kind: SteClaimKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<SteClaimEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Verification {
    pub state: VerificationState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRights {
    pub redistribution: RightsPolicy,
    pub source_cache: RightsPolicy,
    pub derived_text: RightsPolicy,
    pub committed_excerpt: RightsPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceManifest {
    pub schema_version: u32,
    pub id: String,
    pub source_family: String,
    pub publisher: String,
    pub title: String,
    pub document_type: String,
    pub url: String,
    pub media_type: String,
    pub retrieval_date: String,
    pub identity: SourceIdentity,
    pub ste_claim: SteClaim,
    pub verification: Verification,
    pub rights: SourceRights,
}

impl SourceManifest {
    pub fn validate(&self) -> Result<(), BenchmarkError> {
        require_schema_version(self.schema_version, "source")?;
        require_nonempty(&self.id, "source.id")?;
        require_nonempty(&self.source_family, "source.source_family")?;
        require_nonempty(&self.publisher, "source.publisher")?;
        require_nonempty(&self.title, "source.title")?;
        require_nonempty(&self.document_type, "source.document_type")?;

        let locator = self
            .url
            .strip_prefix("https://")
            .ok_or_else(|| BenchmarkError::invalid("source.url must use HTTPS"))?;
        let authority = locator.split('/').next().unwrap_or_default();
        if authority.is_empty() || self.url.chars().any(char::is_whitespace) {
            return Err(BenchmarkError::invalid(
                "source.url must be a non-empty HTTPS PDF locator",
            ));
        }

        if self.media_type != "application/pdf" {
            return Err(BenchmarkError::invalid(
                "source.media_type must be application/pdf",
            ));
        }
        validate_iso_date(&self.retrieval_date)?;

        if !is_lowercase_sha256(&self.identity.sha256) {
            return Err(BenchmarkError::invalid(
                "source.identity.sha256 must be 64 lowercase hexadecimal characters",
            ));
        }
        if self.identity.byte_size == 0 {
            return Err(BenchmarkError::invalid(
                "source.identity.byte_size must be positive",
            ));
        }
        if self.identity.physical_pages == 0 {
            return Err(BenchmarkError::invalid(
                "source.identity.physical_pages must be positive",
            ));
        }

        if self.ste_claim.kind != SteClaimKind::None && self.ste_claim.evidence.is_none() {
            return Err(BenchmarkError::invalid(
                "source.ste_claim.evidence is required when a claim is present",
            ));
        }
        if let Some(evidence) = &self.ste_claim.evidence {
            if evidence.physical_page == 0
                || evidence.physical_page > self.identity.physical_pages
            {
                return Err(BenchmarkError::invalid(
                    "source.ste_claim.evidence.physical_page must be within the PDF",
                ));
            }
            require_nonempty(&evidence.method, "source.ste_claim.evidence.method")?;
            require_nonempty(&evidence.note, "source.ste_claim.evidence.note")?;
        }

        if self.rights.redistribution != RightsPolicy::ManifestOnly
            || self.rights.source_cache != RightsPolicy::LocalOnly
            || self.rights.derived_text != RightsPolicy::LocalOnly
            || self.rights.committed_excerpt != RightsPolicy::None
        {
            return Err(BenchmarkError::invalid(
                "source.rights must use the seed-v1 manifest-only/local-only policy",
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuiteManifest {
    pub schema_version: u32,
    pub id: String,
    pub selections: Vec<SuiteSelection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuiteSelection {
    pub id: String,
    pub source_id: String,
    pub cohort: Cohort,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub match_group: Option<String>,
    pub first_page: u32,
    pub last_page: u32,
    pub mode: LintMode,
}

impl SuiteManifest {
    pub fn validate(
        &self,
        sources: &BTreeMap<String, SourceManifest>,
    ) -> Result<(), BenchmarkError> {
        require_schema_version(self.schema_version, "suite")?;
        require_nonempty(&self.id, "suite.id")?;
        if self.selections.is_empty() {
            return Err(BenchmarkError::invalid(
                "suite.selections must contain at least one selection",
            ));
        }

        let mut selection_ids = BTreeSet::new();
        for selection in &self.selections {
            require_nonempty(&selection.id, "suite.selections[].id")?;
            require_nonempty(&selection.source_id, "suite.selections[].source_id")?;
            if !selection_ids.insert(selection.id.as_str()) {
                return Err(BenchmarkError::invalid(format!(
                    "suite selection id is duplicated: {}",
                    selection.id
                )));
            }
            if let Some(match_group) = &selection.match_group {
                require_nonempty(match_group, "suite.selections[].match_group")?;
            }
            if selection.first_page == 0 || selection.last_page == 0 {
                return Err(BenchmarkError::invalid("suite page ranges are one-based"));
            }
            if selection.first_page > selection.last_page {
                return Err(BenchmarkError::invalid(
                    "suite first_page must not exceed last_page",
                ));
            }

            let source = sources.get(&selection.source_id).ok_or_else(|| {
                BenchmarkError::invalid(format!(
                    "suite references unknown source: {}",
                    selection.source_id
                ))
            })?;
            source.validate()?;
            if selection.last_page > source.identity.physical_pages {
                return Err(BenchmarkError::invalid(format!(
                    "suite selection {} exceeds source page count",
                    selection.id
                )));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkDiagnostic {
    pub code: String,
    pub severity: Severity,
    pub rules: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageObservation {
    pub source_id: String,
    pub selection_id: String,
    pub cohort: Cohort,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub match_group: Option<String>,
    pub physical_page: u32,
    pub mode: LintMode,
    pub normalized_text_sha256: String,
    pub normalized_byte_count: u64,
    pub word_count: u64,
    pub outcome: Outcome,
    pub diagnostics: Vec<BenchmarkDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkResult {
    pub schema_version: u32,
    pub suite_id: String,
    pub authoritative_runtime: bool,
    pub pages: Vec<PageObservation>,
}

impl BenchmarkResult {
    pub fn validate(&self) -> Result<(), BenchmarkError> {
        require_schema_version(self.schema_version, "result")?;
        require_nonempty(&self.suite_id, "result.suite_id")?;

        for page in &self.pages {
            require_nonempty(&page.source_id, "result.pages[].source_id")?;
            require_nonempty(&page.selection_id, "result.pages[].selection_id")?;
            if let Some(match_group) = &page.match_group {
                require_nonempty(match_group, "result.pages[].match_group")?;
            }
            if page.physical_page == 0 {
                return Err(BenchmarkError::invalid(
                    "result.pages[].physical_page must be one-based",
                ));
            }
            if !is_lowercase_sha256(&page.normalized_text_sha256) {
                return Err(BenchmarkError::invalid(
                    "result.pages[].normalized_text_sha256 must be lowercase SHA-256",
                ));
            }
            let byte_count = usize::try_from(page.normalized_byte_count).map_err(|_| {
                BenchmarkError::invalid(
                    "result.pages[].normalized_byte_count does not fit this platform",
                )
            })?;
            for diagnostic in &page.diagnostics {
                require_nonempty(&diagnostic.code, "result.pages[].diagnostics[].code")?;
                if diagnostic.span.start > diagnostic.span.end || diagnostic.span.end > byte_count {
                    return Err(BenchmarkError::invalid(
                        "result diagnostic span must fit the normalized page bytes",
                    ));
                }
            }
        }

        Ok(())
    }
}

fn require_schema_version(version: u32, kind: &str) -> Result<(), BenchmarkError> {
    if version == BENCHMARK_SCHEMA_VERSION {
        Ok(())
    } else {
        Err(BenchmarkError::invalid(format!(
            "{kind}.schema_version must be {BENCHMARK_SCHEMA_VERSION}"
        )))
    }
}

fn require_nonempty(value: &str, field: &str) -> Result<(), BenchmarkError> {
    if value.trim().is_empty() {
        Err(BenchmarkError::invalid(format!(
            "{field} must be non-empty"
        )))
    } else {
        Ok(())
    }
}

fn is_lowercase_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_iso_date(value: &str) -> Result<(), BenchmarkError> {
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !bytes[..4].iter().all(u8::is_ascii_digit)
        || !bytes[5..7].iter().all(u8::is_ascii_digit)
        || !bytes[8..].iter().all(u8::is_ascii_digit)
    {
        return Err(BenchmarkError::invalid(
            "source.retrieval_date must be an ISO YYYY-MM-DD date",
        ));
    }

    let year = value[..4]
        .parse::<u32>()
        .map_err(|_| BenchmarkError::invalid("source.retrieval_date has an invalid year"))?;
    let month = value[5..7]
        .parse::<u32>()
        .map_err(|_| BenchmarkError::invalid("source.retrieval_date has an invalid month"))?;
    let day = value[8..]
        .parse::<u32>()
        .map_err(|_| BenchmarkError::invalid("source.retrieval_date has an invalid day"))?;

    let leap_year = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap_year => 29,
        2 => 28,
        _ => 0,
    };
    if year == 0 || day == 0 || day > max_day {
        return Err(BenchmarkError::invalid(
            "source.retrieval_date must be a real calendar date",
        ));
    }

    Ok(())
}
