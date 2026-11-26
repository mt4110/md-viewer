use super::documents::known_path;
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub(crate) fn print_document(window: tauri::WebviewWindow) -> Result<(), String> {
    window
        .print()
        .map_err(|_| "印刷プレビューを開けませんでした。".into())
}

#[tauri::command]
pub(crate) async fn export_word(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    source: Option<String>,
    markdown: Option<String>,
) -> Result<Option<md_viewer::export::WordExport>, String> {
    let path = source
        .as_deref()
        .map(|source| known_path(&app, source))
        .transpose()?;
    let file_name = path
        .as_deref()
        .and_then(std::path::Path::file_stem)
        .map(|stem| stem.to_string_lossy())
        .unwrap_or_else(|| "Untitled".into());
    let dialog = app
        .dialog()
        .file()
        .set_parent(&window)
        .add_filter("Word", &["docx"])
        .set_file_name(format!("{file_name}.docx"));
    tauri::async_runtime::spawn_blocking(move || {
        let Some(destination) = dialog.blocking_save_file() else {
            return Ok(None);
        };
        let destination = destination
            .into_path()
            .map_err(|_| "Word文書を保存できませんでした。")?;
        let export = match markdown {
            Some(text) => md_viewer::export::save_word_contents(
                &text,
                path.as_deref().and_then(std::path::Path::parent),
                &destination,
            ),
            None => md_viewer::export::save_word(
                &path.ok_or("元の文書を確認できません。")?,
                &destination,
            ),
        }?;
        Ok(Some(export))
    })
    .await
    .map_err(|_| "Word文書を保存できませんでした。".to_string())?
}
