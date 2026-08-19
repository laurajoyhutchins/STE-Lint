use serde_json::json;
use ste_data::RuntimeLexicon;
use ste_lint::{
    AnalysisDocument, EvidenceTarget, LintMode, SemanticObservation, ShadowEvidenceError,
};

const TEXT: &str = "The pump is open. It runs.";
const SOURCE_SHA256: &str = "4ab4f8c5b5bda8d6ccb78ff9a600a641464d166b65900760aff129b68a2f3396";
const CONFIGURATION: &str =
    "lang=en;package=default_accurate;processors=tokenize,mwt,pos,lemma,depparse,constituency,ner,coref;use_gpu=false;offline=true";
const CONFIGURATION_SHA256: &str =
    "664a351f8e4d0b885e2a164a4e16545c80d7550e1468bf12ae0fd47a3fb9830d";

fn bundle(source_sha256: &str, pump_surface: &str) -> String {
    json!({
        "schema_version": 1,
        "source": {
            "sha256": source_sha256,
            "bytes": TEXT.len(),
        },
        "provider": {
            "name": "stanza",
            "version": "1.14.0",
        },
        "model": {
            "name": "en-default_accurate",
            "version": "default_accurate",
            "artifact_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        },
        "configuration": CONFIGURATION,
        "configuration_sha256": CONFIGURATION_SHA256,
        "evidence": [
            {
                "kind": "dependency",
                "relation": "nsubj",
                "source": {"start": 4, "end": 8, "surface": pump_surface},
                "target": {"start": 12, "end": 16, "surface": "open"},
                "confidence": 0.97,
            },
            {
                "kind": "constituency",
                "label": "NP",
                "span": {"start": 0, "end": 8, "surface": "The pump"},
            },
            {
                "kind": "named_entity",
                "class": "EQUIPMENT",
                "span": {"start": 4, "end": 8, "surface": "pump"},
            },
            {
                "kind": "coreference",
                "representative": "pump",
                "source": {"start": 18, "end": 20, "surface": "It"},
                "target": {"start": 4, "end": 8, "surface": "pump"},
            }
        ],
    })
    .to_string()
}

fn analysis<'a>(lexicon: &'a RuntimeLexicon) -> AnalysisDocument<'a> {
    AnalysisDocument::new(TEXT, lexicon, None, None, LintMode::Procedural)
}

#[test]
fn imports_shadow_semantics_without_replacing_harper_evidence() {
    let lexicon = RuntimeLexicon::embedded().expect("embedded lexicon");
    let analysis = analysis(&lexicon);
    let lexical_count = analysis.lexical_evidence().len();

    let shadow = analysis
        .import_shadow_evidence_json(&bundle(SOURCE_SHA256, "pump"))
        .expect("valid shadow bundle");

    assert_eq!(analysis.lexical_evidence().len(), lexical_count);
    assert!(
        analysis
            .lexical_evidence()
            .iter()
            .all(|evidence| evidence.provenance.provider.name == "harper-core")
    );
    assert_eq!(shadow.identity.provider.name, "stanza");
    assert_eq!(
        shadow.identity.model.artifact_sha256.as_deref(),
        Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
    );
    assert_eq!(shadow.identity.configuration, CONFIGURATION);
    assert_eq!(shadow.evidence.len(), 4);

    assert_eq!(
        shadow.evidence[0].value,
        SemanticObservation::Dependency {
            relation: "nsubj".into(),
        }
    );
    assert!(matches!(
        shadow.evidence[0].target,
        EvidenceTarget::Relation { source, target }
            if source.start == 4
                && source.end == 8
                && target.start == 12
                && target.end == 16
    ));
    assert_eq!(
        shadow.evidence[3].value,
        SemanticObservation::Coreference {
            representative: "pump".into(),
        }
    );
}

#[test]
fn rejects_a_bundle_for_different_source_bytes() {
    let lexicon = RuntimeLexicon::embedded().expect("embedded lexicon");
    let analysis = analysis(&lexicon);
    let error = analysis
        .import_shadow_evidence_json(&bundle(
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "pump",
        ))
        .expect_err("source mismatch must fail closed");

    assert_eq!(error, ShadowEvidenceError::SourceDigestMismatch);
}

#[test]
fn rejects_a_span_whose_surface_does_not_match_canonical_source() {
    let lexicon = RuntimeLexicon::embedded().expect("embedded lexicon");
    let analysis = analysis(&lexicon);
    let error = analysis
        .import_shadow_evidence_json(&bundle(SOURCE_SHA256, "valve"))
        .expect_err("surface mismatch must fail closed");

    assert!(matches!(
        error,
        ShadowEvidenceError::SurfaceMismatch {
            kind,
            start: 4,
            end: 8,
        } if kind == "dependency source"
    ));
}