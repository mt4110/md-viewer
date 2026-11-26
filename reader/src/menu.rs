#[tauri::command]
pub(crate) fn set_language(app: tauri::AppHandle, language: String) -> Result<(), String> {
    use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};
    let en = language == "en";
    let labels = if en {
        [
            "File",
            "Open file…",
            "Close tab",
            "Edit",
            "Copy",
            "Paste",
            "Select all",
            "View",
            "Settings…",
            "Quit md-viewer",
        ]
    } else {
        [
            "ファイル",
            "ファイルを開く…",
            "タブを閉じる",
            "編集",
            "コピー",
            "ペースト",
            "すべて選択",
            "表示",
            "設定…",
            "md-viewerを終了",
        ]
    };
    let quit = MenuItemBuilder::with_id("quit", labels[9])
        .accelerator("CmdOrCtrl+Q")
        .build(&app)
        .map_err(|e| e.to_string())?;
    let application = SubmenuBuilder::new(&app, "md-viewer")
        .item(&quit)
        .build()
        .map_err(|e| e.to_string())?;
    let file = SubmenuBuilder::new(&app, labels[0])
        .text(
            "new-draft",
            if en {
                "New Markdown…"
            } else {
                "Markdownを新規作成…"
            },
        )
        .text("open", labels[1])
        .text("export", if en { "Export…" } else { "書き出す…" })
        .text("close", labels[2])
        .build()
        .map_err(|e| e.to_string())?;
    let edit = SubmenuBuilder::new(&app, labels[3])
        .copy_with_text(labels[4])
        .paste_with_text(labels[5])
        .select_all_with_text(labels[6])
        .build()
        .map_err(|e| e.to_string())?;
    let view = SubmenuBuilder::new(&app, labels[7])
        .text("settings", labels[8])
        .separator()
        .text("ja", "日本語")
        .text("en", "English")
        .build()
        .map_err(|e| e.to_string())?;
    let menu = MenuBuilder::new(&app)
        .items(&[&application, &file, &edit, &view])
        .build()
        .map_err(|e| e.to_string())?;
    app.set_menu(menu).map_err(|e| e.to_string())?;
    Ok(())
}
