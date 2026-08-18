use ste_core::Severity;
use ste_data::RuntimeLexicon;
use ste_glossary::Glossary;
use ste_lint::{LintMode, LintOptions, lint_text};

fn lexicon(include_removed_adjective: bool) -> RuntimeLexicon {
    let adjective = if include_removed_adjective {
        r#",{"lemma":"REMOVED","status":"approved","part_of_speech":"adjective","forms":["REMOVED"],"senses":[],"alternatives":[],"restrictions":[]}"#
    } else {
        ""
    };
    RuntimeLexicon::from_json(&format!(
        r#"{{
          "metadata":{{"standard":"ASD-STE100","issue":9,"date":"2025-01-15","scope":"synthetic_deterministic_verb_completion"}},
          "entries":[
            {{"lemma":"THE","status":"approved","part_of_speech":"article","forms":["THE"],"senses":[],"alternatives":[],"restrictions":[]}},
            {{"lemma":"UNIT","status":"approved","part_of_speech":"noun","forms":["UNIT"],"senses":[],"alternatives":[],"restrictions":[]}},
            {{"lemma":"PART","status":"approved","part_of_speech":"noun","forms":["PART"],"senses":[],"alternatives":[],"restrictions":[]}},
            {{"lemma":"READY","status":"approved","part_of_speech":"adjective","forms":["READY"],"senses":[],"alternatives":[],"restrictions":[]}},
            {{"lemma":"REMOVE","status":"approved","part_of_speech":"verb","forms":["REMOVE","REMOVES","REMOVED"],"verb_paradigm":{{"classification":"lexical","source_sequence":["REMOVE","REMOVES","REMOVED","REMOVED"],"base_form":"REMOVE","simple_present_variants":["REMOVES"],"simple_past_variants":["REMOVED"],"past_participle":"REMOVED"}},"senses":[],"alternatives":[],"restrictions":[]}},
            {{"lemma":"BE","status":"approved","part_of_speech":"verb","forms":["IS","WAS"],"verb_paradigm":{{"classification":"irregular_auxiliary","source_sequence":["BE","IS","WAS","BEEN"],"base_form":"BE","simple_present_variants":["IS"],"simple_past_variants":["WAS"],"past_participle":"BEEN"}},"senses":[],"alternatives":[],"restrictions":[]}},
            {{"lemma":"HAVE","status":"approved","part_of_speech":"verb","forms":["HAVE","HAS","HAD"],"verb_paradigm":{{"classification":"irregular_auxiliary","source_sequence":["HAVE","HAS","HAD","HAD"],"base_form":"HAVE","simple_present_variants":["HAS"],"simple_past_variants":["HAD"],"past_participle":"HAD"}},"senses":[],"alternatives":[],"restrictions":[]}},
            {{"lemma":"WILL","status":"approved","part_of_speech":"verb","forms":["WILL"],"verb_paradigm":{{"classification":"defective_modal","source_sequence":["WILL"],"base_form":"WILL","simple_present_variants":[],"simple_past_variants":[],"past_participle":null}},"senses":[],"alternatives":[],"restrictions":[]}}
            {adjective}
          ]
        }}"#
    ))
    .unwrap()
}

fn glossary() -> Glossary {
    Glossary::from_json(
        r#"{
          "schema":"ste-terminology/v2",
          "domain":"synthetic-ing",
          "sources":{"fixture":{"title":"Synthetic -ing terminology"}},
          "terms":[
            {
              "id":"processing",
              "canonical":"processing",
              "roles":["noun"],
              "definition":"Synthetic governed technical noun.",
              "forms":[],
              "aliases":[],
              "sources":[{"source":"fixture","supports":["admission","definition","role","forms","alias","status"]}],
              "status":"approved"
            },
            {
              "id":"processing-unit",
              "canonical":"processing unit",
              "roles":["noun"],
              "definition":"Synthetic governed multiword technical noun.",
              "forms":[],
              "aliases":[],
              "sources":[{"source":"fixture","supports":["admission","definition","role","forms","alias","status"]}],
              "status":"approved"
            }
          ]
        }"#,
    )
    .unwrap()
}

fn lint(text: &str, mode: LintMode, adjective_identity: bool) -> Vec<ste_core::Diagnostic> {
    lint_text(
        text,
        &lexicon(adjective_identity),
        None,
        LintOptions { mode, fix: false },
    )
    .diagnostics
}

#[test]
fn progressive_construction_is_rejected_by_rules_3_2_3_4_and_3_5() {
    let diagnostics = lint(
        "THE UNIT IS REMOVING THE PART.",
        LintMode::Descriptive,
        false,
    );
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.rules.iter().any(|rule| rule == "3.2")
            && diagnostic.rules.iter().any(|rule| rule == "3.4")
            && diagnostic.rules.iter().any(|rule| rule == "3.5")
            && diagnostic.severity == Severity::Error
    }));
}

#[test]
fn passive_participle_is_rejected_in_descriptive_text_when_it_has_only_verb_identity() {
    let diagnostics = lint("THE PART IS REMOVED.", LintMode::Descriptive, false);
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.rules.iter().any(|rule| rule == "3.3")
            && diagnostic.rules.iter().any(|rule| rule == "3.4")
            && diagnostic.severity == Severity::Error
    }));
}

#[test]
fn passive_or_adjectival_participle_with_competing_authoritative_identities_blocks() {
    let diagnostics = lint("THE PART IS REMOVED.", LintMode::Descriptive, true);
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.rules.iter().any(|rule| rule == "3.3")
            && diagnostic.severity == Severity::Blocked
    }));
}

#[test]
fn simple_future_with_will_and_base_form_is_not_rejected_as_complex() {
    let diagnostics = lint(
        "THE UNIT WILL REMOVE THE PART.",
        LintMode::Descriptive,
        false,
    );
    assert!(diagnostics.iter().all(|diagnostic| {
        !(diagnostic.rules.iter().any(|rule| rule == "3.2")
            || diagnostic.rules.iter().any(|rule| rule == "3.4"))
    }));
}

#[test]
fn source_linked_ing_form_used_as_a_verb_is_a_rule_3_5_error() {
    let diagnostics = lint(
        "THE UNIT IS REMOVING THE PART.",
        LintMode::Descriptive,
        false,
    );
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.rules.iter().any(|rule| rule == "3.5") && diagnostic.severity == Severity::Error
    }));
}

#[test]
fn governed_ing_technical_noun_is_allowed() {
    let lexicon = lexicon(false);
    let glossary = glossary();
    let diagnostics = lint_text(
        "PROCESSING IS READY.",
        &lexicon,
        Some(&glossary),
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    )
    .diagnostics;
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.rules.iter().any(|rule| rule == "3.5"))
    );
}

#[test]
fn governed_ing_modifier_inside_technical_noun_is_allowed() {
    let lexicon = lexicon(false);
    let glossary = glossary();
    let diagnostics = lint_text(
        "THE PROCESSING UNIT IS READY.",
        &lexicon,
        Some(&glossary),
        LintOptions {
            mode: LintMode::Descriptive,
            fix: false,
        },
    )
    .diagnostics;
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.rules.iter().any(|rule| rule == "3.5"))
    );
}

#[test]
fn conditional_instruction_still_requires_imperative_command() {
    let diagnostics = lint(
        "IF THE UNIT IS READY, REMOVES THE PART.",
        LintMode::Procedural,
        false,
    );
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.rules.iter().any(|rule| rule == "5.3") && diagnostic.severity == Severity::Error
    }));
}

#[test]
fn ordinary_procedural_sentence_requires_imperative_command() {
    let diagnostics = lint("THE UNIT REMOVES THE PART.", LintMode::Procedural, false);
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.rules.iter().any(|rule| rule == "5.3") && diagnostic.severity == Severity::Error
    }));
}

#[test]
fn approved_base_form_instruction_is_imperative() {
    let diagnostics = lint("REMOVE THE PART.", LintMode::Procedural, false);
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.rules.iter().any(|rule| rule == "5.3"))
    );
}
