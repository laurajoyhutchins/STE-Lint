use serde_json::{Value, json};
use ste_data::{LexiconEntry, PartOfSpeech};

pub(crate) fn dictionary_evidence(
    candidates: &[&LexiconEntry],
    requires_disambiguation: bool,
) -> Value {
    let evidence_candidates = candidates
        .iter()
        .map(|entry| {
            json!({
                "lemma": entry.lemma,
                "part_of_speech": entry.part_of_speech,
                "status": entry.status,
                "senses": entry.senses,
                "alternatives": entry.alternatives,
                "restrictions": entry.restrictions,
                "interpretation_state": entry.interpretation_state,
                "provenance": entry.provenance,
            })
        })
        .collect::<Vec<_>>();
    let possible_parts_of_speech = distinct_parts_of_speech(candidates);
    let role_evidence = distinct_roles(&possible_parts_of_speech);

    json!({
        "candidates": evidence_candidates,
        "possible_parts_of_speech": possible_parts_of_speech,
        "role_evidence": role_evidence,
        "requires_disambiguation": requires_disambiguation,
    })
}

fn distinct_parts_of_speech(candidates: &[&LexiconEntry]) -> Vec<PartOfSpeech> {
    let mut parts = Vec::new();
    for entry in candidates {
        if let Some(part) = entry.part_of_speech
            && !parts.contains(&part)
        {
            parts.push(part);
        }
    }
    parts
}

fn distinct_roles(parts: &[PartOfSpeech]) -> Vec<&'static str> {
    let mut roles = Vec::new();
    for part in parts {
        let role = match part {
            PartOfSpeech::Noun | PartOfSpeech::Pronoun => "nominal",
            PartOfSpeech::Verb => "verbal",
            PartOfSpeech::Adjective | PartOfSpeech::Adverb => "modifier",
            PartOfSpeech::Article | PartOfSpeech::Preposition | PartOfSpeech::Conjunction => {
                "function_word"
            }
        };
        if !roles.contains(&role) {
            roles.push(role);
        }
    }
    roles
}
