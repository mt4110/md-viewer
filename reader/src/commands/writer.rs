use super::documents::{Known, known_path};
use std::{path::PathBuf, sync::Mutex};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub(crate) async fn validate_format(original: String, formatted: String) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || {
        md_viewer::document::preserves_rendering(&original, &formatted)
    })
    .await
    .map_err(|_| "整形結果を確認できませんでした。".to_string())?
}

#[derive(Default)]
pub(crate) struct DraftWrites(Mutex<()>);

#[tauri::command]
pub(crate) async fn read_draft(app: tauri::AppHandle, source: String) -> Result<String, String> {
    let path = known_path(&app, &source)?;
    tauri::async_runtime::spawn_blocking(move || md_viewer::editor::read(&path))
        .await
        .map_err(|_| "Markdownを読み込めませんでした。".to_string())?
}

#[tauri::command]
pub(crate) async fn preview_draft(
    app: tauri::AppHandle,
    source: Option<String>,
    markdown: String,
) -> Result<String, String> {
    let path = source
        .as_deref()
        .map(|source| known_path(&app, source))
        .transpose()?;
    tauri::async_runtime::spawn_blocking(move || {
        md_viewer::document::render_draft(&markdown, path.as_deref()).map(|doc| doc.html)
    })
    .await
    .map_err(|_| "プレビューを更新できませんでした。".to_string())?
}

#[tauri::command]
pub(crate) async fn save_draft(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    source: Option<String>,
    markdown: String,
    expected: Option<String>,
) -> Result<Option<md_viewer::editor::SavedDraft>, String> {
    let path = source
        .as_deref()
        .map(|source| known_path(&app, source))
        .transpose()?;
    let dialog = if expected.is_none() {
        let mut dialog = app
            .dialog()
            .file()
            .set_parent(&window)
            .add_filter("Markdown", &["md", "markdown"]);
        if let Some(path) = path.as_ref() {
            if let Some(parent) = path.parent() {
                dialog = dialog.set_directory(parent);
            }
            if let Some(stem) = path.file_stem() {
                dialog = dialog.set_file_name(format!("{} copy.md", stem.to_string_lossy()));
            }
        } else {
            dialog = dialog.set_file_name("Untitled.md");
        }
        Some(dialog)
    } else {
        None
    };
    tauri::async_runtime::spawn_blocking(move || {
        let destination = if let Some(dialog) = dialog {
            let Some(file) = dialog.blocking_save_file() else {
                return Ok(None);
            };
            file.into_path()
                .map_err(|_| "Markdownを保存できませんでした。")?
        } else {
            path.ok_or("元の文書を確認できません。")?
        };
        let state = app.state::<DraftWrites>();
        let _guard = state.0.lock().unwrap();
        let saved = md_viewer::editor::save(&destination, &markdown, expected.as_deref())?;
        app.state::<Known>()
            .0
            .lock()
            .unwrap()
            .insert(saved.document.id.clone(), PathBuf::from(&saved.document.id));
        Ok(Some(saved))
    })
    .await
    .map_err(|_| "Markdownを保存できませんでした。".to_string())?
}
