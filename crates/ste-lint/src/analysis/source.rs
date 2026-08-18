use pulldown_cmark::{Event, Parser, Tag, TagEnd};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SourceSpan {
    pub start: usize,
    pub end: usize,
}

impl SourceSpan {
    fn new(start: usize, end: usize) -> Option<Self> {
        (start < end).then_some(Self { start, end })
    }

    pub(crate) fn intersects(self, start: usize, end: usize) -> bool {
        start < self.end && self.start < end
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SourceListItem {
    pub span: SourceSpan,
    pub content_start: usize,
    pub content_end: usize,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct SourceDocument {
    protected: Vec<SourceSpan>,
    paragraphs: Vec<SourceSpan>,
    list_items: Vec<SourceListItem>,
}

impl SourceDocument {
    pub(crate) fn new(text: &str) -> Self {
        let mut document = Self::default();
        let mut paragraph_starts = Vec::new();
        let mut item_starts = Vec::new();
        let mut code_block_starts = Vec::new();

        for (event, range) in Parser::new(text).into_offset_iter() {
            match event {
                Event::Start(Tag::Paragraph) => paragraph_starts.push(range.start),
                Event::End(TagEnd::Paragraph) => {
                    if let Some(start) = paragraph_starts.pop()
                        && let Some(span) = SourceSpan::new(start, range.end)
                    {
                        document.paragraphs.push(span);
                    }
                }
                Event::Start(Tag::Item) => item_starts.push(range.start),
                Event::End(TagEnd::Item) => {
                    if let Some(start) = item_starts.pop()
                        && let Some(span) = SourceSpan::new(start, range.end)
                    {
                        let content_start = list_item_content_start(text, span);
                        let content_end = trim_line_end(text, span.end);
                        if content_start < content_end {
                            document.list_items.push(SourceListItem {
                                span,
                                content_start,
                                content_end,
                            });
                        }
                    }
                }
                Event::Code(_) => {
                    if let Some(span) = SourceSpan::new(range.start, range.end) {
                        document.protected.push(span);
                    }
                }
                Event::Start(Tag::CodeBlock(_)) => code_block_starts.push(range.start),
                Event::End(TagEnd::CodeBlock) => {
                    if let Some(start) = code_block_starts.pop()
                        && let Some(span) = SourceSpan::new(start, range.end)
                    {
                        document.protected.push(span);
                    }
                }
                _ => {}
            }
        }

        document
            .protected
            .sort_by_key(|span| (span.start, span.end));
        document.protected = merge_spans(document.protected);
        document
            .paragraphs
            .sort_by_key(|span| (span.start, span.end));
        document
            .list_items
            .sort_by_key(|item| (item.span.start, item.span.end));
        document
    }

    pub(crate) fn protected_ranges(&self) -> &[SourceSpan] {
        &self.protected
    }

    pub(crate) fn paragraph_ranges(&self) -> &[SourceSpan] {
        &self.paragraphs
    }

    pub(crate) fn list_items(&self) -> &[SourceListItem] {
        &self.list_items
    }
}

fn merge_spans(spans: Vec<SourceSpan>) -> Vec<SourceSpan> {
    let mut merged: Vec<SourceSpan> = Vec::new();
    for span in spans {
        if let Some(previous) = merged.last_mut()
            && span.start <= previous.end
        {
            previous.end = previous.end.max(span.end);
            continue;
        }
        merged.push(span);
    }
    merged
}

fn list_item_content_start(text: &str, span: SourceSpan) -> usize {
    let line_end = text[span.start..span.end]
        .find(['\r', '\n'])
        .map_or(span.end, |offset| span.start + offset);
    let line = &text[span.start..line_end];
    let trimmed = line.trim_start();
    let leading = line.len() - trimmed.len();

    for marker in ["- ", "* ", "+ "] {
        if trimmed.starts_with(marker) {
            return span.start + leading + marker.len();
        }
    }

    let bytes = trimmed.as_bytes();
    let mut index = 0;
    while index < bytes.len() && bytes[index].is_ascii_digit() {
        index += 1;
    }
    if index > 0
        && index + 1 < bytes.len()
        && matches!(bytes[index], b'.' | b')')
        && bytes[index + 1].is_ascii_whitespace()
    {
        return span.start + leading + index + 2;
    }

    span.start + leading
}

fn trim_line_end(text: &str, end: usize) -> usize {
    let mut end = end.min(text.len());
    while end > 0 && matches!(text.as_bytes()[end - 1], b'\n' | b'\r') {
        end -= 1;
    }
    end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_document_tracks_multiline_inline_code_as_protected() {
        let text = "USE `alpha.\nbeta` here.";
        let document = SourceDocument::new(text);
        assert_eq!(document.protected_ranges().len(), 1);
        let span = document.protected_ranges()[0];
        assert_eq!(&text[span.start..span.end], "`alpha.\nbeta`");
    }

    #[test]
    fn source_document_tracks_commonmark_list_item_content() {
        let text = "DO THIS:\n- Remove this.\n2. Remove that.";
        let document = SourceDocument::new(text);
        assert_eq!(document.list_items().len(), 2);
        assert_eq!(
            &text[document.list_items()[0].content_start..document.list_items()[0].content_end],
            "Remove this."
        );
        assert_eq!(
            &text[document.list_items()[1].content_start..document.list_items()[1].content_end],
            "Remove that."
        );
    }
}
