use std::env;
use std::fs;

use serde::Deserialize;
use serde_json::{Value, json};
use ste_data::RuntimeLexicon;
use ste_lint::{AnalysisDocument, EvidenceTarget, LintMode};

#[derive(Debug, Deserialize)]
struct Corpus {
    schema_version: u32,
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
struct Case {
    id: String,
    text: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args().nth(1).ok_or("usage: semantic_eval_harper <cases.json>")?;
    let corpus: Corpus = serde_json::from_str(&fs::read_to_string(path)?)?;
    if corpus.schema_version != 1 {
        return Err(format!("unsupported corpus schema: {}", corpus.schema_version).into());
    }

    let lexicon = RuntimeLexicon::embedded()?;
    let mut cases = Vec::with_capacity(corpus.cases.len());
    for case in corpus.cases {
        let analysis = AnalysisDocument::new(&case.text, &lexicon, None, None, LintMode::Procedural);
        let mut tokens = Vec::new();
        for (index, evidence) in analysis.lexical_evidence().iter().enumerate() {
            let EvidenceTarget::Token(span) = evidence.target else {
                continue;
            };
            let value = &evidence.value;
            let dictionary = analysis.dictionary_match_at(index, 1);
            let possible_parts_of_speech = dictionary
                .as_ref()
                .map(|matched| matched.possible_parts_of_speech.iter().map(|part| format!("{part:?}")).collect::<Vec<_>>())
                .unwrap_or_default();
            let dictionary_verb_roles = dictionary
                .as_ref()
                .map(|matched| matched.verb_forms.iter().map(|form| format!("{:?}", form.role)).collect::<Vec<_>>())
                .unwrap_or_default();

            tokens.push(json!({
                "text": &case.text[span.start..span.end],
                "start": span.start,
                "end": span.end,
                "lemma": value.lemma.as_deref(),
                "determiner": value.determiner,
                "conjunction": value.conjunction,
                "noun": value.noun,
                "nominal": value.nominal,
                "adjective": value.adjective,
                "verb": value.verb,
                "auxiliary_verb": value.auxiliary_verb,
                "linking_verb": value.linking_verb,
                "np_member": value.np_member,
                "comparative_adjective": value.comparative_adjective,
                "superlative_adjective": value.superlative_adjective,
                "dictionary_parts_of_speech": possible_parts_of_speech,
                "dictionary_verb_roles": dictionary_verb_roles,
                "provider": {
                    "name": evidence.provenance.provider.name.as_str(),
                    "version": evidence.provenance.provider.version.as_deref(),
                },
            }));
        }
        cases.push(json!({"id": case.id, "tokens": tokens}));
    }

    let output: Value = json!({"schema_version": 1, "provider": "harper", "cases": cases});
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
