use ste_data::RuntimeLexicon;
use ste_lint::{LintMode, LintOptions, LintResult, lint_text};

fn lint(text: &str, lexicon: &RuntimeLexicon, mode: LintMode) -> LintResult {
    lint_text(text, lexicon, None, LintOptions { mode, fix: false })
}

fn has_code(result: &LintResult, code: &str) -> bool {
    result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == code)
}

#[test]
fn resolved_progressive_ing_is_reported_for_rule_3_5() {
    let lexicon = progressive_lexicon(false);
    let result = lint("IT IS OPENING.", &lexicon, LintMode::Descriptive);

    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "STE-GRAM-002")
        .expect("resolved progressive -ing use should be reported");
    assert_eq!(diagnostic.rules, vec!["3.5"]);
    assert_eq!((diagnostic.span.start, diagnostic.span.end), (6, 13));
}

#[test]
fn resolved_nominal_ing_is_not_reported_for_rule_3_5() {
    let lexicon = progressive_lexicon(false);
    let result = lint("THE OPENING IS CLEAR.", &lexicon, LintMode::Descriptive);

    assert!(!has_code(&result, "STE-GRAM-002"));
}

#[test]
fn progressive_adjectival_ambiguity_is_not_guessed_for_rule_3_5() {
    let lexicon = progressive_lexicon(true);
    let result = lint("IT IS OPENING.", &lexicon, LintMode::Descriptive);

    assert!(!has_code(&result, "STE-GRAM-002"));
}

#[test]
fn resolved_procedural_passive_is_reported_for_rule_3_6() {
    let lexicon = passive_lexicon(false);
    let result = lint("THE UNIT IS CONNECTED.", &lexicon, LintMode::Procedural);

    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "STE-GRAM-003")
        .expect("resolved procedural passive should be reported");
    assert_eq!(diagnostic.rules, vec!["3.6"]);
    assert_eq!((diagnostic.span.start, diagnostic.span.end), (12, 21));
}

#[test]
fn resolved_descriptive_passive_is_not_reported_without_actor_semantics() {
    let lexicon = passive_lexicon(false);
    let result = lint("THE UNIT IS CONNECTED.", &lexicon, LintMode::Descriptive);

    assert!(!has_code(&result, "STE-GRAM-003"));
}

#[test]
fn passive_adjectival_ambiguity_is_not_guessed_for_rule_3_6() {
    let lexicon = passive_lexicon(true);
    let result = lint("THE UNIT IS CONNECTED.", &lexicon, LintMode::Procedural);

    assert!(!has_code(&result, "STE-GRAM-003"));
}

fn progressive_lexicon(include_adjective: bool) -> RuntimeLexicon {
    let adjective = if include_adjective {
        r#",{"lemma":"OPENING","status":"approved","part_of_speech":"adjective","forms":["OPENING"],"senses":[],"alternatives":[],"restrictions":[]}"#
    } else {
        ""
    };
    RuntimeLexicon::from_json(&format!(
        r#"{{
          "metadata":{{"standard":"ASD-STE100","issue":9,"date":"2025-01-15","scope":"synthetic_grammar_rule_promotion"}},
          "entries":[
            {{"lemma":"IT","status":"approved","part_of_speech":"pronoun","forms":["IT"],"senses":[],"alternatives":[],"restrictions":[]}},
            {{"lemma":"THE","status":"approved","part_of_speech":"article","forms":["THE"],"senses":[],"alternatives":[],"restrictions":[]}},
            {{"lemma":"BE","status":"approved","part_of_speech":"verb","forms":["IS"],"senses":[],"alternatives":[],"restrictions":[]}},
            {{"lemma":"OPEN","status":"approved","part_of_speech":"verb","forms":["OPENING"],"senses":[],"alternatives":[],"restrictions":[]}},
            {{"lemma":"OPENING","status":"approved","part_of_speech":"noun","forms":["OPENING"],"senses":[],"alternatives":[],"restrictions":[]}},
            {{"lemma":"CLEAR","status":"approved","part_of_speech":"adjective","forms":["CLEAR"],"senses":[],"alternatives":[],"restrictions":[]}}{adjective}
          ]
        }}"#
    ))
    .unwrap()
}

fn passive_lexicon(include_adjective: bool) -> RuntimeLexicon {
    let adjective = if include_adjective {
        r#",{"lemma":"CONNECTED","status":"approved","part_of_speech":"adjective","forms":["CONNECTED"],"senses":[],"alternatives":[],"restrictions":[]}"#
    } else {
        ""
    };
    RuntimeLexicon::from_json(&format!(
        r#"{{
          "metadata":{{"standard":"ASD-STE100","issue":9,"date":"2025-01-15","scope":"synthetic_grammar_rule_promotion"}},
          "entries":[
            {{"lemma":"THE","status":"approved","part_of_speech":"article","forms":["THE"],"senses":[],"alternatives":[],"restrictions":[]}},
            {{"lemma":"UNIT","status":"approved","part_of_speech":"noun","forms":["UNIT"],"senses":[],"alternatives":[],"restrictions":[]}},
            {{"lemma":"BE","status":"approved","part_of_speech":"verb","forms":["IS"],"senses":[],"alternatives":[],"restrictions":[]}},
            {{"lemma":"CONNECT","status":"approved","part_of_speech":"verb","forms":["CONNECT","CONNECTS","CONNECTED"],"verb_paradigm":{{"classification":"lexical","source_sequence":["CONNECT","CONNECTS","CONNECTED","CONNECTED"],"base_form":"CONNECT","simple_present_variants":["CONNECTS"],"simple_past_variants":["CONNECTED"],"past_participle":"CONNECTED"}},"senses":[],"alternatives":[],"restrictions":[]}}{adjective}
          ]
        }}"#
    ))
    .unwrap()
}
