use md_viewer::document::{load, render};
use std::{fs, path::Path};

#[cfg(unix)]
#[test]
fn fifo_documents_and_images_are_rejected_without_waiting() {
    let dir = tempfile::tempdir().unwrap().keep();
    for name in ["pipe.md", "pipe.png"] {
        assert!(
            std::process::Command::new("mkfifo")
                .arg(dir.join(name))
                .status()
                .unwrap()
                .success()
        );
    }
    let root = dir.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        assert!(load(&root.join("pipe.md")).is_err());
        let doc = render("![x](pipe.png)", &root, "test".into()).unwrap();
        tx.send(doc.html).unwrap();
    });
    let html = rx
        .recv_timeout(std::time::Duration::from_secs(2))
        .expect("special file read must not block");
    assert!(html.contains("通常のファイル"));
    fs::write(dir.join("regular.md"), "# Works").unwrap();
    assert!(load(&dir.join("regular.md")).is_ok());
}
#[test]
fn image_count_and_aggregate_pixels_are_bounded() {
    let dir = tempfile::tempdir().unwrap();
    for (size, expected) in [(1, 64), (1000, 40)] {
        image::RgbaImage::new(size, size)
            .save(dir.path().join("test.png"))
            .unwrap();
        let text = std::iter::repeat_n("![x](test.png)", 100)
            .collect::<Vec<_>>()
            .join("\n\n");
        let doc = render(&text, dir.path(), "test".into()).unwrap();
        assert_eq!(doc.html.matches("<img ").count(), expected);
        assert!(doc.html.contains("上限"));
        assert_eq!(doc.html.matches("image-error").count(), 100 - expected);
    }
}
#[cfg(windows)]
#[test]
fn network_and_device_paths_are_rejected_before_access() {
    for value in [
        r"\\server.invalid\share\a.md",
        r"\\?\UNC\server.invalid\share\a.md",
        r"\\.\pipe\a.md",
        r"\\?\GLOBALROOT\Device\a.md",
    ] {
        assert!(md_viewer::document::local_path(Path::new(value)).is_err());
    }
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("local.md"), "# Local").unwrap();
    assert!(load(&dir.path().join("local.md")).is_ok());
}
#[test]
fn ordinary_markdown_still_renders() {
    assert!(
        render("# OK", Path::new("."), "test".into())
            .unwrap()
            .html
            .contains("<h1")
    );
}
