use ste_data::RuntimeLexicon;
use ste_lint::{LintMode, LintOptions, LintResult, lint_text};

fn lint(text: &str, lexicon: &RuntimeLexicon, mode: LintMode) -> LintResult {
    lint_text(text, lexicon, None, LintOptions { mode, fix: false })
}

#[test]
fn unambiguous_perfect_participle_also_cites_rule_3_3() {
    let lexicon = lexicon(false);
    let result = lint("THE UNIT HAS CONNECTED.", &lexicon, LintMode::Descriptive);

    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "STE-VERB-001")
        .expect("unambiguous perfect participle should already be reported");
    assert_eq!(diagnostic.rules, vec!["3.2", "3.3", "3.4"]);
}

#[test]
fn ambiguous_perfect_participle_does_not_claim_rule_3_3() {
    let lexicon = lexicon(true);
    let result = lint("THE UNIT HAS CONNECTED.", &lexicon, LintMode::Descriptive);

    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "STE-VERB-002")
        .expect("competing adjective identity should block the perfect construction");
    assert!(!diagnostic.rules.iter().any(|rule| rule == "3.3"));
}

#[test]
fn resolved_procedural_passive_also_cites_rule_3_3() {
    let lexicon = lexicon(false);
    let result = lint("THE UNIT IS CONNECTED.", &lexicon, LintMode::Procedural);

    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "STE-GRAM-003")
        .expect("resolved procedural passive should already be reported");
    assert_eq!(diagnostic.rules, vec!["3.3", "3.6"]);
}

#[test]
fn passive_adjective_ambiguity_does_not_claim_rule_3_3() {
    let lexicon = lexicon(true);
    let result = lint("THE UNIT IS CONNECTED.", &lexicon, LintMode::Procedural);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "STE-GRAM-003")
    );
}

fn lexicon(include_adjective: bool) -> RuntimeLexicon {
    let adjective = if include_adjective {
        r#",{"lemma":"CONNECTED","status":"approved","part_of_speech":"adjective","forms":["CONNECTED"],"senses":[],"alternatives":[],"restrictions":[]}"#
    } else {
        ""
    };
    RuntimeLexicon::from_json(&format!(
        r#"{{
          "metadata":{{"standard":"ASD-STE100","issue":9,"date":"2025-01-15","scope":"synthetic_participle_rule_promotion"}},
          "entries":[
            {{"lemma":"THE","status":"approved","part_of_speech":"article","forms":["THE"],"senses":[],"alternatives":[],"restrictions":[]}},
            {{"lemma":"UNIT","status":"approved","part_of_speech":"noun","forms":["UNIT"],"senses":[],"alternatives":[],"restrictions":[]}},
            {{"lemma":"HAVE","status":"approved","part_of_speech":"verb","forms":["HAS"],"senses":[],"alternatives":[],"restrictions":[]}},
            {{"lemma":"BE","status":"approved","part_of_speech":"verb","forms":["IS"],"senses":[],"alternatives":[],"restrictions":[]}},
            {{"lemma":"CONNECT","status":"approved","part_of_speech":"verb","forms":["CONNECT","CONNECTS","CONNECTED"],"verb_paradigm":{{"classification":"lexical","source_sequence":["CONNECT","CONNECTS","CONNECTED","CONNECTED"],"base_form":"CONNECT","simple_present_variants":["CONNECTS"],"simple_past_variants":["CONNECTED"],"past_participle":"CONNECTED"}},"senses":[],"alternatives":[],"restrictions":[]}}{adjective}
          ]
        }}"#
    ))
    .unwrap()
}
