use super::documents::Known;
use std::sync::Mutex;
use tauri::Manager;

pub(crate) struct ExternalState(pub(crate) Mutex<md_viewer::external::ExternalLinks>);
#[tauri::command]
pub(crate) fn prepare_external(
    app: tauri::AppHandle,
    source: String,
    url: String,
) -> Result<md_viewer::external::Review, String> {
    if !app.state::<Known>().0.lock().unwrap().contains_key(&source) {
        return Err("元の文書を確認できません。".into());
    }
    app.state::<ExternalState>().0.lock().unwrap().prepare(&url)
}

#[tauri::command]
pub(crate) fn prepare_guide_link(
    app: tauri::AppHandle,
    page: &str,
    language: &str,
) -> Result<md_viewer::external::Review, String> {
    let locale = if language == "ja" { "ja" } else { "en" };
    let url = match page {
        "tutorial" => "https://commonmark.org/help/tutorial/".into(),
        "reference" => "https://commonmark.org/help/".into(),
        "commonmark" => "https://spec.commonmark.org/current/".into(),
        "gfm" => "https://github.github.com/gfm/".into(),
        "github" => format!(
            "https://docs.github.com/{locale}/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax"
        ),
        "tables" => format!(
            "https://docs.github.com/{locale}/get-started/writing-on-github/working-with-advanced-formatting/organizing-information-with-tables"
        ),
        "code" => format!(
            "https://docs.github.com/{locale}/get-started/writing-on-github/working-with-advanced-formatting/creating-and-highlighting-code-blocks"
        ),
        _ => return Err("書き方の資料が見つかりません。".into()),
    };
    app.state::<ExternalState>().0.lock().unwrap().prepare(&url)
}
#[tauri::command]
pub(crate) fn cancel_external(app: tauri::AppHandle, ticket: String) {
    app.state::<ExternalState>()
        .0
        .lock()
        .unwrap()
        .cancel(&ticket);
}
#[tauri::command]
pub(crate) async fn open_external(app: tauri::AppHandle, ticket: String) -> Result<(), String> {
    // Take and consume the reviewed URL before handing it to any OS launcher.
    let url = app
        .state::<ExternalState>()
        .0
        .lock()
        .unwrap()
        .take_confirmed(&ticket)?;
    launch_browser(url).await
}

async fn launch_browser(url: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        #[cfg(windows)]
        let result = open::that_detached(&url); // ShellExecuteExW; never cmd /c start.
        #[cfg(not(windows))]
        let result = open::that(&url);
        result.map_err(|_| {
            "標準ブラウザを開けませんでした。設定を確認してリンクを押し直してください。".to_string()
        })
    })
    .await
    .map_err(|_| {
        "標準ブラウザを開けませんでした。設定を確認してリンクを押し直してください。".to_string()
    })?
}

#[tauri::command]
pub(crate) async fn open_google_docs() -> Result<(), String> {
    launch_browser("https://docs.google.com/document/".into()).await
}

#[tauri::command]
pub(crate) async fn open_releases() -> Result<(), String> {
    launch_browser(md_viewer::update::RELEASES_PAGE.into()).await
}
