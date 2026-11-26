mod commands;
mod menu;

use commands::{
    browser::ExternalState,
    documents::{ActiveWatch, Inbox, Known, receive},
    session::SavedSession,
    writer::DraftWrites,
};
use md_viewer::session::SessionStore;
use std::{path::PathBuf, sync::Mutex};
use tauri::{Emitter, Manager};

#[tauri::command]
fn finish_close(app: tauri::AppHandle) {
    app.exit(0);
}

fn main() {
    let app = tauri::Builder::default()
        .manage(Inbox::default())
        .manage(Known::default())
        .manage(DraftWrites::default())
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
            commands::documents::choose_file,
            commands::documents::take_documents,
            commands::documents::open_local,
            menu::set_language,
            commands::documents::watch_document,
            commands::session::load_session,
            commands::session::save_session,
            commands::session::load_recovery_draft,
            commands::session::save_recovery_draft,
            commands::writer::validate_format,
            commands::documents::restore_document,
            commands::documents::search_document,
            finish_close,
            commands::browser::prepare_external,
            commands::browser::prepare_guide_link,
            commands::browser::cancel_external,
            commands::browser::open_external,
            commands::export::export_word,
            commands::export::print_document,
            commands::browser::open_google_docs,
            commands::updates::app_version,
            commands::updates::check_update,
            commands::browser::open_releases,
            commands::writer::read_draft,
            commands::writer::preview_draft,
            commands::writer::save_draft
        ])
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            if id == "quit" {
                let _ = app.emit("save-and-close", ());
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
            menu::set_language(app.handle().clone(), "ja".into())?;
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
