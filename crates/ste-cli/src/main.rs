use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;
use ste_core::{Diagnostic, Severity};
use ste_data::RuntimeLexicon;
use ste_glossary::Glossary;
use ste_lint::{lint_text, LintMode, LintOptions, LintResult};
use ste_rewrite_check::{check_rewrite, ProposedChange, RewriteCheckResult};

#[derive(Debug, Parser)]
#[command(name = "ste", about = "Lint technical English with structured STE diagnostics")]
struct Cli {
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
    match cli.command {
        Commands::Lint {
            path,
            fix,
            format,
            mode,
        } => run_lint(&path, fix, format, mode.into()),
        Commands::CheckRewrite {
            before,
            after,
            format,
        } => run_check_rewrite(&before, &after, format),
        Commands::Dictionary { command } => run_dictionary(command),
        Commands::Glossary { command } => run_glossary(command),
        Commands::Version => run_version(),
    }
}

fn run_lint(
    path: &Path,
    fix: bool,
    format: OutputFormat,
    mode: LintMode,
) -> Result<u8, AppFailure> {
    let lexicon = runtime_lexicon()?;
    let original = read_text(path)?;
    let glossary = find_project_glossary(path)?;
    let result = lint_text(
        &original,
        &lexicon,
        glossary.as_ref(),
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

fn run_check_rewrite(
    before: &Path,
    after: &Path,
    format: OutputFormat,
) -> Result<u8, AppFailure> {
    let result = check_rewrite(&ProposedChange {
        original: read_text(before)?,
        proposed: read_text(after)?,
        target_diagnostics: Vec::new(),
    });
    print_rewrite_result(&result, format)?;
    Ok(if result.accepted { 0 } else { 1 })
}

fn run_dictionary(command: DictionaryCommands) -> Result<u8, AppFailure> {
    let lexicon = runtime_lexicon()?;
    match command {
        DictionaryCommands::Lookup { word, format } => {
            let entries = if let Some(entry) = lexicon.lookup_form(&word) {
                vec![entry]
            } else {
                lexicon.lookup_lemma(&word)
            };

            match format {
                OutputFormat::Json => print_json(&entries)?,
                OutputFormat::Human => {
                    if entries.is_empty() {
                        println!("no matches");
                    } else {
                        for entry in entries {
                            println!(
                                "{} ({:?}) {:?}",
                                entry.lemma, entry.part_of_speech, entry.status
                            );
                        }
                    }
                }
            }
            Ok(0)
        }
    }
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

fn run_version() -> Result<u8, AppFailure> {
    let lexicon = runtime_lexicon()?;
    println!("ste {}", env!("CARGO_PKG_VERSION"));
    println!(
        "language: {} Issue {}",
        lexicon.metadata().standard,
        lexicon.metadata().issue
    );
    println!("runtime data: {}", lexicon.metadata().scope);
    Ok(0)
}

fn runtime_lexicon() -> Result<RuntimeLexicon, AppFailure> {
    RuntimeLexicon::embedded().map_err(|error| {
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

fn find_project_glossary(path: &Path) -> Result<Option<Glossary>, AppFailure> {
    let start = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or_else(|| Path::new("."))
    };

    for ancestor in start.ancestors() {
        let candidate = ancestor.join(".ste/terms.json");
        if candidate.is_file() {
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
            return Ok(Some(glossary));
        }
    }

    Ok(None)
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

fn print_diagnostics(
    diagnostics: &[Diagnostic],
    format: OutputFormat,
) -> Result<(), AppFailure> {
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
