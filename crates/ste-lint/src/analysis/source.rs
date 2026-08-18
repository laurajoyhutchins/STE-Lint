use pulldown_cmark::{Event, Parser, Tag, TagEnd};

use crate::{LintContext, TextAuthorityKind};

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
        Self::with_context(text, None)
    }

    pub(crate) fn with_context(text: &'a str, context: Option<&LintContext>) -> Self {
        Self {
            text,
            structure: SourceDocument::with_context(text, context),
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
        self.structure.is_protected(span.start, span.end)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SourceSpan {
    pub start: usize,
    pub end: usize,
}

impl SourceSpan {
    pub(crate) fn new(start: usize, end: usize) -> Option<Self> {
        (start < end).then_some(Self { start, end })
    }

    pub(crate) fn intersects(self, start: usize, end: usize) -> bool {
        start < self.end && self.start < end
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SourceList {
    pub id: usize,
    pub span: SourceSpan,
    pub depth: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SourceListItem {
    pub span: SourceSpan,
    pub content_start: usize,
    pub content_end: usize,
    pub list_id: usize,
    pub depth: usize,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct SourceDocument {
    protected: Vec<SourceSpan>,
    headings: Vec<SourceSpan>,
    paragraphs: Vec<SourceSpan>,
    lists: Vec<SourceList>,
    list_items: Vec<SourceListItem>,
}

#[derive(Debug, Clone, Copy)]
struct LineSpan {
    start: usize,
    end: usize,
}

#[derive(Debug, Clone, Copy)]
struct OpenLegacyList {
    id: usize,
    start: usize,
    indent: usize,
}

#[derive(Debug, Clone, Copy)]
struct OpenLegacyItem {
    start: usize,
    content_start: usize,
    list_id: usize,
}

impl SourceDocument {
    pub(crate) fn new(text: &str) -> Self {
        Self::with_context(text, None)
    }

    pub(crate) fn with_context(text: &str, context: Option<&LintContext>) -> Self {
        let mut document = Self::default();
        let mut paragraph_starts = Vec::new();
        let mut heading_starts = Vec::new();
        let mut list_starts = Vec::<(usize, usize)>::new();
        let mut item_starts = Vec::<(usize, usize)>::new();
        let mut code_block_starts = Vec::new();
        let mut next_list_id = 0usize;

        for (event, range) in Parser::new(text).into_offset_iter() {
            match event {
                Event::Start(Tag::Paragraph) => paragraph_starts.push(range.start),
                Event::End(TagEnd::Paragraph) => {
                    if let Some(start) = paragraph_starts.pop()
                        && let Some(span) = SourceSpan::new(start, trim_line_end(text, range.end))
                    {
                        document.paragraphs.push(span);
                    }
                }
                Event::Start(Tag::Heading { .. }) => heading_starts.push(range.start),
                Event::End(TagEnd::Heading(_)) => {
                    if let Some(start) = heading_starts.pop()
                        && let Some(span) = SourceSpan::new(start, trim_line_end(text, range.end))
                    {
                        document.headings.push(span);
                    }
                }
                Event::Start(Tag::List(_)) => {
                    let id = next_list_id;
                    next_list_id += 1;
                    list_starts.push((id, range.start));
                }
                Event::End(TagEnd::List(_)) => {
                    if let Some((id, start)) = list_starts.pop()
                        && let Some(span) = SourceSpan::new(start, range.end)
                    {
                        document.lists.push(SourceList { id, span, depth: 0 });
                    }
                }
                Event::Start(Tag::Item) => {
                    if let Some((list_id, _)) = list_starts.last().copied() {
                        item_starts.push((range.start, list_id));
                    }
                }
                Event::End(TagEnd::Item) => {
                    if let Some((start, list_id)) = item_starts.pop()
                        && let Some(span) = SourceSpan::new(start, range.end)
                    {
                        let content_start = list_item_content_start(text, span);
                        let content_end = trim_line_end(text, span.end);
                        if content_start < content_end {
                            document.list_items.push(SourceListItem {
                                span,
                                content_start,
                                content_end,
                                list_id,
                                depth: 0,
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

        if let Some(context) = context {
            document.protected.extend(
                context
                    .occurrences
                    .iter()
                    .filter(|occurrence| {
                        occurrence
                            .text_authority
                            .is_some_and(protects_from_authored_text_rules)
                    })
                    .filter_map(|occurrence| SourceSpan::new(occurrence.start, occurrence.end)),
            );
        }

        append_legacy_lists(text, &mut document, &mut next_list_id);
        normalize_list_depths(&mut document);

        document.protected.sort_by_key(|span| (span.start, span.end));
        document.protected = merge_spans(document.protected);
        document.headings.sort_by_key(|span| (span.start, span.end));
        document.headings = merge_spans(document.headings);
        document.paragraphs.sort_by_key(|span| (span.start, span.end));
        document
            .lists
            .sort_by_key(|list| (list.span.start, list.depth, list.span.end));
        document
            .list_items
            .sort_by_key(|item| (item.span.start, item.depth, item.span.end));
        document
    }

    pub(crate) fn protected_ranges(&self) -> &[SourceSpan] {
        &self.protected
    }

    pub(crate) fn heading_ranges(&self) -> &[SourceSpan] {
        &self.headings
    }

    pub(crate) fn paragraph_ranges(&self) -> &[SourceSpan] {
        &self.paragraphs
    }

    pub(crate) fn lists(&self) -> &[SourceList] {
        &self.lists
    }

    pub(crate) fn list_items(&self) -> &[SourceListItem] {
        &self.list_items
    }

    pub(crate) fn is_protected(&self, start: usize, end: usize) -> bool {
        self.protected
            .iter()
            .any(|span| span.intersects(start, end))
    }
}

fn protects_from_authored_text_rules(authority: TextAuthorityKind) -> bool {
    matches!(
        authority,
        TextAuthorityKind::ProtectedText
            | TextAuthorityKind::QuotedExternalText
            | TextAuthorityKind::CodeOrVerbatim
            | TextAuthorityKind::Formula
            | TextAuthorityKind::DocumentNumbering
    )
}

fn append_legacy_lists(text: &str, document: &mut SourceDocument, next_list_id: &mut usize) {
    let commonmark_lists = document
        .lists
        .iter()
        .map(|list| list.span)
        .collect::<Vec<_>>();
    let mut open_lists = Vec::<OpenLegacyList>::new();
    let mut open_items = Vec::<OpenLegacyItem>::new();

    for line in line_spans(text) {
        let raw = text[line.start..line.end].trim_end_matches(['\r', '\n']);
        if raw.trim().is_empty() {
            close_all_legacy(text, line.start, document, &mut open_lists, &mut open_items);
            continue;
        }
        if commonmark_lists
            .iter()
            .any(|span| span.intersects(line.start, line.end))
        {
            continue;
        }
        let Some((indent, content_offset)) = legacy_ste_list_item_layout(raw) else {
            continue;
        };

        while open_lists.last().is_some_and(|list| list.indent > indent) {
            close_legacy_item(text, line.start, document, &mut open_items);
            close_legacy_list(line.start, document, &mut open_lists);
        }
        if open_lists.last().is_none_or(|list| list.indent < indent) {
            let id = *next_list_id;
            *next_list_id += 1;
            open_lists.push(OpenLegacyList {
                id,
                start: line.start,
                indent,
            });
        }
        let list_id = open_lists
            .last()
            .expect("legacy list exists after opening marker")
            .id;
        if open_items
            .last()
            .is_some_and(|item| item.list_id == list_id)
        {
            close_legacy_item(text, line.start, document, &mut open_items);
        }
        open_items.push(OpenLegacyItem {
            start: line.start,
            content_start: line.start + content_offset,
            list_id,
        });
    }
    close_all_legacy(text, text.len(), document, &mut open_lists, &mut open_items);
}

fn close_all_legacy(
    text: &str,
    end: usize,
    document: &mut SourceDocument,
    open_lists: &mut Vec<OpenLegacyList>,
    open_items: &mut Vec<OpenLegacyItem>,
) {
    while !open_items.is_empty() {
        close_legacy_item(text, end, document, open_items);
    }
    while !open_lists.is_empty() {
        close_legacy_list(end, document, open_lists);
    }
}

fn close_legacy_item(
    text: &str,
    end: usize,
    document: &mut SourceDocument,
    open_items: &mut Vec<OpenLegacyItem>,
) {
    let Some(item) = open_items.pop() else {
        return;
    };
    let content_end = trim_line_end(text, end);
    let Some(span) = SourceSpan::new(item.start, end) else {
        return;
    };
    if item.content_start < content_end {
        document.list_items.push(SourceListItem {
            span,
            content_start: item.content_start,
            content_end,
            list_id: item.list_id,
            depth: 0,
        });
    }
}

fn close_legacy_list(
    end: usize,
    document: &mut SourceDocument,
    open_lists: &mut Vec<OpenLegacyList>,
) {
    let Some(list) = open_lists.pop() else {
        return;
    };
    if let Some(span) = SourceSpan::new(list.start, end) {
        document.lists.push(SourceList {
            id: list.id,
            span,
            depth: 0,
        });
    }
}

fn normalize_list_depths(document: &mut SourceDocument) {
    for _ in 0..document.lists.len().max(1) {
        let depths = document
            .lists
            .iter()
            .map(|list| (list.id, list.depth))
            .collect::<Vec<_>>();
        let items = document.list_items.clone();
        for list in &mut document.lists {
            let parent = items
                .iter()
                .filter(|item| item.list_id != list.id)
                .filter(|item| item.span.start < list.span.start && list.span.end <= item.span.end)
                .min_by_key(|item| item.span.end.saturating_sub(item.span.start));
            let Some(parent) = parent else {
                continue;
            };
            let parent_depth = depths
                .iter()
                .find_map(|(id, depth)| (*id == parent.list_id).then_some(*depth))
                .unwrap_or(0);
            list.depth = parent_depth + 1;
        }
    }
    for item in &mut document.list_items {
        item.depth = document
            .lists
            .iter()
            .find_map(|list| (list.id == item.list_id).then_some(list.depth))
            .unwrap_or(0);
    }
}

fn merge_spans(spans: Vec<SourceSpan>) -> Vec<SourceSpan> {
    let mut merged = Vec::<SourceSpan>::new();
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

fn legacy_ste_list_item_layout(line: &str) -> Option<(usize, usize)> {
    let trimmed = line.trim_start();
    let leading = line.len() - trimmed.len();
    if trimmed.starts_with("• ") {
        return Some((leading, leading + "• ".len()));
    }
    let bytes = trimmed.as_bytes();
    if bytes.first() == Some(&b'(') {
        let mut index = 1;
        let label_start = index;
        while index < bytes.len() && bytes[index].is_ascii_alphanumeric() {
            index += 1;
        }
        if index > label_start
            && index + 1 < bytes.len()
            && bytes[index] == b')'
            && bytes[index + 1].is_ascii_whitespace()
        {
            return Some((leading, leading + index + 2));
        }
        return None;
    }
    let mut index = 0;
    while index < bytes.len() && bytes[index].is_ascii_alphabetic() {
        index += 1;
    }
    let label = &trimmed[..index];
    if label.len() == 1
        && index + 1 < bytes.len()
        && matches!(bytes[index], b'.' | b')')
        && bytes[index + 1].is_ascii_whitespace()
    {
        return Some((leading, leading + index + 2));
    }
    None
}

fn line_spans(text: &str) -> Vec<LineSpan> {
    let mut lines = Vec::new();
    let mut start = 0;
    for (index, character) in text.char_indices() {
        if character == '\n' {
            lines.push(LineSpan {
                start,
                end: index + 1,
            });
            start = index + 1;
        }
    }
    if start < text.len() || text.is_empty() {
        lines.push(LineSpan {
            start,
            end: text.len(),
        });
    }
    lines
}

fn trim_line_end(text: &str, end: usize) -> usize {
    let mut end = end.min(text.len());
    while end > 0 && matches!(text.as_bytes()[end - 1], b'\n' | b'\r') {
        end -= 1;
    }
    end
}
