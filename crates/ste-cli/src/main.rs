mod coverage;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use coverage::{CoverageStatus, RuleCoverageManifest};
use serde::Serialize;
use ste_core::{Diagnostic, Severity};
use ste_data::{LexiconEntry, RuntimeLexicon};
use ste_glossary::Glossary;
use ste_lint::{LintContext, LintMode, LintOptions, LintResult, lint_text_with_context};
use ste_rewrite_check::{ProposedChange, RewriteCheckResult, check_rewrite};

#[derive(Debug, Parser)]
#[command(
    name = "ste",
    about = "Lint technical English with structured STE diagnostics"
)]
struct Cli {
    #[arg(long, global = true, value_name = "PATH")]
    lexicon: Option<PathBuf>,
    #[arg(long, global = true)]
    allow_test_lexicon: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Lint {
        path: PathBuf,
        #[arg(long)]
        fix: bool,
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
        #[arg(long, value_enum, default_value_t = ModeArg::Descriptive)]
        mode: ModeArg,
    },
    CheckRewrite {
        before: PathBuf,
        after: PathBuf,
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
    },
    Dictionary {
        #[command(subcommand)]
        command: DictionaryCommands,
    },
    Glossary {
        #[command(subcommand)]
        command: GlossaryCommands,
    },
    Coverage {
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
    },
    Version,
}

#[derive(Debug, Subcommand)]
enum DictionaryCommands {
    Lookup {
        word: String,
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
    },
}

#[derive(Debug, Subcommand)]
enum GlossaryCommands {
    Check {
        path: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum ModeArg {
    Procedural,
    Descriptive,
}

impl From<ModeArg> for LintMode {
    fn from(value: ModeArg) -> Self {
        match value {
            ModeArg::Procedural => Self::Procedural,
            ModeArg::Descriptive => Self::Descriptive,
        }
    }
}

struct AppFailure {
    exit_code: u8,
    message: String,
}

impl AppFailure {
    fn invalid_data(message: impl Into<String>) -> Self {
        Self {
            exit_code: 3,
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            exit_code: 4,
            message: message.into(),
        }
    }
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(code) => ExitCode::from(code),
        Err(failure) => {
            eprintln!("{}", failure.message);
            ExitCode::from(failure.exit_code)
        }
    }
}

fn run(cli: Cli) -> Result<u8, AppFailure> {
    let Cli {
        lexicon,
        allow_test_lexicon,
        command,
    } = cli;
    match command {
        Commands::Lint {
            path,
            fix,
            format,
            mode,
        } => run_lint(
            &path,
            fix,
            format,
            mode.into(),
            lexicon.as_deref(),
            allow_test_lexicon,
        ),
        Commands::CheckRewrite {
            before,
            after,
            format,
        } => run_check_rewrite(&before, &after, format),
        Commands::Dictionary { command } => {
            run_dictionary(command, lexicon.as_deref(), allow_test_lexicon)
        }
        Commands::Glossary { command } => run_glossary(command),
        Commands::Coverage { format } => run_coverage(format),
        Commands::Version => run_version(lexicon.as_deref()),
    }
}

fn run_lint(
    path: &Path,
    fix: bool,
    format: OutputFormat,
    mode: LintMode,
    lexicon_path: Option<&Path>,
    allow_test_lexicon: bool,
) -> Result<u8, AppFailure> {
    let (lexicon, _) = runtime_lexicon(lexicon_path, allow_test_lexicon)?;
    let original = read_text(path)?;
    let glossary = find_project_glossary(path)?;
    let context = find_project_context(path)?;
    let result = lint_text_with_context(
        &original,
        &lexicon,
        glossary.as_ref(),
        context.as_ref(),
        LintOptions { mode, fix },
    );

    if fix && result.text != original {
        fs::write(path, &result.text).map_err(|error| {
            AppFailure::internal(format!("could not write {}: {error}", path.display()))
        })?;
    }

    print_lint_result(&result, format)?;
    Ok(exit_code_for_diagnostics(&result.diagnostics))
}

fn run_check_rewrite(before: &Path, after: &Path, format: OutputFormat) -> Result<u8, AppFailure> {
    let result = check_rewrite(&ProposedChange {
        original: read_text(before)?,
        proposed: read_text(after)?,
        target_diagnostics: Vec::new(),
    });
    print_rewrite_result(&result, format)?;
    Ok(if result.accepted { 0 } else { 1 })
}

fn run_dictionary(
    command: DictionaryCommands,
    lexicon_path: Option<&Path>,
    allow_test_lexicon: bool,
) -> Result<u8, AppFailure> {
    let (lexicon, _) = runtime_lexicon(lexicon_path, allow_test_lexicon)?;
    match command {
        DictionaryCommands::Lookup { word, format } => {
            let mut entries = lexicon.lookup_form_candidates(&word);
            if entries.is_empty() {
                entries = lexicon.lookup_lemma(&word);
            }

            match format {
                OutputFormat::Json => print_json(&entries)?,
                OutputFormat::Human => {
                    if entries.is_empty() {
                        println!("no matches");
                    } else {
                        for (index, entry) in entries.iter().enumerate() {
                            if index > 0 {
                                println!();
                            }
                            print_human_dictionary_entry(entry);
                        }
                    }
                }
            }
            Ok(0)
        }
    }
}

fn print_human_dictionary_entry(entry: &LexiconEntry) {
    let status = serialized_label(&entry.status);
    let part_of_speech = entry
        .part_of_speech
        .as_ref()
        .map(serialized_label)
        .unwrap_or_else(|| "expression".to_string());
    println!("{} — {} {}", entry.lemma, status, part_of_speech);

    if !entry.forms.is_empty() {
        println!("forms: {}", entry.forms.join(", "));
    }
    if let Some(paradigm) = &entry.verb_paradigm {
        println!("verb class: {}", serialized_label(&paradigm.classification));
        println!(
            "source form sequence: {}",
            paradigm.source_sequence.join(", ")
        );
    }
    for sense in &entry.senses {
        println!("meaning: {}", sense.meaning);
    }
    for alternative in &entry.alternatives {
        println!(
            "alternative: {} [{}; strategy: {}]",
            alternative.text,
            serialized_label(&alternative.kind),
            serialized_label(&alternative.strategy)
        );
    }
    for restriction in &entry.restrictions {
        println!("restriction: {restriction}");
    }

    if let Some(source) = &entry.source_semantics {
        if entry.senses.is_empty() && !source.meaning_or_alternatives.trim().is_empty() {
            println!(
                "source meaning/help: {}",
                source.meaning_or_alternatives.trim()
            );
        }
        if !source.ste_example.trim().is_empty() {
            println!("STE example: {}", source.ste_example.trim());
        }
        if !source.non_ste_example.trim().is_empty() {
            println!("non-STE example: {}", source.non_ste_example.trim());
        }
    }
    if let Some(provenance) = &entry.provenance {
        let pages = provenance
            .source_pages
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        println!("source pages: {pages}");
    }
    println!(
        "interpretation: {}",
        serialized_label(&entry.interpretation_state)
    );
}

fn serialized_label<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value)
        .unwrap_or_else(|_| "\"unknown\"".to_string())
        .trim_matches('"')
        .to_string()
}

fn run_glossary(command: GlossaryCommands) -> Result<u8, AppFailure> {
    match command {
        GlossaryCommands::Check { path, format } => {
            let path = path.unwrap_or_else(|| PathBuf::from(".ste/terms.json"));
            let glossary = parse_glossary(&path)?;
            let diagnostics = glossary.validate();
            print_diagnostics(&diagnostics, format)?;
            Ok(exit_code_for_diagnostics(&diagnostics))
        }
    }
}

fn run_coverage(format: OutputFormat) -> Result<u8, AppFailure> {
    let manifest = RuleCoverageManifest::embedded().map_err(AppFailure::internal)?;
    match format {
        OutputFormat::Json => print_json(&manifest)?,
        OutputFormat::Human => {
            let counts = manifest.status_counts();
            println!("{} Issue {}", manifest.standard, manifest.issue);
            println!("{} rules tracked", manifest.total_rules);
            for status in [
                CoverageStatus::Implemented,
                CoverageStatus::Partial,
                CoverageStatus::ContextRequired,
                CoverageStatus::NotImplemented,
            ] {
                println!(
                    "{}: {}",
                    status.as_str(),
                    counts.get(&status).copied().unwrap_or(0)
                );
            }
            println!("full Issue 9 compliance is not claimed");
            println!("use --format json for per-rule status and diagnostic mappings");
        }
    }
    Ok(0)
}

fn run_version(lexicon_path: Option<&Path>) -> Result<u8, AppFailure> {
    let (lexicon, source) = runtime_lexicon(lexicon_path, true)?;
    println!("ste {}", env!("CARGO_PKG_VERSION"));
    println!(
        "language: {} Issue {}",
        lexicon.metadata().standard,
        lexicon.metadata().issue
    );
    println!("runtime data: {}", lexicon.metadata().scope);
    println!("runtime source: {source}");
    Ok(0)
}

fn runtime_lexicon(
    explicit_path: Option<&Path>,
    allow_test_lexicon: bool,
) -> Result<(RuntimeLexicon, String), AppFailure> {
    let configured_path = explicit_path
        .map(Path::to_path_buf)
        .or_else(|| std::env::var_os("STE_LINT_LEXICON").map(PathBuf::from));

    if let Some(path) = configured_path {
        let bytes = fs::read(&path).map_err(|error| {
            AppFailure::invalid_data(format!(
                "configured runtime lexicon {} could not be read: {error}",
                path.display()
            ))
        })?;
        let lexicon = RuntimeLexicon::verified_issue9_from_bytes(&bytes).map_err(|error| {
            AppFailure::invalid_data(format!(
                "configured runtime lexicon {} failed verification: {error}",
                path.display()
            ))
        })?;
        return Ok((
            lexicon,
            format!("verified external Issue 9 lexicon ({})", path.display()),
        ));
    }

    if !allow_test_lexicon {
        return Err(AppFailure::invalid_data(
            "no verified runtime lexicon is configured; use --lexicon <PATH> or STE_LINT_LEXICON. For development and public fixtures only, pass --allow-test-lexicon.",
        ));
    }

    RuntimeLexicon::embedded()
        .map(|lexicon| (lexicon, "embedded test lexicon".to_string()))
        .map_err(|error| {
            AppFailure::invalid_data(format!("embedded runtime lexicon is invalid: {error}"))
        })
}

fn read_text(path: &Path) -> Result<String, AppFailure> {
    fs::read_to_string(path).map_err(|error| {
        AppFailure::internal(format!("could not read {}: {error}", path.display()))
    })
}

fn parse_glossary(path: &Path) -> Result<Glossary, AppFailure> {
    let text = read_text(path)?;
    Glossary::from_json(&text).map_err(|error| {
        AppFailure::invalid_data(format!("invalid glossary {}: {error}", path.display()))
    })
}

fn parse_context(path: &Path) -> Result<LintContext, AppFailure> {
    let text = read_text(path)?;
    LintContext::from_json(&text).map_err(|error| {
        AppFailure::invalid_data(format!("invalid lint context {}: {error}", path.display()))
    })
}

fn project_file(path: &Path, relative: &str) -> Option<PathBuf> {
    let start = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or_else(|| Path::new("."))
    };

    start
        .ancestors()
        .map(|ancestor| ancestor.join(relative))
        .find(|candidate| candidate.is_file())
}

fn find_project_glossary(path: &Path) -> Result<Option<Glossary>, AppFailure> {
    let Some(candidate) = project_file(path, ".ste/terms.json") else {
        return Ok(None);
    };

    let glossary = parse_glossary(&candidate)?;
    let diagnostics = glossary.validate();
    if !diagnostics.is_empty() {
        let codes = diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(AppFailure::invalid_data(format!(
            "project glossary {} failed validation: {codes}",
            candidate.display()
        )));
    }
    Ok(Some(glossary))
}

fn find_project_context(path: &Path) -> Result<Option<LintContext>, AppFailure> {
    let Some(candidate) = project_file(path, ".ste/context.json") else {
        return Ok(None);
    };
    parse_context(&candidate).map(Some)
}

fn print_lint_result(result: &LintResult, format: OutputFormat) -> Result<(), AppFailure> {
    match format {
        OutputFormat::Json => print_json(result),
        OutputFormat::Human => {
            print_human_diagnostics(&result.diagnostics);
            Ok(())
        }
    }
}

fn print_rewrite_result(
    result: &RewriteCheckResult,
    format: OutputFormat,
) -> Result<(), AppFailure> {
    match format {
        OutputFormat::Json => print_json(result),
        OutputFormat::Human => {
            if result.accepted {
                println!("accepted");
            } else {
                print_human_diagnostics(&result.diagnostics);
            }
            Ok(())
        }
    }
}

fn print_diagnostics(diagnostics: &[Diagnostic], format: OutputFormat) -> Result<(), AppFailure> {
    match format {
        OutputFormat::Json => print_json(diagnostics),
        OutputFormat::Human => {
            print_human_diagnostics(diagnostics);
            Ok(())
        }
    }
}

fn print_human_diagnostics(diagnostics: &[Diagnostic]) {
    if diagnostics.is_empty() {
        println!("clean");
        return;
    }

    for diagnostic in diagnostics {
        println!(
            "{} {}:{} {}",
            diagnostic.code, diagnostic.span.start, diagnostic.span.end, diagnostic.message
        );
    }
}

fn print_json<T: Serialize + ?Sized>(value: &T) -> Result<(), AppFailure> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|error| AppFailure::internal(format!("could not serialize output: {error}")))?;
    println!("{json}");
    Ok(())
}

fn exit_code_for_diagnostics(diagnostics: &[Diagnostic]) -> u8 {
    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
    {
        1
    } else if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Blocked)
    {
        2
    } else {
        0
    }
}
