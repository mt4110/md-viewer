use md_viewer::session::{ReadingAnchor, Session, SessionStore, Tab};
use std::{fs, path::PathBuf};
fn folder() -> PathBuf {
    // Atomic unique allocation avoids timestamp collisions between parallel tests.
    // Keep the directory so failure evidence is preserved.
    tempfile::Builder::new()
        .prefix("md-session-")
        .tempdir()
        .unwrap()
        .keep()
}
fn state(root: &std::path::Path) -> Session {
    Session {
        version: 1,
        restore_enabled: true,
        selected: Some(root.join("b.md").to_string_lossy().into_owned()),
        tabs: vec![
            Tab {
                path: root.join("a.md").to_string_lossy().into_owned(),
                anchor: Some(ReadingAnchor {
                    block_hash: "0123456789abcdef".into(),
                    heading_hash: None,
                    occurrence: 0,
                    count: 1,
                    top: -4.5,
                    scroll: 900.0,
                }),
            },
            Tab {
                path: root.join("b.md").to_string_lossy().into_owned(),
                anchor: None,
            },
        ],
    }
}
#[test]
fn roundtrip_preserves_order_selection_and_anchor_without_opening_files() {
    let root = folder();
    let store = SessionStore::new(&root);
    let s = state(&root);
    store.save(&s).unwrap();
    assert_eq!(store.load().unwrap().session, s);
    let mut changed = s.clone();
    changed.tabs.reverse();
    store.save(&changed).unwrap();
    assert_eq!(store.load().unwrap().session, changed);
}
#[test]
fn disabled_restore_does_not_persist_paths() {
    let root = folder();
    let store = SessionStore::new(&root);
    let mut s = state(&root);
    s.restore_enabled = false;
    store.save(&s).unwrap();
    let loaded = store.load().unwrap().session;
    assert!(!loaded.restore_enabled);
    assert!(loaded.tabs.is_empty());
    assert!(
        !fs::read_to_string(root.join("session.json"))
            .unwrap()
            .contains("a.md")
    );
}
#[test]
fn malformed_state_is_preserved_and_defaults_are_usable() {
    let root = folder();
    let file = root.join("session.json");
    fs::write(&file, "{broken").unwrap();
    let loaded = SessionStore::new(&root).load().unwrap();
    assert!(loaded.warning.is_some());
    assert!(loaded.session.tabs.is_empty());
    assert_eq!(fs::read_to_string(file).unwrap(), "{broken");
    let backups: Vec<_> = fs::read_dir(root)
        .unwrap()
        .map(|x| x.unwrap().path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("session-invalid-")
        })
        .collect();
    assert_eq!(backups.len(), 1);
    assert_eq!(fs::read_to_string(&backups[0]).unwrap(), "{broken");
}
#[test]
fn invalid_save_preserves_last_valid_state() {
    let root = folder();
    let store = SessionStore::new(&root);
    let s = state(&root);
    store.save(&s).unwrap();
    let before = fs::read(root.join("session.json")).unwrap();
    let mut bad = s.clone();
    bad.tabs.push(bad.tabs[0].clone());
    assert!(store.save(&bad).is_err());
    bad = s.clone();
    bad.version = 999;
    assert!(store.save(&bad).is_err());
    bad = s.clone();
    bad.tabs[0].path = "relative.md".into();
    assert!(store.save(&bad).is_err());
    assert_eq!(before, fs::read(root.join("session.json")).unwrap());
}
#[test]
fn oversized_and_failed_storage_do_not_destroy_evidence() {
    let root = folder();
    let file = root.join("session.json");
    fs::write(&file, vec![b' '; 1024 * 1024 + 1]).unwrap();
    let store = SessionStore::new(&root);
    assert!(store.load().is_err());
    assert!(store.save(&Session::default()).is_err());
    assert_eq!(fs::metadata(file).unwrap().len(), 1024 * 1024 + 1);
    let parent = root.join("not-a-folder");
    fs::write(&parent, "preserve").unwrap();
    assert!(
        SessionStore::new(&parent)
            .save(&Session::default())
            .is_err()
    );
    assert_eq!(fs::read_to_string(parent).unwrap(), "preserve");
}
