use serde_json::json;
use ste_core::{Diagnostic, Severity, Span};
use ste_data::{ApprovalStatus, LexiconEntry, PartOfSpeech};

use crate::AnalysisDocument;
use crate::analysis::linguistic::GenericVerbForm;

pub(crate) fn check(analysis: &AnalysisDocument<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (token_index, token) in analysis.tokens().iter().enumerate() {
        // Exact runtime forms are authoritative and always win over generic morphology.
        if analysis.dictionary_match_at(token_index, 1).is_some() {
            continue;
        }
        let Some(generic) = analysis.linguistic_token(token_index) else {
            continue;
        };
        let Some(lemma) = generic.lemma.as_deref() else {
            continue;
        };

        if is_generic_verb_inflection(generic) {
            let candidates = approved_lemma_candidates(analysis, lemma, PartOfSpeech::Verb);
            if let [entry] = candidates.as_slice() {
                diagnostics.push(out_of_inventory_diagnostic(
                    token.text,
                    token.start,
                    token.end,
                    entry,
                    "verb",
                    generic_verb_form_names(&generic.verb_forms),
                    vec!["1.4".into(), "3.1".into()],
                ));
            }
        }

        if generic.comparative_adjective || generic.superlative_adjective {
            let candidates = approved_lemma_candidates(analysis, lemma, PartOfSpeech::Adjective);
            if let [entry] = candidates.as_slice() {
                let form = if generic.comparative_adjective && generic.superlative_adjective {
                    "comparative_or_superlative"
                } else if generic.comparative_adjective {
                    "comparative"
                } else {
                    "superlative"
                };
                diagnostics.push(out_of_inventory_diagnostic(
                    token.text,
                    token.start,
                    token.end,
                    entry,
                    "adjective",
                    vec![form],
                    vec!["1.4".into()],
                ));
            }
        }
    }

    diagnostics
}

fn approved_lemma_candidates<'a>(
    analysis: &'a AnalysisDocument<'_>,
    lemma: &str,
    part_of_speech: PartOfSpeech,
) -> Vec<&'a LexiconEntry> {
    analysis
        .lexicon()
        .lookup_lemma(lemma)
        .into_iter()
        .filter(|entry| {
            entry.status == ApprovalStatus::Approved && entry.part_of_speech == Some(part_of_speech)
        })
        .collect()
}

fn is_generic_verb_inflection(
    generic: &crate::analysis::linguistic::LinguisticTokenEvidence,
) -> bool {
    generic.verb
        && generic
            .verb_forms
            .iter()
            .any(|form| !matches!(form, GenericVerbForm::Lemma))
}

fn generic_verb_form_names(forms: &[GenericVerbForm]) -> Vec<&'static str> {
    forms
        .iter()
        .filter_map(|form| match form {
            GenericVerbForm::Lemma => None,
            GenericVerbForm::Past => Some("past"),
            GenericVerbForm::SimplePast => Some("simple_past"),
            GenericVerbForm::PastParticiple => Some("past_participle"),
            GenericVerbForm::Progressive => Some("progressive"),
            GenericVerbForm::ThirdPersonSingularPresent => Some("third_person_singular_present"),
        })
        .collect()
}

fn out_of_inventory_diagnostic(
    observed: &str,
    start: usize,
    end: usize,
    entry: &LexiconEntry,
    part_of_speech: &str,
    generic_forms: Vec<&str>,
    rules: Vec<String>,
) -> Diagnostic {
    Diagnostic {
        code: "STE-FORM-001".into(),
        severity: Severity::Error,
        message: format!(
            "Do not use '{observed}' as this {part_of_speech} form of '{}'; the source-backed dictionary entry does not supply this form.",
            entry.lemma
        ),
        span: Span { start, end },
        rules,
        evidence: Some(json!({
            "coverage": "source_linked_out_of_inventory_form_v1",
            "observed": observed,
            "dictionary_lemma": entry.lemma,
            "dictionary_part_of_speech": part_of_speech,
            "generic_morphology": generic_forms,
            "authority": "Harper supplies only generic morphology/lemma linkage; the ASD-STE100 runtime entry remains authoritative for the permitted form inventory"
        })),
        autofix: None,
    }
}
