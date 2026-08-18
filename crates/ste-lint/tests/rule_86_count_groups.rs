use ste_data::RuntimeLexicon;
use ste_glossary::Glossary;
use ste_lint::{LintContext, LintMode, LintOptions, lint_text_with_context};

fn entity_context() -> LintContext {
    LintContext::from_json(
        r#"{
          "named_entities":[{
            "id":"north-atlantic-treaty-organization",
            "canonical":"North Atlantic Treaty Organization",
            "class":"organization",
            "forms":["NATO"],
            "source":"independent named-entity fixture authority"
          }]
        }"#,
    )
    .unwrap()
}

fn glossary_with_multiword_abbreviation() -> Glossary {
    Glossary::from_json(
        r#"{
          "schema":"ste-terminology/v2",
          "domain":"fixture",
          "sources":{"fixture":{"title":"Independent abbreviation fixture authority"}},
          "terms":[{
            "id":"control-display-unit",
            "canonical":"control display unit",
            "roles":["noun"],
            "definition":"A synthetic technical noun used only by this test.",
            "forms":[],
            "aliases":[{"text":"CD U","kind":"abbreviation"}],
            "sources":[{
              "source":"fixture",
              "supports":["admission","definition","role","alias","status"]
            }],
            "status":"approved"
          }]
        }"#,
    )
    .unwrap()
}

fn lint(
    text: &str,
    mode: LintMode,
    glossary: Option<&Glossary>,
    context: Option<&LintContext>,
) -> ste_lint::LintResult {
    let lexicon = RuntimeLexicon::embedded().unwrap();
    lint_text_with_context(
        text,
        &lexicon,
        glossary,
        context,
        LintOptions { mode, fix: false },
    )
}

fn has_length_error(result: &ste_lint::LintResult) -> bool {
    result
        .diagnostics
        .iter()
        .any(|diagnostic| matches!(diagnostic.code.as_str(), "STE-LEN-001" | "STE-LEN-002"))
}

fn uses(count: usize) -> String {
    vec!["USE"; count].join(" ")
}

#[test]
fn rule_86_governed_multiword_proper_noun_counts_as_one_in_procedure() {
    let context = entity_context();
    let text = format!("{} North Atlantic Treaty Organization.", uses(19));
    let result = lint(&text, LintMode::Procedural, None, Some(&context));
    assert!(!has_length_error(&result));
}

#[test]
fn rule_86_governed_multiword_proper_noun_counts_as_one_in_description() {
    let context = entity_context();
    let text = format!("{} North Atlantic Treaty Organization.", uses(24));
    let result = lint(&text, LintMode::Descriptive, None, Some(&context));
    assert!(!has_length_error(&result));
}

#[test]
fn rule_86_named_entity_authority_survives_document_edits() {
    let context = entity_context();
    for text in [
        format!("{} North Atlantic Treaty Organization.", uses(19)),
        format!("USE.\n\n{} North Atlantic Treaty Organization.", uses(19)),
    ] {
        let result = lint(&text, LintMode::Procedural, None, Some(&context));
        assert!(!has_length_error(&result), "{text}");
    }
}

#[test]
fn rule_86_does_not_infer_uppercase_proper_noun_without_authority() {
    let text = format!("{} NORTH ATLANTIC TREATY ORGANIZATION.", uses(19));
    let result = lint(&text, LintMode::Procedural, None, None);
    assert!(has_length_error(&result));
}

#[test]
fn rule_86_governed_multiword_abbreviation_counts_as_one() {
    let glossary = glossary_with_multiword_abbreviation();
    let text = format!("{} CD U.", uses(19));
    let result = lint(&text, LintMode::Procedural, Some(&glossary), None);
    assert!(!has_length_error(&result));
}

#[test]
fn rule_86_signed_number_with_temperature_unit_counts_as_one() {
    let text = format!("{} -10 °C.", uses(19));
    let result = lint(&text, LintMode::Procedural, None, None);
    assert!(!has_length_error(&result));
}

#[test]
fn rule_86_fraction_with_unit_counts_as_one() {
    let text = format!("{} 1/2 in.", uses(19));
    let result = lint(&text, LintMode::Procedural, None, None);
    assert!(!has_length_error(&result));
}

#[test]
fn rule_86_range_with_unit_counts_as_one() {
    let text = format!("{} 10–12 kg.", uses(19));
    let result = lint(&text, LintMode::Procedural, None, None);
    assert!(!has_length_error(&result));
}

#[test]
fn rule_86_compound_unit_expression_counts_with_number_as_one() {
    let text = format!("{} 10 kg/m³.", uses(19));
    let result = lint(&text, LintMode::Procedural, None, None);
    assert!(!has_length_error(&result));
}

#[test]
fn rule_86_spaced_compound_unit_expression_counts_with_number_as_one() {
    let text = format!("{} 10 N m.", uses(19));
    let result = lint(&text, LintMode::Procedural, None, None);
    assert!(!has_length_error(&result));
}

#[test]
fn rule_86_quoted_text_counts_as_one() {
    let text = format!("{} \"CONTROL PANEL READY\".", uses(19));
    let result = lint(&text, LintMode::Procedural, None, None);
    assert!(!has_length_error(&result));
}

#[test]
fn rule_86_formula_authority_counts_the_formula_as_quoted_text() {
    let formula = "C = (A - B) - 0.063 mm";
    let text = format!("{} {formula}.", uses(19));
    let start = text.find(formula).unwrap();
    let end = start + formula.len();
    let context = LintContext::from_json(&format!(
        r#"{{"occurrences":[{{"start":{start},"end":{end},"source":"formula node","text_authority":"formula"}}]}}"#
    ))
    .unwrap();
    let result = lint(&text, LintMode::Procedural, None, Some(&context));
    assert!(!has_length_error(&result));
}

#[test]
fn rule_86_markdown_atx_heading_is_one_structural_count_group() {
    let text = format!("# {}", uses(25));
    let result = lint(&text, LintMode::Procedural, None, None);
    assert!(!has_length_error(&result));
}

#[test]
fn rule_86_markdown_setext_heading_is_one_structural_count_group() {
    let text = format!("{}\n===", uses(25));
    let result = lint(&text, LintMode::Procedural, None, None);
    assert!(!has_length_error(&result));
}

#[test]
fn rule_86_explicit_title_placard_and_label_groups_remain_supported() {
    let group = "ALPHA BETA GAMMA DELTA";
    for kind in ["title", "placard", "label"] {
        let text = format!("{} {group}.", uses(19));
        let start = text.find(group).unwrap();
        let end = start + group.len();
        let context = LintContext::from_json(&format!(
            r#"{{"occurrences":[{{"start":{start},"end":{end},"source":"document structure fixture","count_group":"{kind}"}}]}}"#
        ))
        .unwrap();
        let result = lint(&text, LintMode::Procedural, None, Some(&context));
        assert!(!has_length_error(&result), "count-group kind {kind}");
    }
}

#[test]
fn rule_86_protected_external_text_is_shared_with_punctuation_scope() {
    let protected = "MODE; SAFE CONFIGURATION";
    let text = format!("{} \"{protected}\".", uses(19));
    let start = text.find('"').unwrap();
    let end = text.rfind('"').unwrap() + 1;
    let context = LintContext::from_json(&format!(
        r#"{{"occurrences":[{{"start":{start},"end":{end},"source":"immutable UI contract","text_authority":"quoted_external_text"}}]}}"#
    ))
    .unwrap();
    let result = lint(&text, LintMode::Procedural, None, Some(&context));
    assert!(!has_length_error(&result));
    assert!(!result.diagnostics.iter().any(|d| d.code == "STE-PUNC-001"));
}

#[test]
fn rule_86_document_numbering_can_be_explicitly_excluded() {
    let numbering = "12.4.3";
    let text = format!("{} {numbering}.", uses(20));
    let start = text.find(numbering).unwrap();
    let end = start + numbering.len();
    let context = LintContext::from_json(&format!(
        r#"{{"occurrences":[{{"start":{start},"end":{end},"source":"document numbering node","text_authority":"document_numbering"}}]}}"#
    ))
    .unwrap();
    let result = lint(&text, LintMode::Procedural, None, Some(&context));
    assert!(!has_length_error(&result));
}

#[test]
fn rule_84_list_boundary_and_rule_86_entity_group_use_same_counter() {
    let context = entity_context();
    let text = format!(
        "DO THIS:\n- {} North Atlantic Treaty Organization.",
        uses(19)
    );
    let result = lint(&text, LintMode::Procedural, None, Some(&context));
    assert!(!has_length_error(&result));
}

#[test]
fn rules_85_86_and_87_share_count_semantics_inside_parenthetical() {
    let context = entity_context();
    let text = format!(
        "USE THIS ({} North Atlantic Treaty Organization high-pressure).",
        uses(18)
    );
    let result = lint(&text, LintMode::Procedural, None, Some(&context));
    assert!(!has_length_error(&result));
}
