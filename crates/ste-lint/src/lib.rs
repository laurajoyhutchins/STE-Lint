mod analysis;
mod context;
mod document_structure;
mod passes;
mod structure;

use std::cmp::Reverse;

pub use analysis::{
    ActionCardinality, ActionStructure, AnalysisDocument, AnalysisEvidence, AnalysisSentence,
    AnalysisToken, AuxiliaryChain, AuxiliaryKind, CanonicalSpan, CountGroup, CountGroupProjection,
    DictionaryMatch, DocumentGraph, DocumentNode, DocumentNodeId, DocumentNodeKind,
    DocumentReferenceRelation, DocumentRelation, DocumentRelationKind, DocumentSemanticOrdering,
    DocumentSpan, EntityIdentity, EntityMention, EntityMentionKind, EvidenceAlternative,
    EvidenceProvenance, EvidenceTarget, GlossaryMatch, GrammarSpan, IngRole, IngUse,
    LexicalObservation, ModelIdentity, NounPhrase, ObservedRole, ObservedRoleEvidence,
    ParticipleRole, ParticipleUse, ProviderIdentity, ReferenceBasis, ReferenceLink, Resolution,
    SafetyEvidenceSource, SafetyLevel, SafetyLevelEvidence, SafetySemantics, SafetySpanEvidence,
    SemanticObservation, SenseEvidence, SenseIdentity, SenseProvenance, SenseRestrictionTag,
    ShadowEvidenceError, ShadowEvidenceIdentity, ShadowEvidenceSet, SubjectPredicate,
    VerbFormCandidate, VerbFormRole,
};
pub use context::{
    CountGroupKind, DictionaryMeaningUse, LintContext, MeasurementUnitFact, NamedEntityClass,
    NamedEntityFact, OccurrenceFact, ParenthesisUseKind, SafetyFact, SafetyLevelFact,
    SafetySpanFact, SemanticOrderTarget, SemanticOrderTargetKind, SemanticOrderingFact,
    SpellingUse, TechnicalNounScope, TextAuthorityKind, TopicFact,
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
    let mut diagnostics = passes::punctuation::check(&analysis);
    diagnostics.extend(passes::contractions::check(text));
    diagnostics.extend(passes::length::check(&analysis));
    diagnostics.extend(passes::lists::check(&analysis));
    diagnostics.extend(passes::notes::check(text, lexicon, mode));
    diagnostics.extend(passes::paragraph::check(text, mode, context));
    diagnostics.extend(passes::verb_constructions::check(&analysis));
    diagnostics.extend(passes::grammar_semantics::check(&analysis));
    diagnostics.extend(passes::entity_semantics::check(&analysis));
    diagnostics.extend(passes::discourse_semantics::check(&analysis));
    diagnostics.extend(passes::technical_roles::check(&analysis));
    diagnostics.extend(passes::dictionary_forms::check(&analysis));
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