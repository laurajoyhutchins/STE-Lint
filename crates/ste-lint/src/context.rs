use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LintContext {
    #[serde(default)]
    pub occurrences: Vec<OccurrenceFact>,
    #[serde(default)]
    pub named_entities: Vec<NamedEntityFact>,
    #[serde(default)]
    pub measurement_units: Vec<MeasurementUnitFact>,
    #[serde(default)]
    pub topics: Vec<TopicFact>,
    #[serde(default)]
    pub semantic_orderings: Vec<SemanticOrderingFact>,
    #[serde(default)]
    pub safety_facts: Vec<SafetyFact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OccurrenceFact {
    pub start: usize,
    pub end: usize,
    pub source: String,
    #[serde(default)]
    pub dictionary_meaning: Option<DictionaryMeaningUse>,
    #[serde(default)]
    pub technical_noun_scope: Option<TechnicalNounScope>,
    #[serde(default)]
    pub spelling: Option<SpellingUse>,
    #[serde(default)]
    pub official_technical_name: bool,
    #[serde(default)]
    pub count_group: Option<CountGroupKind>,
    #[serde(default)]
    pub text_authority: Option<TextAuthorityKind>,
    #[serde(default)]
    pub hyphen_direct_relation: Option<bool>,
    #[serde(default)]
    pub parenthesis_use: Option<ParenthesisUseKind>,
    #[serde(default)]
    pub phrasal_verb: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedEntityFact {
    pub id: String,
    pub canonical: String,
    pub class: NamedEntityClass,
    #[serde(default)]
    pub forms: Vec<String>,
    pub source: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NamedEntityClass {
    Person,
    Group,
    Organization,
    GeopoliticalEntity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeasurementUnitFact {
    pub id: String,
    pub canonical: String,
    #[serde(default)]
    pub forms: Vec<String>,
    pub source: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextAuthorityKind {
    ProtectedText,
    QuotedExternalText,
    CodeOrVerbatim,
    Formula,
    Title,
    Placard,
    Label,
    DocumentNumbering,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopicFact {
    pub start: usize,
    pub end: usize,
    pub topic: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticOrderingFact {
    pub before: SemanticOrderTarget,
    pub after: SemanticOrderTarget,
    pub source: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticOrderTarget {
    pub kind: SemanticOrderTargetKind,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticOrderTargetKind {
    Sentence,
    Paragraph,
    Topic,
    EntityMention,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetyFact {
    pub start: usize,
    pub end: usize,
    pub source: String,
    #[serde(default)]
    pub safety_level: Option<SafetyLevelFact>,
    #[serde(default)]
    pub actor: Option<SafetySpanFact>,
    #[serde(default)]
    pub command: Option<SafetySpanFact>,
    #[serde(default)]
    pub hazard: Option<SafetySpanFact>,
    #[serde(default)]
    pub consequence: Option<SafetySpanFact>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetySpanFact {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SafetyLevelFact {
    Warning,
    Caution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DictionaryMeaningUse {
    Approved,
    NotApproved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TechnicalNounScope {
    International,
    Regional,
    Slang,
    Jargon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpellingUse {
    American,
    NonAmerican,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CountGroupKind {
    Number,
    NumberWithUnit,
    Abbreviation,
    AlphanumericIdentifier,
    QuotedText,
    Title,
    Heading,
    Placard,
    Label,
    ProperNoun,
    ProperNounPerson,
    ProperNounGroup,
    ProperNounOrganization,
    ProperNounGeopoliticalEntity,
    Parenthetical,
    HyphenatedWord,
    DocumentNumberingExcluded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParenthesisUseKind {
    Reference,
    ItemIdentifier,
    WorkStepIdentifier,
    Abbreviation,
    SingularPlural,
    Explanation,
    Alternative,
    Other,
}

impl LintContext {
    pub fn from_json(source: &str) -> Result<Self, String> {
        let context: Self = serde_json::from_str(source)
            .map_err(|error| format!("invalid STE lint context JSON: {error}"))?;
        context.validate_evidence()?;
        Ok(context)
    }

    pub fn validate(&self, text_len: usize) -> Result<(), String> {
        self.validate_evidence()?;
        for (index, occurrence) in self.occurrences.iter().enumerate() {
            validate_span(
                "context occurrence",
                index,
                occurrence.start,
                occurrence.end,
                text_len,
            )?;
        }
        for (index, topic) in self.topics.iter().enumerate() {
            validate_span("context topic", index, topic.start, topic.end, text_len)?;
        }
        for (index, ordering) in self.semantic_orderings.iter().enumerate() {
            validate_span(
                "context semantic ordering before target",
                index,
                ordering.before.start,
                ordering.before.end,
                text_len,
            )?;
            validate_span(
                "context semantic ordering after target",
                index,
                ordering.after.start,
                ordering.after.end,
                text_len,
            )?;
        }
        for (index, fact) in self.safety_facts.iter().enumerate() {
            validate_span("context safety fact", index, fact.start, fact.end, text_len)?;
            validate_safety_component("actor", index, fact.actor, fact, text_len)?;
            validate_safety_component("command", index, fact.command, fact, text_len)?;
            validate_safety_component("hazard", index, fact.hazard, fact, text_len)?;
            validate_safety_component("consequence", index, fact.consequence, fact, text_len)?;
        }
        self.validate_count_group_overlaps()?;
        Ok(())
    }

    /// Validate all caller-provided byte spans against the exact source text.
    /// Length-only validation cannot distinguish a UTF-8 continuation byte.
    pub fn validate_for_text(&self, text: &str) -> Result<(), String> {
        self.validate(text.len())?;
        for (index, occurrence) in self.occurrences.iter().enumerate() {
            validate_char_boundaries(
                "context occurrence",
                index,
                occurrence.start,
                occurrence.end,
                text,
            )?;
        }
        for (index, topic) in self.topics.iter().enumerate() {
            validate_char_boundaries("context topic", index, topic.start, topic.end, text)?;
        }
        for (index, ordering) in self.semantic_orderings.iter().enumerate() {
            validate_char_boundaries(
                "context semantic ordering before target",
                index,
                ordering.before.start,
                ordering.before.end,
                text,
            )?;
            validate_char_boundaries(
                "context semantic ordering after target",
                index,
                ordering.after.start,
                ordering.after.end,
                text,
            )?;
        }
        for (index, fact) in self.safety_facts.iter().enumerate() {
            validate_char_boundaries("context safety fact", index, fact.start, fact.end, text)?;
            for (component, span) in [
                ("actor", fact.actor),
                ("command", fact.command),
                ("hazard", fact.hazard),
                ("consequence", fact.consequence),
            ] {
                if let Some(span) = span {
                    validate_char_boundaries(
                        &format!("context safety fact {component}"),
                        index,
                        span.start,
                        span.end,
                        text,
                    )?;
                }
            }
        }
        Ok(())
    }

    fn validate_evidence(&self) -> Result<(), String> {
        for (index, occurrence) in self.occurrences.iter().enumerate() {
            if occurrence.source.trim().is_empty() {
                return Err(format!(
                    "context occurrence {index} source must be non-empty"
                ));
            }
            if occurrence.dictionary_meaning.is_none()
                && occurrence.technical_noun_scope.is_none()
                && occurrence.spelling.is_none()
                && !occurrence.official_technical_name
                && occurrence.count_group.is_none()
                && occurrence.text_authority.is_none()
                && occurrence.hyphen_direct_relation.is_none()
                && occurrence.parenthesis_use.is_none()
                && occurrence.phrasal_verb.is_none()
            {
                return Err(format!(
                    "context occurrence {index} must contain at least one evidence fact"
                ));
            }
        }
        validate_named_entities(&self.named_entities)?;
        validate_measurement_units(&self.measurement_units)?;
        for (index, topic) in self.topics.iter().enumerate() {
            if topic.source.trim().is_empty() {
                return Err(format!("context topic {index} source must be non-empty"));
            }
            if topic.topic.trim().is_empty() {
                return Err(format!("context topic {index} topic must be non-empty"));
            }
        }
        for (index, ordering) in self.semantic_orderings.iter().enumerate() {
            if ordering.source.trim().is_empty() {
                return Err(format!(
                    "context semantic ordering {index} source must be non-empty"
                ));
            }
        }
        for (index, fact) in self.safety_facts.iter().enumerate() {
            if fact.source.trim().is_empty() {
                return Err(format!(
                    "context safety fact {index} source must be non-empty"
                ));
            }
            if fact.safety_level.is_none()
                && fact.actor.is_none()
                && fact.command.is_none()
                && fact.hazard.is_none()
                && fact.consequence.is_none()
            {
                return Err(format!(
                    "context safety fact {index} must contain at least one safety evidence fact"
                ));
            }
        }
        Ok(())
    }

    fn validate_count_group_overlaps(&self) -> Result<(), String> {
        let mut groups = self
            .occurrences
            .iter()
            .enumerate()
            .filter(|(_, occurrence)| occurrence.count_group.is_some())
            .map(|(index, occurrence)| (occurrence.start, occurrence.end, index))
            .collect::<Vec<_>>();
        groups.sort_by_key(|&(start, end, _)| (start, end));

        for pair in groups.windows(2) {
            let (left_start, left_end, left_index) = pair[0];
            let (right_start, right_end, right_index) = pair[1];
            if right_start < left_end {
                return Err(format!(
                    "context count groups {left_index} ({left_start}..{left_end}) and {right_index} ({right_start}..{right_end}) overlap"
                ));
            }
        }
        Ok(())
    }
}

fn validate_named_entities(facts: &[NamedEntityFact]) -> Result<(), String> {
    let mut ids = HashSet::new();
    let mut forms = HashSet::new();
    for (index, fact) in facts.iter().enumerate() {
        validate_global_authority(
            "named entity",
            index,
            &fact.id,
            &fact.canonical,
            &fact.source,
        )?;
        if !ids.insert(fact.id.to_ascii_lowercase()) {
            return Err(format!(
                "named entity {index} duplicates a governed entity id"
            ));
        }
        for surface in std::iter::once(&fact.canonical).chain(&fact.forms) {
            let normalized = normalize_surface(surface);
            if normalized.is_empty() {
                return Err(format!(
                    "named entity {index} contains an empty surface form"
                ));
            }
            if !forms.insert(normalized) {
                return Err(format!(
                    "named entity {index} has a surface form that collides with another governed entity"
                ));
            }
        }
    }
    Ok(())
}

fn validate_measurement_units(facts: &[MeasurementUnitFact]) -> Result<(), String> {
    let mut ids = HashSet::new();
    let mut forms = HashSet::new();
    for (index, fact) in facts.iter().enumerate() {
        validate_global_authority(
            "measurement unit",
            index,
            &fact.id,
            &fact.canonical,
            &fact.source,
        )?;
        if !ids.insert(fact.id.to_ascii_lowercase()) {
            return Err(format!(
                "measurement unit {index} duplicates a governed unit id"
            ));
        }
        for surface in std::iter::once(&fact.canonical).chain(&fact.forms) {
            let normalized = normalize_surface(surface);
            if normalized.is_empty() {
                return Err(format!(
                    "measurement unit {index} contains an empty surface form"
                ));
            }
            if !forms.insert(normalized) {
                return Err(format!(
                    "measurement unit {index} has a surface form that collides with another governed unit"
                ));
            }
        }
    }
    Ok(())
}

fn validate_global_authority(
    kind: &str,
    index: usize,
    id: &str,
    canonical: &str,
    source: &str,
) -> Result<(), String> {
    if id.trim().is_empty() {
        return Err(format!("context {kind} {index} id must be non-empty"));
    }
    if canonical.trim().is_empty() {
        return Err(format!(
            "context {kind} {index} canonical form must be non-empty"
        ));
    }
    if source.trim().is_empty() {
        return Err(format!("context {kind} {index} source must be non-empty"));
    }
    Ok(())
}

fn normalize_surface(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn validate_safety_component(
    component: &str,
    index: usize,
    span: Option<SafetySpanFact>,
    fact: &SafetyFact,
    text_len: usize,
) -> Result<(), String> {
    let Some(span) = span else {
        return Ok(());
    };
    validate_span(
        &format!("context safety fact {component}"),
        index,
        span.start,
        span.end,
        text_len,
    )?;
    if span.start < fact.start || span.end > fact.end {
        return Err(format!(
            "context safety fact {index} {component} span {}..{} must be contained in safety fact span {}..{}",
            span.start, span.end, fact.start, fact.end
        ));
    }
    Ok(())
}

fn validate_span(
    kind: &str,
    index: usize,
    start: usize,
    end: usize,
    text_len: usize,
) -> Result<(), String> {
    if start >= end {
        return Err(format!(
            "{kind} {index} must have start < end ({start}..{end})"
        ));
    }
    if end > text_len {
        return Err(format!(
            "{kind} {index} ends at {end}, beyond text length {text_len}"
        ));
    }
    Ok(())
}

fn validate_char_boundaries(
    kind: &str,
    index: usize,
    start: usize,
    end: usize,
    text: &str,
) -> Result<(), String> {
    if !text.is_char_boundary(start) || !text.is_char_boundary(end) {
        return Err(format!(
            "{kind} {index} span {start}..{end} is not on UTF-8 character boundaries"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod utf8_context_tests {
    use super::LintContext;

    #[test]
    fn rejects_interior_offsets_for_every_context_span_family() {
        let cases = [
            r#"{"occurrences":[{"start":1,"end":2,"source":"review","count_group":"number"}]}"#,
            r#"{"topics":[{"start":0,"end":1,"topic":"topic","source":"review"}]}"#,
            r#"{"semantic_orderings":[{"before":{"kind":"sentence","start":0,"end":1},"after":{"kind":"sentence","start":3,"end":4},"source":"review"}]}"#,
            r#"{"safety_facts":[{"start":0,"end":2,"source":"review","actor":{"start":0,"end":1}}]}"#,
        ];
        for raw in cases {
            let context = LintContext::from_json(raw).unwrap();
            let error = context.validate_for_text("é A").unwrap_err();
            assert!(error.contains("UTF-8 character boundaries"), "{error}");
        }
    }

    #[test]
    fn accepts_valid_unicode_and_empty_context_but_rejects_out_of_range_spans() {
        let context = LintContext::from_json(
            r#"{"occurrences":[{"start":0,"end":2,"source":"review","count_group":"number"}],"topics":[{"start":3,"end":4,"topic":"A","source":"review"}],"safety_facts":[{"start":0,"end":2,"source":"review","actor":{"start":0,"end":2}}]}"#,
        )
        .unwrap();
        context.validate_for_text("é A").unwrap();
        LintContext::default().validate_for_text("").unwrap();

        let out_of_range = LintContext::from_json(
            r#"{"occurrences":[{"start":0,"end":2,"source":"review","count_group":"number"}]}"#,
        )
        .unwrap();
        assert!(out_of_range.validate_for_text("").is_err());
    }

    #[test]
    fn public_lint_blocks_malformed_unicode_before_counting() {
        let lexicon = ste_data::RuntimeLexicon::embedded().unwrap();
        let context = LintContext::from_json(
            r#"{"occurrences":[{"start":1,"end":2,"source":"review","count_group":"number"}]}"#,
        )
        .unwrap();
        let result = crate::lint_text_with_context(
            "é A",
            &lexicon,
            None,
            Some(&context),
            crate::LintOptions {
                mode: crate::LintMode::Descriptive,
                fix: false,
            },
        );
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "STE-CTX-000");
    }
}
