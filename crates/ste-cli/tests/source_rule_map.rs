use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde_json::Value;

fn rules() -> BTreeMap<String, Value> {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = fs::read_to_string(repo.join("data/rules.json")).unwrap();
    let manifest: Value = serde_json::from_str(&source).unwrap();
    manifest["rules"]
        .as_array()
        .expect("rules array")
        .iter()
        .map(|rule| (rule["id"].as_str().unwrap().to_owned(), rule.clone()))
        .collect()
}

#[test]
fn all_issue_nine_rules_have_source_audited_semantic_keys() {
    let rules = rules();
    let expected = [
        ("1.1", "permitted_word_classes"),
        ("1.2", "approved_part_of_speech"),
        ("1.3", "approved_meaning"),
        ("1.4", "approved_verb_adjective_forms"),
        ("1.5", "technical_noun_categories"),
        ("1.6", "unapproved_word_as_technical_noun"),
        ("1.7", "technical_noun_not_verb"),
        ("1.8", "governed_technical_noun_approval"),
        ("1.9", "short_clear_technical_noun"),
        ("1.10", "no_regional_slang_jargon_technical_nouns"),
        ("1.11", "consistent_technical_noun_for_item"),
        ("1.12", "technical_verb_categories"),
        ("1.13", "technical_verb_not_noun"),
        ("1.14", "american_english_spelling"),
        ("2.1", "multiword_noun_max_three_words"),
        ("2.2", "long_technical_noun_clarification"),
        ("3.1", "dictionary_verb_forms_only"),
        ("3.2", "allowed_verb_forms_and_tenses"),
        ("3.3", "past_participle_as_adjective"),
        ("3.4", "no_complex_auxiliary_constructions"),
        ("3.5", "ing_only_technical_noun_or_modifier"),
        ("3.6", "active_voice_with_unknown_agent_exception"),
        ("3.7", "approved_verb_for_action"),
        ("4.1", "short_clear_sentences"),
        ("4.2", "no_omission_or_contractions"),
        ("4.3", "vertical_list_for_complex_text"),
        ("4.4", "connect_related_sentences"),
        ("4.5", "articles_or_demonstratives_before_nouns"),
        ("5.1", "procedural_sentence_max_20_words"),
        ("5.2", "one_instruction_unless_simultaneous"),
        ("5.3", "imperative_procedural_instructions"),
        ("5.4", "condition_first_then_comma"),
        ("5.5", "notes_information_not_instructions"),
        ("6.1", "gradual_information"),
        ("6.2", "key_words_phrases_logical_structure"),
        ("6.3", "descriptive_sentence_max_25_words"),
        ("6.4", "paragraphs_for_related_information"),
        ("6.5", "one_topic_per_paragraph"),
        ("6.6", "paragraph_max_six_sentences"),
        ("7.1", "risk_level_word_or_symbol"),
        ("7.2", "safety_command_or_condition_first"),
        ("7.3", "explain_risk_or_result"),
        ("8.1", "no_semicolon"),
        ("8.2", "hyphen_directly_related_words"),
        ("8.3", "allowed_parenthesis_uses"),
        ("8.4", "vertical_list_colon_ends_count_unit"),
        ("8.5", "parenthetical_text_counts_one_word"),
        ("8.6", "specified_elements_count_one_word"),
        ("8.7", "hyphenated_words_count_one_word"),
        (
            "9.1",
            "different_sentence_construction_when_replacement_insufficient",
        ),
        ("9.2", "use_approved_words_correctly"),
        ("9.3", "no_phrasal_verbs"),
        ("9.4", "consistent_terminology_and_wording"),
    ];

    assert_eq!(rules.len(), expected.len());
    for (id, semantic_key) in expected {
        assert_eq!(
            rules[id]["semantic_key"].as_str(),
            Some(semantic_key),
            "source semantic key drift for Rule {id}"
        );
    }
}

#[test]
fn audited_coverage_preserves_the_verified_rule_boundary() {
    let rules = rules();

    assert_eq!(rules["1.4"]["status"], "partial");
    assert_eq!(
        rules["1.4"]["diagnostic_codes"],
        serde_json::json!(["STE-FORM-001"])
    );
    assert_eq!(rules["3.1"]["status"], "partial");
    assert_eq!(
        rules["3.1"]["diagnostic_codes"],
        serde_json::json!(["STE-FORM-001"])
    );
    assert_eq!(rules["9.3"]["status"], "partial");
    assert_eq!(
        rules["9.3"]["diagnostic_codes"],
        serde_json::json!(["STE-PHRASE-001"])
    );

    assert_eq!(rules["2.2"]["status"], "implemented");
    assert_eq!(
        rules["2.2"]["diagnostic_codes"],
        serde_json::json!(["STE-NOUN-002"])
    );
    assert_eq!(rules["5.1"]["status"], "implemented");
    assert_eq!(rules["6.3"]["status"], "implemented");
    assert_eq!(rules["8.1"]["status"], "implemented");
    assert_eq!(rules["8.6"]["status"], "implemented");
    assert_eq!(rules["3.3"]["status"], "partial");
    assert_eq!(
        rules["3.3"]["diagnostic_codes"],
        serde_json::json!([
            "STE-VERB-001",
            "STE-VERB-003",
            "STE-VERB-004",
            "STE-GRAM-003"
        ])
    );
    assert_eq!(rules["6.1"]["status"], "partial");
    assert_eq!(
        rules["6.1"]["diagnostic_codes"],
        serde_json::json!(["STE-DISC-001"])
    );
    assert_eq!(rules["9.2"]["status"], "partial");
    assert_eq!(
        rules["9.2"]["diagnostic_codes"],
        serde_json::json!(["STE-CTX-001"])
    );
    assert_eq!(
        rules["9.2"]["evidence_artifacts"],
        serde_json::json!(["crates/ste-lint/tests/context_evidence.rs"])
    );

    assert_eq!(rules["6.6"]["status"], "implemented");
    assert_eq!(rules["8.4"]["status"], "implemented");
    assert_eq!(
        rules["5.5"]["diagnostic_codes"],
        serde_json::json!(["STE-NOTE-001", "STE-NOTE-002"])
    );
    assert!(
        !rules["1.1"]["diagnostic_codes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|code| code == "STE-TERM-002")
    );
    assert!(
        rules["1.8"]["diagnostic_codes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|code| code == "STE-TERM-002")
    );

    let mut counts = BTreeMap::<&str, usize>::new();
    for rule in rules.values() {
        *counts.entry(rule["status"].as_str().unwrap()).or_default() += 1;
    }
    assert_eq!(counts.get("implemented"), Some(&9));
    assert_eq!(counts.get("partial"), Some(&33));
    assert_eq!(counts.get("context_required"), Some(&11));
    assert_eq!(counts.get("not_implemented"), None);
}
