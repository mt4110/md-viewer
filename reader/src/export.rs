use crate::document::{ImageBudget, read_local_image, read_markdown};
use md2docx::{Config, Severity};
use serde::Serialize;
use std::{
    ffi::OsString,
    io::{ErrorKind, Write},
    path::Path,
    sync::{Arc, Mutex},
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WordExport {
    file_name: String,
    has_warnings: bool,
}

pub fn save_word(source: &Path, destination: &Path) -> Result<WordExport, String> {
    let (source, markdown) = read_markdown(source)?;
    save_word_contents(&markdown, source.parent(), destination)
}

pub fn save_word_contents(
    markdown: &str,
    root: Option<&Path>,
    destination: &Path,
) -> Result<WordExport, String> {
    let failed = "Word文書を保存できませんでした。";
    let mut name = OsString::from(destination.file_name().ok_or(failed)?);
    if !destination
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("docx"))
    {
        name.push(".docx");
    }
    let parent = destination.parent().ok_or(failed)?;
    let destination = parent.join(&name);
    let exists = "同名のファイルがあります。別の名前を選んでください。";
    if destination.symlink_metadata().is_ok() {
        return Err(exists.into());
    }

    let root = root.map(crate::document::local_path).transpose()?;
    let images = Mutex::new(ImageBudget::default());
    let config = Config::untrusted().with_image_resolver(move |reference: &str| {
        Ok(root
            .as_deref()
            .and_then(|root| read_local_image(root, reference, &mut images.lock().unwrap()).ok())
            .map(|image| Arc::from(image.bytes)))
    });
    let conversion = md2docx::convert(markdown.trim_start_matches('\u{feff}'), &config)
        .map_err(|_| "Word文書へ変換できませんでした。")?;

    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|_| failed)?;
    temporary.write_all(&conversion.docx).map_err(|_| failed)?;
    temporary.as_file().sync_all().map_err(|_| failed)?;
    temporary.persist_noclobber(destination).map_err(|error| {
        if error.error.kind() == ErrorKind::AlreadyExists {
            exists
        } else {
            failed
        }
    })?;
    Ok(WordExport {
        file_name: name.to_string_lossy().into_owned(),
        has_warnings: conversion
            .diagnostics
            .iter()
            .any(|finding| finding.severity == Severity::Warning),
    })
}
