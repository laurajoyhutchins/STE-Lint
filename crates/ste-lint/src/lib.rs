mod passes;

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
    _lexicon: &RuntimeLexicon,
    _glossary: Option<&Glossary>,
    options: LintOptions,
) -> LintResult {
    let initial = passes::punctuation::check(text);
    let mut output = text.to_owned();
    let mut fixed_any = false;

    if options.fix {
        let mut fixes = initial
            .iter()
            .filter_map(|diagnostic| diagnostic.autofix.clone())
            .collect::<Vec<_>>();
        fixes.sort_by(|left, right| right.span.start.cmp(&left.span.start));
        for fix in fixes {
            output.replace_range(fix.span.start..fix.span.end, &fix.replacement);
            fixed_any = true;
        }
    }

    let diagnostics = if options.fix {
        passes::punctuation::check(&output)
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

    #[test]
    fn semicolon_is_reported_and_can_be_safely_fixed() {
        let lexicon = RuntimeLexicon::embedded().unwrap();
        let options = LintOptions {
            mode: LintMode::Procedural,
            fix: false,
        };
        let result = lint_text("USE THIS; USE THIS.", &lexicon, None, options);
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "STE-PUNC-001")
        );

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
        assert!(
            !fixed
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "STE-PUNC-001")
        );
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
}
