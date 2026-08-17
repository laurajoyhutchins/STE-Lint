from pathlib import Path

path = Path("crates/ste-glossary/src/lib.rs")
text = path.read_text()
text = text.replace(
'''        if let Some(replacement) = &term.replacement {
            if term.status != TermStatus::Deprecated || !all_ids.contains(replacement.as_str()) {
                diagnostics.push(simple_diagnostic(
                    "TERM-REPLACEMENT-001",
                    "Technical term replacement must name an existing term and may only be set on a deprecated term.",
                    serde_json::json!({"term_id": term.id, "replacement": replacement}),
                ));
            }
        }
''',
'''        if let Some(replacement) = &term.replacement
            && (term.status != TermStatus::Deprecated || !all_ids.contains(replacement.as_str()))
        {
            diagnostics.push(simple_diagnostic(
                "TERM-REPLACEMENT-001",
                "Technical term replacement must name an existing term and may only be set on a deprecated term.",
                serde_json::json!({"term_id": term.id, "replacement": replacement}),
            ));
        }
''')
text = text.replace(
'''enum LegacyTechnicalTermKind {
    TechnicalNoun,
    TechnicalVerb,
    TechnicalNounAndVerb,
}
''',
'''enum LegacyTechnicalTermKind {
    #[serde(rename = "technical_noun")]
    Noun,
    #[serde(rename = "technical_verb")]
    Verb,
    #[serde(rename = "technical_noun_and_verb")]
    NounAndVerb,
}
''')
text = text.replace("LegacyTechnicalTermKind::TechnicalNounAndVerb", "LegacyTechnicalTermKind::NounAndVerb")
text = text.replace("LegacyTechnicalTermKind::TechnicalNoun", "LegacyTechnicalTermKind::Noun")
text = text.replace("LegacyTechnicalTermKind::TechnicalVerb", "LegacyTechnicalTermKind::Verb")
path.write_text(text)
