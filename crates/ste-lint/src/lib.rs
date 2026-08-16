mod analysis;
mod context;
mod document_structure;
mod passes;
mod structure;

use std::cmp::Reverse;

pub use analysis::{
    ActionCardinality, ActionStructure, AnalysisDocument, AnalysisSentence, AnalysisToken,
    AuxiliaryChain, AuxiliaryKind, DictionaryMatch, DocumentGraph, DocumentNode, DocumentNodeId,
    DocumentNodeKind, DocumentReferenceRelation, DocumentRelation, DocumentRelationKind,
    DocumentSemanticOrdering, DocumentSpan, EntityIdentity, EntityMention, EntityMentionKind,
    GlossaryMatch, GrammarSpan, IngRole, IngUse, NounPhrase, ObservedRole, ObservedRoleEvidence,
    ParticipleRole, ParticipleUse, ReferenceBasis, ReferenceLink, Resolution, SafetyEvidenceSource,
    SafetyLevel, SafetyLevelEvidence, SafetySemantics, SafetySpanEvidence, SenseEvidence,
    SenseIdentity, SenseProvenance, SenseRestrictionTag, SubjectPredicate, VerbFormCandidate,
    VerbFormRole,
};
pub use context::{
    CountGroupKind, DictionaryMeaningUse, LintContext, OccurrenceFact, ParenthesisUseKind,
    SafetyFact, SafetyLevelFact, SafetySpanFact, SemanticOrderTarget, SemanticOrderTargetKind,
    SemanticOrderingFact, SpellingUse, TechnicalNounScope, TopicFact,
};
use serde::{Deserialize, Serialize};
use ste_core::{Diagnostic, Outcome, Severity};
use ste_data::RuntimeLexicon;
use ste_glossary::Glossary;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LintMode {
    Procedural,
    Descriptive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LintOptions {
    pub mode: LintMode,
    pub fix: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LintResult {
    pub text: String,
    pub diagnostics: Vec<Diagnostic>,
    pub outcome: Outcome,
}

pub fn lint_text(
    text: &str,
    lexicon: &RuntimeLexicon,
    glossary: Option<&Glossary>,
    options: LintOptions,
) -> LintResult {
    lint_text_with_context(text, lexicon, glossary, None, options)
}

pub fn lint_text_with_context(
    text: &str,
    lexicon: &RuntimeLexicon,
    glossary: Option<&Glossary>,
    context: Option<&LintContext>,
    options: LintOptions,
) -> LintResult {
    let initial = collect_diagnostics(text, lexicon, glossary, context, options.mode);
    let mut output = text.to_owned();
    let mut fixed_any = false;

    if options.fix {
        let mut fixes = initial
            .iter()
            .filter_map(|diagnostic| diagnostic.autofix.clone())
            .collect::<Vec<_>>();
        fixes.sort_by_key(|fix| Reverse(fix.span.start));
        for fix in fixes {
            output.replace_range(fix.span.start..fix.span.end, &fix.replacement);
            fixed_any = true;
        }
    }

    let diagnostics = if options.fix && fixed_any {
        collect_diagnostics(&output, lexicon, glossary, context, options.mode)
    } else {
        initial
    };
    let outcome = classify_outcome(&diagnostics, fixed_any);

    LintResult {
        text: output,
        diagnostics,
        outcome,
    }
}

fn collect_diagnostics(
    text: &str,
    lexicon: &RuntimeLexicon,
    glossary: Option<&Glossary>,
    context: Option<&LintContext>,
    mode: LintMode,
) -> Vec<Diagnostic> {
    let analysis = AnalysisDocument::new(text, lexicon, glossary, context, mode);
    let mut diagnostics = passes::punctuation::check(text);
    diagnostics.extend(passes::contractions::check(text));
    diagnostics.extend(passes::length::check(text, mode, context));
    diagnostics.extend(passes::lists::check(text));
    diagnostics.extend(passes::notes::check(text, lexicon, mode));
    diagnostics.extend(passes::paragraph::check(text, mode, context));
    diagnostics.extend(passes::perfect::check(&analysis));
    diagnostics.extend(passes::technical_roles::check(&analysis));
    diagnostics.extend(passes::dictionary_roles::check(&analysis));
    diagnostics.extend(passes::procedural::check(&analysis));
    diagnostics.extend(passes::contextual::check(text, context));
    diagnostics.extend(passes::lexical::check(text, lexicon, glossary));
    diagnostics.sort_by_key(|diagnostic| (diagnostic.span.start, diagnostic.code.clone()));
    diagnostics
}

fn classify_outcome(diagnostics: &[Diagnostic], fixed_any: bool) -> Outcome {
    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
    {
        Outcome::Error
    } else if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Blocked)
    {
        Outcome::Blocked
    } else if fixed_any {
        Outcome::Fixed
    } else {
        Outcome::Clean
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ste_data::RuntimeLexicon;
    use ste_glossary::Glossary;

    fn has_code(result: &LintResult, code: &str) -> bool {
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == code)
    }

    #[test]
    fn semicolon_is_reported_and_can_be_safely_fixed() {
        let lexicon = RuntimeLexicon::embedded().unwrap();
        let options = LintOptions {
            mode: LintMode::Procedural,
            fix: false,
        };
        let result = lint_text("USE THIS; USE THIS.", &lexicon, None, options);
        assert!(has_code(&result, "STE-PUNC-001"));

        let fixed = lint_text(
            "USE THIS; USE THIS.",
            &lexicon,
            None,
            LintOptions {
                mode: LintMode::Procedural,
                fix: true,
            },
        );
        assert_eq!(fixed.text, "USE THIS. USE THIS.");
        assert!(!has_code(&fixed, "STE-PUNC-001"));
    }

    #[test]
    fn semicolon_autofix_is_idempotent() {
        let lexicon = RuntimeLexicon::embedded().unwrap();
        let first = lint_text(
            "USE THIS; USE THIS.",
            &lexicon,
            None,
            LintOptions {
                mode: LintMode::Procedural,
                fix: true,
            },
        );
        let second = lint_text(
            &first.text,
            &lexicon,
            None,
            LintOptions {
                mode: LintMode::Procedural,
                fix: true,
            },
        );
        assert_eq!(first.text, second.text);
        assert_eq!(second.outcome, Outcome::Clean);
    }

    #[test]
    fn procedural_sentences_over_twenty_words_are_errors() {
        let lexicon = RuntimeLexicon::embedded().unwrap();
        let text = vec!["USE"; 21].join(" ");
        let result = lint_text(
            &text,
            &lexicon,
            None,
            LintOptions {
                mode: LintMode::Procedural,
                fix: false,
            },
        );
        assert!(has_code(&result, "STE-LEN-001"));
    }

    #[test]
    fn descriptive_sentences_over_twenty_five_words_are_errors() {
        let lexicon = RuntimeLexicon::embedded().unwrap();
        let text = vec!["USE"; 26].join(" ");
        let result = lint_text(
            &text,
            &lexicon,
            None,
            LintOptions {
                mode: LintMode::Descriptive,
                fix: false,
            },
        );
        assert!(has_code(&result, "STE-LEN-002"));
    }

    #[test]
    fn known_unapproved_word_emits_lexical_diagnostic_without_autofix() {
        let lexicon = RuntimeLexicon::embedded().unwrap();
        let result = lint_text(
            "acceptable",
            &lexicon,
            None,
            LintOptions {
                mode: LintMode::Descriptive,
                fix: true,
            },
        );
        assert!(has_code(&result, "STE-LEX-001"));
        assert_eq!(result.text, "acceptable");
    }

    #[test]
    fn project_glossary_resolves_a_technical_term() {
        let lexicon = RuntimeLexicon::embedded().unwrap();
        let glossary =
            Glossary::from_json(include_str!("../../../fixtures/glossary/valid.json")).unwrap();
        let result = lint_text(
            "busway",
            &lexicon,
            Some(&glossary),
            LintOptions {
                mode: LintMode::Descriptive,
                fix: false,
            },
        );
        assert!(!has_code(&result, "STE-TERM-001"));
    }

    #[test]
    fn unknown_prose_word_is_blocked_for_term_classification() {
        let lexicon = RuntimeLexicon::embedded().unwrap();
        let result = lint_text(
            "fluxcapacitor",
            &lexicon,
            None,
            LintOptions {
                mode: LintMode::Descriptive,
                fix: false,
            },
        );
        assert!(has_code(&result, "STE-TERM-001"));
        assert_eq!(result.outcome, Outcome::Blocked);
    }

    #[test]
    fn machine_like_tokens_are_not_treated_as_unknown_prose_terms() {
        let lexicon = RuntimeLexicon::embedded().unwrap();
        let result = lint_text(
            "occurrence_id path/to/file foo-bar 1.2",
            &lexicon,
            None,
            LintOptions {
                mode: LintMode::Descriptive,
                fix: false,
            },
        );
        assert!(!has_code(&result, "STE-TERM-001"));
    }
}
