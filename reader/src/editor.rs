use crate::document::{Document, is_markdown_path, local_path, read_markdown_at, render};
use serde::Serialize;
use std::{
    fs,
    io::{ErrorKind, Write},
    path::Path,
};

const FAILED: &str = "Markdownを保存できませんでした。";
const CHANGED: &str = "ファイルが編集中に変更されました。別名で保存してください。";

#[derive(Serialize)]
pub struct SavedDraft {
    pub document: Document,
    pub original: String,
}

pub fn read(source: &Path) -> Result<String, String> {
    if local_path(source)? != source {
        return Err(CHANGED.into());
    }
    read_markdown_at(source)
}

pub fn save(destination: &Path, text: &str, expected: Option<&str>) -> Result<SavedDraft, String> {
    if text.len() as u64 > crate::document::MAX_DOCUMENT {
        return Err("文書サイズ上限です。".into());
    }
    let parent = local_path(destination.parent().ok_or(FAILED)?)?;
    let mut name = destination.file_name().ok_or(FAILED)?.to_os_string();
    if !is_markdown_path(destination) {
        name.push(".md");
    }
    let path = parent.join(&name);
    let name = name.to_string_lossy().into_owned();
    let mut original = text.to_string();
    if let Some(expected) = expected {
        if expected.contains("\r\n")
            && expected.match_indices("\r\n").count() == expected.matches('\n').count()
        {
            original = original.replace('\n', "\r\n");
        }
        if expected.starts_with('\u{feff}') {
            original.insert(0, '\u{feff}');
        }
    }
    if original.len() as u64 > crate::document::MAX_DOCUMENT {
        return Err("文書サイズ上限です。".into());
    }
    let mut document = render(original.trim_start_matches('\u{feff}'), &parent, name)?;
    document.id = path.to_string_lossy().into_owned();

    let mut temporary = tempfile::NamedTempFile::new_in(&parent).map_err(|_| FAILED)?;
    temporary
        .write_all(original.as_bytes())
        .map_err(|_| FAILED)?;
    if let Some(expected) = expected {
        if path != destination {
            return Err(CHANGED.into());
        }
        let permissions = fs::symlink_metadata(&path)
            .map_err(|_| CHANGED)?
            .permissions();
        if permissions.readonly() {
            return Err(FAILED.into());
        }
        temporary
            .as_file()
            .set_permissions(permissions)
            .map_err(|_| FAILED)?;
        temporary.as_file().sync_all().map_err(|_| FAILED)?;
        if !read(&path).is_ok_and(|current| current == expected) {
            return Err(CHANGED.into());
        }
        temporary.persist(&path).map_err(|_| FAILED)?;
    } else {
        temporary.as_file().sync_all().map_err(|_| FAILED)?;
        temporary.persist_noclobber(&path).map_err(|error| {
            if error.error.kind() == ErrorKind::AlreadyExists {
                "同名のファイルがあります。別の名前を選んでください。"
            } else {
                FAILED
            }
        })?;
    }
    Ok(SavedDraft { document, original })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_markdown_and_refuses_to_replace_a_name_when_saving_a_copy() {
        let temp = tempfile::tempdir().unwrap();
        let destination = temp.path().join("議事録");
        let markdown = "# 議事録\n\n| 項目 | 内容 |\n| --- | --- |\n| 決定 | 保留 |\n";
        let saved = save(&destination, markdown, None).unwrap();
        let path = Path::new(&saved.document.id);
        assert_eq!(path.extension().unwrap(), "md");
        assert_eq!(fs::read_to_string(path).unwrap(), markdown);
        assert!(saved.document.html.contains("<table>"));
        assert!(save(&destination, "different", None).is_err());
        assert_eq!(fs::read_to_string(path).unwrap(), markdown);
    }

    #[test]
    fn refuses_external_changes_and_preserves_crlf_and_bom_on_normal_save() {
        let temp = tempfile::tempdir().unwrap();
        let path = local_path(temp.path()).unwrap().join("notes.markdown");
        let original = "\u{feff}# Before\r\n";
        fs::write(&path, original).unwrap();
        let saved = save(&path, "# After\n\nParagraph\n", Some(original)).unwrap();
        assert_eq!(saved.original, "\u{feff}# After\r\n\r\nParagraph\r\n");
        assert_eq!(read(&path).unwrap(), saved.original);
        fs::write(&path, "# Updated externally\n").unwrap();
        assert_eq!(
            save(&path, "# Draft\n", Some(&saved.original))
                .err()
                .as_deref(),
            Some(CHANGED),
        );
        assert_eq!(read(&path).unwrap(), "# Updated externally\n");
    }
}
