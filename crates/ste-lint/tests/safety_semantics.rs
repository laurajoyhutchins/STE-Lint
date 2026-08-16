use ste_data::RuntimeLexicon;
use ste_lint::{
    AnalysisDocument, LintContext, LintMode, Resolution, SafetyEvidenceSource, SafetyLevel,
};

#[test]
fn safety_semantics_combine_structural_level_and_command_with_explicit_context() {
    let text = "WARNING: DISCONNECT POWER TO PREVENT SHOCK.";
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let context = LintContext::from_json(
        r#"{
          "safety_facts": [{
            "start": 0,
            "end": 43,
            "source": "project-hazard-analysis",
            "actor": {"start":20,"end":25},
            "hazard": {"start":37,"end":42},
            "consequence": {"start":29,"end":42}
          }]
        }"#,
    )
    .unwrap();
    context.validate(text.len()).unwrap();

    let analysis = AnalysisDocument::new(
        text,
        &lexicon,
        None,
        Some(&context),
        LintMode::Procedural,
    );
    let semantics = analysis.safety_semantics();
    assert_eq!(semantics.len(), 1);
    let safety = &semantics[0];
    assert_eq!((safety.span.start, safety.span.end), (0, 43));

    let Resolution::Resolved(level) = &safety.level else {
        panic!("WARNING label should resolve the structural safety level");
    };
    assert_eq!(level.level, SafetyLevel::Warning);
    assert_eq!(level.source, SafetyEvidenceSource::Structure);

    let Resolution::Resolved(command) = &safety.command else {
        panic!("bounded imperative opening should resolve the command span");
    };
    assert_eq!((command.span.start, command.span.end), (9, 19));
    assert_eq!(command.source, SafetyEvidenceSource::Structure);

    let Resolution::Resolved(actor) = &safety.actor else {
        panic!("explicit actor evidence should resolve");
    };
    assert_eq!((actor.span.start, actor.span.end), (20, 25));
    assert_eq!(
        actor.source,
        SafetyEvidenceSource::Context("project-hazard-analysis".into())
    );

    let Resolution::Resolved(hazard) = &safety.hazard else {
        panic!("explicit hazard evidence should resolve");
    };
    assert_eq!((hazard.span.start, hazard.span.end), (37, 42));

    let Resolution::Resolved(consequence) = &safety.consequence else {
        panic!("explicit consequence evidence should resolve");
    };
    assert_eq!((consequence.span.start, consequence.span.end), (29, 42));
}

#[test]
fn structural_safety_without_semantic_context_stays_unknown_where_required() {
    let text = "CAUTION: DISCONNECT POWER.";
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let analysis =
        AnalysisDocument::new(text, &lexicon, None, None, LintMode::Procedural);

    let semantics = analysis.safety_semantics();
    assert_eq!(semantics.len(), 1);
    let safety = &semantics[0];
    assert!(matches!(safety.level, Resolution::Resolved(_)));
    assert!(matches!(safety.command, Resolution::Resolved(_)));
    assert!(matches!(safety.actor, Resolution::Unknown));
    assert!(matches!(safety.hazard, Resolution::Unknown));
    assert!(matches!(safety.consequence, Resolution::Unknown));
}

#[test]
fn conflicting_explicit_safety_levels_remain_ambiguous() {
    let text = "WARNING: DISCONNECT POWER.";
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let context = LintContext::from_json(
        r#"{
          "safety_facts": [{
            "start": 0,
            "end": 26,
            "source": "project-hazard-analysis",
            "safety_level": "caution"
          }]
        }"#,
    )
    .unwrap();
    let analysis = AnalysisDocument::new(
        text,
        &lexicon,
        None,
        Some(&context),
        LintMode::Procedural,
    );

    let semantics = analysis.safety_semantics();
    let Resolution::Ambiguous(levels) = &semantics[0].level else {
        panic!("conflicting structural and explicit safety levels must remain ambiguous");
    };
    assert_eq!(levels.len(), 2);
    assert!(levels.iter().any(|level| level.level == SafetyLevel::Warning));
    assert!(levels.iter().any(|level| level.level == SafetyLevel::Caution));
}

#[test]
fn competing_explicit_actor_evidence_remains_ambiguous() {
    let text = "WARNING: DISCONNECT POWER TO PREVENT SHOCK.";
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let context = LintContext::from_json(
        r#"{
          "safety_facts": [
            {
              "start": 0,
              "end": 43,
              "source": "analysis-a",
              "actor": {"start":20,"end":25}
            },
            {
              "start": 0,
              "end": 43,
              "source": "analysis-b",
              "actor": {"start":37,"end":42}
            }
          ]
        }"#,
    )
    .unwrap();
    let analysis = AnalysisDocument::new(
        text,
        &lexicon,
        None,
        Some(&context),
        LintMode::Procedural,
    );

    let semantics = analysis.safety_semantics();
    let Resolution::Ambiguous(actors) = &semantics[0].actor else {
        panic!("competing explicit actor evidence must remain ambiguous");
    };
    assert_eq!(actors.len(), 2);
    assert_eq!(
        actors
            .iter()
            .map(|actor| (actor.span.start, actor.span.end))
            .collect::<Vec<_>>(),
        vec![(20, 25), (37, 42)]
    );
}

#[test]
fn context_fact_for_non_safety_span_does_not_create_safety_semantics() {
    let text = "DISCONNECT POWER.";
    let lexicon = RuntimeLexicon::embedded().unwrap();
    let context = LintContext::from_json(
        r#"{
          "safety_facts": [{
            "start": 0,
            "end": 17,
            "source": "project-hazard-analysis",
            "safety_level": "warning",
            "hazard": {"start":11,"end":16}
          }]
        }"#,
    )
    .unwrap();
    let analysis = AnalysisDocument::new(
        text,
        &lexicon,
        None,
        Some(&context),
        LintMode::Procedural,
    );

    assert!(analysis.safety_semantics().is_empty());
}
