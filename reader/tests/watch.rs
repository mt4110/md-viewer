use md_viewer::{document::Document, watch::WatchHub};
use std::{
    fs,
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant},
};
fn fixture() -> PathBuf {
    let root = tempfile::Builder::new()
        .prefix("md-watch-")
        .tempdir()
        .unwrap()
        .keep();
    let path = root.join("日本語 空白.md");
    fs::write(&path, "# 初期\n\n本文").unwrap();
    path.canonicalize().unwrap()
}
fn until(
    rx: &mpsc::Receiver<Result<Document, String>>,
    predicate: impl Fn(&Result<Document, String>) -> bool,
) -> Result<Document, String> {
    let start = Instant::now();
    loop {
        let result = rx
            .recv_timeout(Duration::from_secs(10) - start.elapsed())
            .expect("watch notification");
        if predicate(&result) {
            return result;
        }
    }
}
#[test]
fn real_os_watch_handles_overwrite_replace_and_invalid_then_recovery() {
    let path = fixture();
    let (tx, rx) = mpsc::channel();
    let mut hub = WatchHub::default();
    hub.start(path.clone(), move |d| {
        tx.send(d).unwrap();
    })
    .unwrap();
    rx.recv_timeout(Duration::from_secs(5)).unwrap().unwrap();
    fs::write(&path, "# 上書き\n\n変更後").unwrap();
    until(&rx, |d| d.as_ref().is_ok_and(|d| d.html.contains("上書き"))).unwrap();
    let replacement = path.with_file_name("editor-save.tmp");
    fs::write(&replacement, "# 置換\n\n新しいファイル").unwrap();
    fs::rename(&replacement, &path).unwrap();
    until(&rx, |d| d.as_ref().is_ok_and(|d| d.html.contains("置換"))).unwrap();
    fs::write(&path, [255]).unwrap();
    assert!(until(&rx, Result::is_err).is_err());
    fs::write(&path, "# 復旧").unwrap();
    until(&rx, |d| d.as_ref().is_ok_and(|d| d.html.contains("復旧"))).unwrap();
    hub.stop();
    while rx.try_recv().is_ok() {}
    fs::write(&path, "# 停止後").unwrap();
    assert!(rx.recv_timeout(Duration::from_millis(500)).is_err());
}
#[test]
fn switching_and_burst_saves_use_latest_file() {
    let a = fixture();
    let b = fixture();
    let (tx, rx) = mpsc::channel();
    let mut hub = WatchHub::default();
    let first = tx.clone();
    hub.start(a.clone(), move |d| {
        let _ = first.send(d);
    })
    .unwrap();
    rx.recv_timeout(Duration::from_secs(5)).unwrap().unwrap();
    hub.start(b.clone(), move |d| {
        let _ = tx.send(d);
    })
    .unwrap();
    rx.recv_timeout(Duration::from_secs(5)).unwrap().unwrap();
    fs::write(a, "# 背景").unwrap();
    for n in 0..12 {
        fs::write(&b, format!("# 連続{n}")).unwrap();
    }
    let final_doc = until(&rx, |d| d.as_ref().is_ok_and(|d| d.html.contains("連続11"))).unwrap();
    assert_eq!(final_doc.id, b.to_string_lossy());
}
