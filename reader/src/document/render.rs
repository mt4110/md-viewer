use super::images::LocalImage;
use super::{Document, Heading, ImageBudget, MAX_DOCUMENT, local_path, read_local_image};
use base64::{Engine, engine::general_purpose::STANDARD};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, html};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

fn escaped(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
pub fn render(text: &str, root: &Path, name: String) -> Result<Document, String> {
    render_mode(text, Some(root), name, true)
}
pub fn render_draft(text: &str, source: Option<&Path>) -> Result<Document, String> {
    let root = source.and_then(Path::parent);
    render_mode(text, root, String::new(), true)
}

pub fn preserves_rendering(original: &str, formatted: &str) -> Result<bool, String> {
    Ok(render_draft(original, None)?.html == render_draft(formatted, None)?.html)
}

pub(super) fn render_mode(
    text: &str,
    root: Option<&Path>,
    name: String,
    images: bool,
) -> Result<Document, String> {
    if text.len() as u64 > MAX_DOCUMENT {
        return Err("文書サイズ上限です。".into());
    }
    let root = root
        .map(local_path)
        .transpose()
        .map_err(|_| "参照フォルダーがありません。")?;
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES;
    let (metadata, text) = if let Some(rest) = text.strip_prefix("---\n") {
        if let Some(end) = rest.find("\n---\n") {
            (Some(&rest[..end]), &rest[end + 5..])
        } else {
            (None, text)
        }
    } else {
        (None, text)
    };
    let events: Vec<_> = Parser::new_ext(text, options).collect();
    if events.len() > 100_000 {
        return Err("文書の構造が複雑すぎます。".into());
    }
    let mut output = Vec::new();
    let mut headings = Vec::new();
    let mut counts = HashMap::<String, usize>::new();
    let mut used_ids = HashSet::new();
    let mut image_budget = ImageBudget::default();
    let mut i = 0;
    while i < events.len() {
        match &events[i] {
            Event::Start(Tag::Heading { level, .. }) => {
                let title: String = events[i + 1..]
                    .iter()
                    .take_while(|e| !matches!(e, Event::End(TagEnd::Heading(_))))
                    .filter_map(|e| match e {
                        Event::Text(t) | Event::Code(t) => Some(t.as_ref()),
                        _ => None,
                    })
                    .collect();
                let base: String = title
                    .to_lowercase()
                    .chars()
                    .filter_map(|c| {
                        if c.is_alphanumeric() || c == '_' || c == '-' {
                            Some(c)
                        } else if c.is_whitespace() {
                            Some('-')
                        } else {
                            None
                        }
                    })
                    .collect();
                let count = counts.entry(base.clone()).or_default();
                let mut id = if *count == 0 {
                    base.clone()
                } else {
                    format!("{base}-{count}")
                };
                while !used_ids.insert(id.clone()) {
                    *count += 1;
                    id = format!("{base}-{count}");
                }
                *count += 1;
                headings.push(Heading {
                    id: id.clone(),
                    title,
                    level: *level as u8,
                });
                output.push(Event::Html(
                    format!("<h{} id=\"{}\">", *level as u8, escaped(&id)).into(),
                ));
            }
            Event::Html(t) | Event::InlineHtml(t) => output.push(Event::Text(t.clone())),
            Event::Start(Tag::Image { dest_url, .. }) => {
                let mut alt = String::new();
                i += 1;
                while i < events.len() && !matches!(events[i], Event::End(TagEnd::Image)) {
                    if let Event::Text(t) | Event::Code(t) = &events[i] {
                        alt.push_str(t);
                    }
                    i += 1;
                }
                if !images {
                    i += 1;
                    continue;
                }
                let markup = match root
                    .as_deref()
                    .ok_or_else(|| "保存先を選ぶと相対パスの画像を表示できます".to_string())
                    .and_then(|root| read_local_image(root, dest_url, &mut image_budget))
                {
                    Ok(LocalImage {
                        bytes,
                        mime,
                        width,
                        height,
                    }) => format!(
                        "<img src=\"data:{mime};base64,{}\" width=\"{width}\" height=\"{height}\" alt=\"{}\">",
                        STANDARD.encode(bytes),
                        escaped(&alt)
                    ),
                    Err(reason) => format!(
                        "<span class=\"image-error\">画像：{}（{}）</span>",
                        escaped(&alt),
                        escaped(&reason)
                    ),
                };
                output.push(Event::Html(markup.into()));
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                let local = !dest_url.contains(':')
                    && !dest_url.starts_with(['/', '\\'])
                    && !dest_url.starts_with('#');
                let href = if dest_url.starts_with('#') {
                    escaped(dest_url)
                } else {
                    String::new()
                };
                let attr = if dest_url.split_once(':').is_some_and(|(scheme, _)| {
                    scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https")
                }) {
                    format!(" data-external=\"{}\"", escaped(dest_url))
                } else if local {
                    format!(" data-local=\"{}\"", escaped(dest_url))
                } else {
                    String::new()
                };
                output.push(Event::Html(format!("<a href=\"{href}\"{attr}>").into()));
            }
            e => output.push(e.clone()),
        }
        i += 1;
    }
    let mut body = metadata
        .map(|m| {
            format!(
                "<details><summary>文書の情報</summary><pre>{}</pre></details>",
                escaped(m)
            )
        })
        .unwrap_or_default();
    html::push_html(&mut body, output.into_iter());
    Ok(Document {
        id: name.clone(),
        anchor: None,
        name,
        html: body,
        headings,
    })
}

#[cfg(test)]
mod formatting_tests {
    use super::preserves_rendering;

    #[test]
    fn accepts_cosmetic_whitespace_and_rejects_changed_hard_breaks_or_literal_blocks() {
        assert!(preserves_rendering("# Title   \n\n\nText", "# Title\n\nText\n").unwrap());
        assert!(!preserves_rendering("one   \ntwo", "one\ntwo\n").unwrap());
        assert!(!preserves_rendering("<pre>\na  \n</pre>\n", "<pre>\na\n</pre>\n").unwrap());
    }
}
