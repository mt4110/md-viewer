mod images;
mod render;

pub(crate) use images::{ImageBudget, read_local_image};
use render::render_mode;
pub use render::{preserves_rendering, render, render_draft};
use serde::Serialize;
use std::{fs, io::Read, path::Path};

pub const MAX_DOCUMENT: u64 = 2 * 1024 * 1024;
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
    let (path, text) = read_markdown(path)?;
    let mut document = render_mode(
        text.trim_start_matches('\u{feff}'),
        Some(path.parent().ok_or("参照フォルダーがありません。")?),
        path.file_name().unwrap().to_string_lossy().into_owned(),
        images,
    )?;
    document.id = path.to_string_lossy().into_owned();
    Ok(document)
}

pub(crate) fn read_markdown(path: &Path) -> Result<(std::path::PathBuf, String), String> {
    let path = local_path(path)?;
    let text = read_markdown_at(&path)?;
    Ok((path, text))
}

pub(crate) fn read_markdown_at(path: &Path) -> Result<String, String> {
    if !is_markdown_path(path) {
        return Err("Markdownファイルを選んでください。".into());
    }
    let bytes = read_bounded(path, MAX_DOCUMENT)?;
    String::from_utf8(bytes).map_err(|_| "UTF-8の文書を選んでください。".into())
}

pub(crate) fn is_markdown_path(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|s| s.to_str())
            .map(str::to_lowercase)
            .as_deref(),
        Some("md" | "markdown")
    )
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
    let file = local_path(&root.join(target)).map_err(|_| "リンク切れです。")?;
    if !file.starts_with(&root) {
        return Err("親フォルダー外の文書です。「ファイルを開く」から選んでください。".into());
    }
    Ok(file)
}
