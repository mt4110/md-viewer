use md_viewer::document::{Document, load, resolve_local};
use md_viewer::session::{LoadedSession, Session, SessionStore};
use md_viewer::watch::WatchHub;
struct SavedSession(Mutex<SessionStore>);
#[tauri::command]
async fn load_session(app: tauri::AppHandle) -> Result<LoadedSession, String> {
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
async fn save_session(app: tauri::AppHandle, session: Session) -> Result<(), String> {
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
#[tauri::command]
async fn restore_document(app: tauri::AppHandle, source: String) -> Result<Document, String> {
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
async fn search_document(app: tauri::AppHandle, source: String) -> Result<Document, String> {
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
struct ExternalState(Mutex<md_viewer::external::ExternalLinks>);
#[tauri::command]
fn prepare_external(
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
fn cancel_external(app: tauri::AppHandle, ticket: String) {
    app.state::<ExternalState>()
        .0
        .lock()
        .unwrap()
        .cancel(&ticket);
}
#[tauri::command]
async fn open_external(app: tauri::AppHandle, ticket: String) -> Result<(), String> {
    // Take and consume the reviewed URL before handing it to any OS launcher.
    let url = app
        .state::<ExternalState>()
        .0
        .lock()
        .unwrap()
        .take_confirmed(&ticket)?;
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
fn finish_close(app: tauri::AppHandle) {
    app.exit(0);
}

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
};
use tauri::{Emitter, Manager};
#[derive(Default)]
struct ActiveWatch(Mutex<WatchHub>);
#[derive(Clone, serde::Serialize)]
struct Update {
    source: String,
    revision: u64,
    result: Result<Document, String>,
}
#[tauri::command]
fn watch_document(
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

use tauri_plugin_dialog::DialogExt;
#[derive(Default)]
struct Inbox(Mutex<Vec<Result<Document, String>>>, AtomicU64);
#[derive(Default)]
struct Known(Mutex<HashMap<String, PathBuf>>);
fn receive(app: &tauri::AppHandle, path: PathBuf) {
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
fn take_documents(state: tauri::State<Inbox>) -> Vec<Result<Document, String>> {
    std::mem::take(&mut *state.0.lock().unwrap())
}
#[tauri::command]
fn choose_file(app: tauri::AppHandle) {
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
fn open_local(app: tauri::AppHandle, source: String, target: String) -> Result<(), String> {
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
fn set_language(app: tauri::AppHandle, language: String) -> Result<(), String> {
    use tauri::menu::{MenuBuilder, SubmenuBuilder};
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
            "Appearance…",
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
            "表示設定…",
            "md-viewerを終了",
        ]
    };
    let application = SubmenuBuilder::new(&app, "md-viewer")
        .quit_with_text(labels[9])
        .build()
        .map_err(|e| e.to_string())?;
    let file = SubmenuBuilder::new(&app, labels[0])
        .text("open", labels[1])
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

fn main() {
    let app = tauri::Builder::default()
        .manage(Inbox::default())
        .manage(Known::default())
        .manage(ExternalState(Mutex::new(Default::default())))
        .manage(ActiveWatch::default())
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            for arg in args.into_iter().skip(1).filter(|a| !a.starts_with('-')) {
                receive(app, PathBuf::from(&cwd).join(arg));
            }
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            choose_file,
            take_documents,
            open_local,
            set_language,
            watch_document,
            load_session,
            save_session,
            restore_document,
            search_document,
            finish_close,
            prepare_external,
            cancel_external,
            open_external
        ])
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            if id == "quit" {
                app.exit(0);
            } else {
                let _ = app.emit("menu-action", id);
            }
        })
        .setup(|app| {
            let directory = app.path().app_data_dir()?;
            #[cfg(debug_assertions)]
            let directory = std::env::var_os("MD_VIEWER_STATE_DIR")
                .map(PathBuf::from)
                .unwrap_or(directory);
            app.manage(SavedSession(Mutex::new(SessionStore::new(&directory))));
            set_language(app.handle().clone(), "ja".into())?;
            for arg in std::env::args_os().skip(1) {
                receive(app.handle(), PathBuf::from(arg));
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.emit("save-and-close", ());
            }
            if let tauri::WindowEvent::DragDrop(tauri::DragDropEvent::Drop { paths, .. }) = event {
                for path in paths {
                    receive(window.app_handle(), path.clone());
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("アプリを起動できません");
    app.run(|app, event| {
        if let tauri::RunEvent::ExitRequested {
            api, code: None, ..
        } = &event
        {
            api.prevent_exit();
            let _ = app.emit("save-and-close", ());
        }
        #[cfg(not(target_os = "macos"))]
        let _ = (app, event);
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Opened { urls } = event {
            for url in urls {
                if let Ok(path) = url.to_file_path() {
                    receive(app, path);
                }
            }
        }
    });
}
