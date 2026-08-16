use crate::context::{SemanticOrderTarget, SemanticOrderTargetKind};
use crate::structure::paragraph_ranges;

use super::document::{AnalysisDocument, Resolution};
use super::entity::EntityMention;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocumentNodeKind {
    Sentence,
    Paragraph,
    Topic,
    EntityMention,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DocumentNodeId {
    pub kind: DocumentNodeKind,
    pub index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocumentSpan {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentNode {
    pub id: DocumentNodeId,
    pub span: DocumentSpan,
    pub label: Option<String>,
    pub provenance: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentRelationKind {
    Contains,
    Precedes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocumentRelation {
    pub kind: DocumentRelationKind,
    pub from: DocumentNodeId,
    pub to: DocumentNodeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentReferenceRelation {
    pub reference: DocumentSpan,
    pub source_sentence: DocumentNodeId,
    pub target: Resolution<DocumentNodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSemanticOrdering {
    pub before: Resolution<DocumentNodeId>,
    pub after: Resolution<DocumentNodeId>,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DocumentGraph {
    pub nodes: Vec<DocumentNode>,
    pub relations: Vec<DocumentRelation>,
    pub references: Vec<DocumentReferenceRelation>,
    pub semantic_orderings: Vec<DocumentSemanticOrdering>,
}

impl<'a> AnalysisDocument<'a> {
    pub fn document_graph(&self) -> DocumentGraph {
        let paragraphs = paragraph_ranges(self.text());
        let entities = self.entity_mentions();
        let mut graph = DocumentGraph::default();

        for sentence in self.sentences() {
            graph.nodes.push(DocumentNode {
                id: DocumentNodeId {
                    kind: DocumentNodeKind::Sentence,
                    index: sentence.id,
                },
                span: DocumentSpan {
                    start: sentence.start,
                    end: sentence.end,
                },
                label: None,
                provenance: None,
            });
        }

        for (index, (start, end)) in paragraphs.iter().copied().enumerate() {
            graph.nodes.push(DocumentNode {
                id: DocumentNodeId {
                    kind: DocumentNodeKind::Paragraph,
                    index,
                },
                span: DocumentSpan { start, end },
                label: None,
                provenance: None,
            });
        }

        if let Some(context) = self.context() {
            for (index, topic) in context.topics.iter().enumerate() {
                graph.nodes.push(DocumentNode {
                    id: DocumentNodeId {
                        kind: DocumentNodeKind::Topic,
                        index,
                    },
                    span: DocumentSpan {
                        start: topic.start,
                        end: topic.end,
                    },
                    label: Some(topic.topic.clone()),
                    provenance: Some(topic.source.clone()),
                });
            }
        }

        for (index, mention) in entities.iter().enumerate() {
            graph.nodes.push(DocumentNode {
                id: DocumentNodeId {
                    kind: DocumentNodeKind::EntityMention,
                    index,
                },
                span: DocumentSpan {
                    start: mention.span.start,
                    end: mention.span.end,
                },
                label: Some(mention.surface.clone()),
                provenance: mention.provenance.first().cloned(),
            });
        }

        add_sequence_relations(&mut graph, DocumentNodeKind::Sentence);
        add_sequence_relations(&mut graph, DocumentNodeKind::Paragraph);
        add_containment_relations(&mut graph);
        graph.references = build_reference_relations(self, &entities);
        graph.semantic_orderings = build_semantic_orderings(self, &graph.nodes);
        graph
    }
}

fn add_sequence_relations(graph: &mut DocumentGraph, kind: DocumentNodeKind) {
    let ids = graph
        .nodes
        .iter()
        .filter(|node| node.id.kind == kind)
        .map(|node| node.id)
        .collect::<Vec<_>>();
    for pair in ids.windows(2) {
        graph.relations.push(DocumentRelation {
            kind: DocumentRelationKind::Precedes,
            from: pair[0],
            to: pair[1],
        });
    }
}

fn add_containment_relations(graph: &mut DocumentGraph) {
    let paragraphs = graph
        .nodes
        .iter()
        .filter(|node| node.id.kind == DocumentNodeKind::Paragraph)
        .cloned()
        .collect::<Vec<_>>();
    let children = graph
        .nodes
        .iter()
        .filter(|node| {
            matches!(
                node.id.kind,
                DocumentNodeKind::Sentence
                    | DocumentNodeKind::Topic
                    | DocumentNodeKind::EntityMention
            )
        })
        .cloned()
        .collect::<Vec<_>>();

    for paragraph in paragraphs {
        for child in &children {
            if child.span.start >= paragraph.span.start && child.span.end <= paragraph.span.end {
                graph.relations.push(DocumentRelation {
                    kind: DocumentRelationKind::Contains,
                    from: paragraph.id,
                    to: child.id,
                });
            }
        }
    }
}

fn build_reference_relations(
    analysis: &AnalysisDocument<'_>,
    entities: &[EntityMention],
) -> Vec<DocumentReferenceRelation> {
    let mut relations = Vec::new();
    for (token_index, token) in analysis.tokens().iter().enumerate() {
        if !matches!(token.text.to_ascii_lowercase().as_str(), "it" | "its") {
            continue;
        }
        let Some(sentence_id) = token.sentence_id else {
            continue;
        };
        let target = map_entity_resolution(analysis.reference_at(token_index), entities);
        relations.push(DocumentReferenceRelation {
            reference: DocumentSpan {
                start: token.start,
                end: token.end,
            },
            source_sentence: DocumentNodeId {
                kind: DocumentNodeKind::Sentence,
                index: sentence_id,
            },
            target,
        });
    }
    relations
}

fn map_entity_resolution(
    resolution: Resolution<EntityMention>,
    entities: &[EntityMention],
) -> Resolution<DocumentNodeId> {
    match resolution {
        Resolution::Resolved(mention) => {
            entity_node_id(&mention, entities).map_or(Resolution::Unknown, Resolution::Resolved)
        }
        Resolution::Ambiguous(mentions) => {
            let targets = mentions
                .iter()
                .filter_map(|mention| entity_node_id(mention, entities))
                .collect::<Vec<_>>();
            resolution_from_candidates(targets)
        }
        Resolution::Unknown => Resolution::Unknown,
    }
}

fn entity_node_id(mention: &EntityMention, entities: &[EntityMention]) -> Option<DocumentNodeId> {
    entities
        .iter()
        .position(|candidate| {
            candidate.identity == mention.identity && candidate.span == mention.span
        })
        .map(|index| DocumentNodeId {
            kind: DocumentNodeKind::EntityMention,
            index,
        })
}

fn build_semantic_orderings(
    analysis: &AnalysisDocument<'_>,
    nodes: &[DocumentNode],
) -> Vec<DocumentSemanticOrdering> {
    analysis.context().map_or_else(Vec::new, |context| {
        context
            .semantic_orderings
            .iter()
            .map(|ordering| DocumentSemanticOrdering {
                before: resolve_semantic_target(ordering.before, nodes),
                after: resolve_semantic_target(ordering.after, nodes),
                source: ordering.source.clone(),
            })
            .collect()
    })
}

fn resolve_semantic_target(
    target: SemanticOrderTarget,
    nodes: &[DocumentNode],
) -> Resolution<DocumentNodeId> {
    let kind = match target.kind {
        SemanticOrderTargetKind::Sentence => DocumentNodeKind::Sentence,
        SemanticOrderTargetKind::Paragraph => DocumentNodeKind::Paragraph,
        SemanticOrderTargetKind::Topic => DocumentNodeKind::Topic,
        SemanticOrderTargetKind::EntityMention => DocumentNodeKind::EntityMention,
    };
    let candidates = nodes
        .iter()
        .filter(|node| {
            node.id.kind == kind && node.span.start == target.start && node.span.end == target.end
        })
        .map(|node| node.id)
        .collect::<Vec<_>>();
    resolution_from_candidates(candidates)
}

fn resolution_from_candidates<T>(mut candidates: Vec<T>) -> Resolution<T> {
    match candidates.len() {
        0 => Resolution::Unknown,
        1 => Resolution::Resolved(candidates.pop().unwrap()),
        _ => Resolution::Ambiguous(candidates),
    }
}
