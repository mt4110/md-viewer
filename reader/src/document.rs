use base64::{Engine, engine::general_purpose::STANDARD};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, html};
use serde::Serialize;
use std::{collections::HashMap, fs, io::Read, path::Path};

pub const MAX_DOCUMENT: u64 = 2 * 1024 * 1024;
const MAX_IMAGE: u64 = 8 * 1024 * 1024;
const MAX_ASSETS: usize = 24 * 1024 * 1024;
#[derive(Clone, Serialize)]
pub struct Heading {
    pub id: String,
    pub title: String,
    pub level: u8,
}
#[derive(Clone, Serialize)]
pub struct Document {
    pub id: String,
    pub anchor: Option<String>,
    pub name: String,
    pub html: String,
    pub headings: Vec<Heading>,
}
/// Reject unsupported Windows paths before canonicalization or opening.
pub fn local_path(path: &Path) -> Result<std::path::PathBuf, String> {
    #[cfg(windows)]
    let absolute = std::path::absolute(path).map_err(|_| "パスを確認できません。")?;
    #[cfg(windows)]
    let path = absolute.as_path();
    #[cfg(windows)]
    {
        use std::{
            os::windows::fs::MetadataExt,
            path::{Component, Prefix},
        };
        if !matches!(path.components().next(), Some(Component::Prefix(p)) if matches!(p.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_)))
        {
            return Err("ローカルドライブのファイルを選んでください。".into());
        }
        let mut current = std::path::PathBuf::new();
        for part in path.components() {
            current.push(part.as_os_str());
            if matches!(part, Component::Prefix(_)) {
                continue;
            }
            if std::fs::symlink_metadata(&current).is_ok_and(|m| m.file_attributes() & 0x400 != 0) {
                return Err("再解析ポイントを含むパスは開けません。".into());
            }
        }
    }
    path.canonicalize()
        .map_err(|_| "ファイルが見つかりません。".into())
}
fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    if !fs::symlink_metadata(path)
        .map_err(|_| "ファイル情報を読めません。")?
        .is_file()
    {
        return Err("通常のファイルを選んでください。".into());
    }
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x00200000); // FILE_FLAG_OPEN_REPARSE_POINT
    }
    let file = options
        .open(path)
        .map_err(|_| "ファイルを開けません。場所とアクセス権を確認してください。")?;
    if !file
        .metadata()
        .map_err(|_| "ファイル情報を読めません。")?
        .is_file()
    {
        return Err("通常のファイルを選んでください。".into());
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if file
            .metadata()
            .map_err(|_| "ファイル情報を読めません。")?
            .file_attributes()
            & 0x400
            != 0
        {
            return Err("再解析ポイントは開けません。".into());
        }
    }
    let mut bytes = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "ファイルを読み込めません。")?;
    if bytes.len() as u64 > limit {
        return Err("表示できるファイルサイズを超えています。".into());
    }
    Ok(bytes)
}
pub fn load(path: &Path) -> Result<Document, String> {
    load_mode(path, true)
}
/// Search restored tabs without reading or decoding image assets.
pub fn load_for_search(path: &Path) -> Result<Document, String> {
    load_mode(path, false)
}
fn load_mode(path: &Path, images: bool) -> Result<Document, String> {
    let path = local_path(path)?;
    if !matches!(
        path.extension()
            .and_then(|s| s.to_str())
            .map(str::to_lowercase)
            .as_deref(),
        Some("md" | "markdown")
    ) {
        return Err("Markdownファイルを選んでください。".into());
    }
    let bytes = read_bounded(&path, MAX_DOCUMENT)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| "UTF-8の文書を選んでください。")?
        .trim_start_matches('\u{feff}');
    let mut document = render_mode(
        text,
        path.parent().ok_or("参照フォルダーがありません。")?,
        path.file_name().unwrap().to_string_lossy().into_owned(),
        images,
    )?;
    document.id = path.to_string_lossy().into_owned();
    Ok(document)
}
fn escaped(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
fn image_url(
    root: &Path,
    dest: &str,
    total: &mut usize,
    pixels: &mut u64,
    count: &mut usize,
) -> Result<(String, u32, u32), String> {
    let decoded = percent_encoding::percent_decode_str(dest)
        .decode_utf8()
        .map_err(|_| "画像パスが不正です")?;
    let dest = decoded.as_ref();
    if dest.contains(':') || dest.starts_with(['/', '\\']) {
        return Err("外部画像・絶対パスは読み込みません".into());
    }
    if *count >= 64 {
        return Err("画像の個数上限です".into());
    }
    *count += 1;
    let file = local_path(&root.join(dest)).map_err(|_| "画像が見つかりません")?;
    if !file.starts_with(root) {
        return Err("参照フォルダー外の画像です".into());
    }
    let mime = match file
        .extension()
        .and_then(|s| s.to_str())
        .map(str::to_lowercase)
        .as_deref()
    {
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        _ => return Err("未対応の画像形式です".into()),
    };
    let bytes = read_bounded(&file, MAX_IMAGE)?;
    if *total + bytes.len() > MAX_ASSETS {
        return Err("画像の合計サイズ上限です".into());
    }
    let reader = image::ImageReader::new(std::io::Cursor::new(&bytes))
        .with_guessed_format()
        .map_err(|_| "画像を確認できません")?;
    let (w, h) = reader.into_dimensions().map_err(|_| "画像が壊れています")?;
    if u64::from(w) * u64::from(h) > 20_000_000 {
        return Err("画像の画素数上限です".into());
    }
    let area = u64::from(w) * u64::from(h);
    if *pixels + area > 40_000_000 {
        return Err("画像の合計画素数上限です".into());
    }
    *pixels += area;
    *total += bytes.len();
    Ok((
        format!("data:{mime};base64,{}", STANDARD.encode(bytes)),
        w,
        h,
    ))
}
pub fn render(text: &str, root: &Path, name: String) -> Result<Document, String> {
    render_mode(text, root, name, true)
}
fn render_mode(text: &str, root: &Path, name: String, images: bool) -> Result<Document, String> {
    if text.len() as u64 > MAX_DOCUMENT {
        return Err("文書サイズ上限です。".into());
    }
    let root = local_path(root).map_err(|_| "参照フォルダーがありません。")?;
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
    let mut total = 0;
    let mut pixels = 0;
    let mut image_count = 0;
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
                let id = if *count == 0 {
                    base
                } else {
                    format!("{base}-{count}")
                };
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
                let markup =
                    match image_url(&root, dest_url, &mut total, &mut pixels, &mut image_count) {
                        Ok((url, w, h)) => format!(
                            "<img src=\"{url}\" width=\"{w}\" height=\"{h}\" alt=\"{}\">",
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

pub fn resolve_local(source: &Path, target: &str) -> Result<std::path::PathBuf, String> {
    let decoded = percent_encoding::percent_decode_str(target.split('#').next().unwrap_or(""))
        .decode_utf8()
        .map_err(|_| "リンクの文字コードが不正です。")?;
    let target = decoded.as_ref();
    if target.contains(':') || target.starts_with(['/', '\\']) {
        return Err("このリンク形式にはまだ対応していません。".into());
    }
    let root = local_path(source.parent().ok_or("参照フォルダーがありません。")?)?;
    let file = local_path(&root.join(target.split('#').next().unwrap_or("")))
        .map_err(|_| "リンク切れです。")?;
    if !file.starts_with(&root) {
        return Err("親フォルダー外の文書です。「ファイルを開く」から選んでください。".into());
    }
    Ok(file)
}
