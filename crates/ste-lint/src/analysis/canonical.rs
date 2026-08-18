use super::source::SourceDocument;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CanonicalSpan {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct CanonicalSource<'a> {
    text: &'a str,
    structure: SourceDocument,
}

impl<'a> CanonicalSource<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        Self {
            text,
            structure: SourceDocument::new(text),
        }
    }

    pub(crate) fn text(&self) -> &'a str {
        self.text
    }

    pub(crate) fn span(&self, start: usize, end: usize) -> Option<CanonicalSpan> {
        (start < end
            && end <= self.text.len()
            && self.text.is_char_boundary(start)
            && self.text.is_char_boundary(end))
        .then_some(CanonicalSpan { start, end })
    }

    pub(crate) fn is_protected(&self, span: CanonicalSpan) -> bool {
        self.structure
            .protected_ranges()
            .iter()
            .any(|protected| protected.intersects(span.start, span.end))
    }
}
