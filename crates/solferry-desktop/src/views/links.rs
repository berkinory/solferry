use std::ops::Range;

use gpui_kit::component::ActiveTheme;
use gpui_kit::{
    App, Div, ElementId, HighlightStyle, InteractiveText, ParentElement, SharedString, Styled,
    StyledText, UnderlineStyle, div, px,
};

use crate::tokens::TYPE_BODY;

/// Only links written out with one of these, never a bare domain, so nothing opens that the
/// sender didn't spell out.
const SCHEMES: [&str; 3] = ["https://", "http://", "mailto:"];
const TRAILING: &[char] = &['.', ',', ';', ':', '!', '?', '\'', '"'];

/// Received text with its links underlined. They open only when clicked.
pub fn text(id: impl Into<ElementId>, value: impl Into<SharedString>, cx: &App) -> Div {
    let value = value.into();
    let ranges = find(&value);
    let style = HighlightStyle {
        color: Some(cx.theme().primary),
        underline: Some(UnderlineStyle { thickness: px(1.), ..UnderlineStyle::default() }),
        ..HighlightStyle::default()
    };
    let urls: Vec<String> = ranges.iter().map(|range| value[range.clone()].to_owned()).collect();
    let styled =
        StyledText::new(value).with_highlights(ranges.iter().map(|range| (range.clone(), style)));
    let interactive = InteractiveText::new(id, styled)
        .on_click(ranges, move |index, _, cx| cx.open_url(&urls[index]));
    div().text_size(px(TYPE_BODY)).child(interactive)
}

/// Byte ranges of the http, https and mailto links in `text`.
fn find(text: &str) -> Vec<Range<usize>> {
    // ASCII lowercasing keeps every byte offset where it was.
    let lower = text.to_ascii_lowercase();
    let mut links = Vec::new();
    let mut at = 0;
    while let Some((start, scheme)) = next_scheme(&lower, at) {
        let rest = &text[start..];
        let end = rest.find(|c: char| c.is_whitespace() || matches!(c, '<' | '>' | '"'));
        let link = trim(&rest[..end.unwrap_or(rest.len())]);
        if link.len() > scheme.len() {
            links.push(start..start + link.len());
        }
        at = start + link.len().max(scheme.len());
    }
    links
}

/// Where the next scheme starts that isn't the tail of a longer word.
fn next_scheme(lower: &str, from: usize) -> Option<(usize, &'static str)> {
    lower[from..].char_indices().map(|(offset, _)| from + offset).find_map(|start| {
        let starts_word = !lower[..start].chars().next_back().is_some_and(char::is_alphanumeric);
        let scheme = SCHEMES.into_iter().find(|scheme| lower[start..].starts_with(scheme))?;
        starts_word.then_some((start, scheme))
    })
}

/// Leaves sentence punctuation and an unopened closing bracket outside the link.
fn trim(mut link: &str) -> &str {
    loop {
        let unopened = link.ends_with(')') && link.matches('(').count() < link.matches(')').count();
        if link.ends_with(TRAILING) || unopened {
            link = &link[..link.len() - 1];
        } else {
            return link;
        }
    }
}
