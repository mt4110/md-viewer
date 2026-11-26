use md_viewer::{
    document::{Document, load, resolve_local},
    watch::WatchHub,
};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
};
use tauri::{Emitter, Manager};
use tauri_plugin_dialog::DialogExt;

#[derive(Default)]
pub(crate) struct ActiveWatch(Mutex<WatchHub>);
#[derive(Clone, serde::Serialize)]
struct Update {
    source: String,
    revision: u64,
    result: Result<Document, String>,
}
#[tauri::command]
pub(crate) fn watch_document(
    app: tauri::AppHandle,
    source: Option<String>,
    revision: u64,
) -> Result<(), String> {
    let state = app.state::<ActiveWatch>();
    let mut watcher = state.0.lock().unwrap();
    if let Some(source) = source {
        let path = app
            .state::<Known>()
            .0
            .lock()
            .unwrap()
            .get(&source)
            .cloned()
            .ok_or("元の文書を確認できません。")?;
        let handle = app.clone();
        watcher.start(path, move |result| {
            let _ = handle.emit(
                "document-updated",
                Update {
                    source: source.clone(),
                    revision,
                    result,
                },
            );
        })
    } else {
        watcher.stop();
        Ok(())
    }
}

#[derive(Default)]
pub(crate) struct Inbox(Mutex<Vec<Result<Document, String>>>, AtomicU64);
#[derive(Default)]
pub(crate) struct Known(pub(crate) Mutex<HashMap<String, PathBuf>>);

pub(crate) fn known_path(app: &tauri::AppHandle, source: &str) -> Result<PathBuf, String> {
    app.state::<Known>()
        .0
        .lock()
        .unwrap()
        .get(source)
        .cloned()
        .ok_or_else(|| "元の文書を確認できません。".into())
}

pub(crate) fn receive(app: &tauri::AppHandle, path: PathBuf) {
    receive_at(app, path, None);
}
fn receive_at(app: &tauri::AppHandle, path: PathBuf, anchor: Option<String>) {
    let generation = app.state::<Inbox>().1.fetch_add(1, Ordering::SeqCst) + 1;
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut doc = load(&path);
        if let Ok(d) = &mut doc {
            d.anchor = anchor;
        }
        let _ = generation;
        if let Ok(doc) = &doc {
            app.state::<Known>()
                .0
                .lock()
                .unwrap()
                .insert(doc.id.clone(), PathBuf::from(&doc.id));
        }
        let state = app.state::<Inbox>();
        let mut queue = state.0.lock().unwrap();
        queue.push(doc);
        drop(queue);
        let _ = app.emit("documents-ready", ());
    });
}
#[tauri::command]
pub(crate) fn take_documents(state: tauri::State<Inbox>) -> Vec<Result<Document, String>> {
    std::mem::take(&mut *state.0.lock().unwrap())
}
#[tauri::command]
pub(crate) fn choose_file(app: tauri::AppHandle) {
    app.dialog()
        .file()
        .add_filter("Markdown", &["md", "markdown"])
        .pick_file(move |file| {
            if let Some(file) = file
                && let Ok(path) = file.into_path()
            {
                receive(&app, path);
            }
        });
}
#[tauri::command]
pub(crate) fn open_local(
    app: tauri::AppHandle,
    source: String,
    target: String,
) -> Result<(), String> {
    let path = app
        .state::<Known>()
        .0
        .lock()
        .unwrap()
        .get(&source)
        .cloned()
        .ok_or("元の文書を確認できません。")?;
    let anchor = target.split_once('#').map(|(_, a)| {
        percent_encoding::percent_decode_str(a)
            .decode_utf8_lossy()
            .into_owned()
    });
    receive_at(&app, resolve_local(&path, &target)?, anchor);
    Ok(())
}
#[tauri::command]
pub(crate) async fn restore_document(
    app: tauri::AppHandle,
    source: String,
) -> Result<Document, String> {
    let path = app
        .state::<Known>()
        .0
        .lock()
        .unwrap()
        .get(&source)
        .cloned()
        .ok_or("元の文書を確認できません。")?;
    tauri::async_runtime::spawn_blocking(move || load(&path))
        .await
        .map_err(|_| "ファイルを読み込めません。".to_string())?
}
#[tauri::command]
pub(crate) async fn search_document(
    app: tauri::AppHandle,
    source: String,
) -> Result<Document, String> {
    let path = app
        .state::<Known>()
        .0
        .lock()
        .unwrap()
        .get(&source)
        .cloned()
        .ok_or("元の文書を確認できません。")?;
    tauri::async_runtime::spawn_blocking(move || md_viewer::document::load_for_search(&path))
        .await
        .map_err(|_| "ファイルを読み込めません。".to_string())?
}
