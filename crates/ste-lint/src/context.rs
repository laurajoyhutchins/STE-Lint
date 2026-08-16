use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LintContext {
    #[serde(default)]
    pub occurrences: Vec<OccurrenceFact>,
    #[serde(default)]
    pub topics: Vec<TopicFact>,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopicFact {
    pub start: usize,
    pub end: usize,
    pub topic: String,
    pub source: String,
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
            {
                return Err(format!(
                    "context occurrence {index} must contain at least one evidence fact"
                ));
            }
        }
        for (index, topic) in self.topics.iter().enumerate() {
            if topic.source.trim().is_empty() {
                return Err(format!("context topic {index} source must be non-empty"));
            }
            if topic.topic.trim().is_empty() {
                return Err(format!("context topic {index} topic must be non-empty"));
            }
        }
        Ok(())
    }
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
