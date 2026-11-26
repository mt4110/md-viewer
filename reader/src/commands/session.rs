use super::documents::Known;
use md_viewer::session::{LoadedSession, Session, SessionStore};
use std::{path::PathBuf, sync::Mutex};
use tauri::Manager;

pub(crate) struct SavedSession(pub(crate) Mutex<SessionStore>);
#[tauri::command]
pub(crate) async fn load_recovery_draft(app: tauri::AppHandle) -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<SavedSession>().0.lock().unwrap().load_draft()
    })
    .await
    .map_err(|_| "下書きを復元できませんでした。".to_string())?
}

#[tauri::command]
pub(crate) async fn save_recovery_draft(
    app: tauri::AppHandle,
    markdown: Option<String>,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<SavedSession>()
            .0
            .lock()
            .unwrap()
            .save_draft(markdown.as_deref())
    })
    .await
    .map_err(|_| "下書きを自動保存できませんでした。".to_string())?
}
#[tauri::command]
pub(crate) async fn load_session(app: tauri::AppHandle) -> Result<LoadedSession, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let loaded = app.state::<SavedSession>().0.lock().unwrap().load()?;
        let known = app.state::<Known>();
        let mut paths = known.0.lock().unwrap();
        for tab in &loaded.session.tabs {
            paths.insert(tab.path.clone(), PathBuf::from(&tab.path));
        }
        Ok(loaded)
    })
    .await
    .map_err(|_| "復元情報を読み込めません。".to_string())?
}
#[tauri::command]
pub(crate) async fn save_session(app: tauri::AppHandle, session: Session) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        {
            let known = app.state::<Known>();
            let paths = known.0.lock().unwrap();
            if session.tabs.iter().any(|t| !paths.contains_key(&t.path)) {
                return Err("開いたことのない文書は保存できません。".into());
            }
        }
        app.state::<SavedSession>().0.lock().unwrap().save(&session)
    })
    .await
    .map_err(|_| "復元情報を保存できません。".to_string())?
}
