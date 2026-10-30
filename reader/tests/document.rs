use md_viewer::document::{MAX_DOCUMENT, load, render};
use std::path::Path;
#[test]
fn japanese_structure_and_duplicate_anchors() {
    let d = render(
        "# 日本語\n\n# 日本語\n\n| 列 | 値 |\n|---|---|\n| a | b |\n\n- [x] 完了",
        Path::new("."),
        "test".into(),
    )
    .unwrap();
    assert_eq!(d.headings[0].id, "日本語");
    assert_eq!(d.headings[1].id, "日本語-1");
    assert!(d.html.contains("<table>"));
    assert!(d.html.contains("disabled"));
}

#[test]
fn heading_ids_remain_unique_when_titles_include_generated_suffixes() {
    for (text, expected) in [
        (
            "# foo\n\n# foo-1\n\n# foo\n\n# foo-2\n\n# foo\n\n# foo-1",
            vec!["foo", "foo-1", "foo-2", "foo-2-1", "foo-3", "foo-1-1"],
        ),
        (
            "# foo\n\n# foo\n\n# foo-1\n\n# foo\n\n# foo-1",
            vec!["foo", "foo-1", "foo-1-1", "foo-2", "foo-1-2"],
        ),
        (
            "# 見出し\n\n# 見出し-1\n\n# 見出し",
            vec!["見出し", "見出し-1", "見出し-2"],
        ),
    ] {
        let doc = render(text, Path::new("."), "test".into()).unwrap();
        let ids: Vec<_> = doc.headings.iter().map(|h| h.id.as_str()).collect();
        assert_eq!(ids, expected);
        for id in ids {
            assert_eq!(doc.html.matches(&format!("id=\"{id}\"")).count(), 1);
        }
    }
}

#[test]
fn encoded_hash_in_local_filename_is_distinct_from_fragment() {
    use md_viewer::document::resolve_local;
    let root = tempfile::tempdir().unwrap().keep();
    let source = root.join("入口.md");
    std::fs::write(&source, "# Main").unwrap();
    for name in ["notes#1.md", "notes.md#1.md", "notes%231.md"] {
        std::fs::write(root.join(name), "# Target").unwrap();
    }
    // A literal fragment must still select notes.md, while %23 belongs to the filename.
    std::fs::write(root.join("notes.md"), "# Different document").unwrap();
    for (target, expected) in [
        ("notes%231.md", "notes#1.md"),
        ("notes%231.md#target", "notes#1.md"),
        ("notes.md%231.md#target", "notes.md#1.md"),
        ("notes%25231.md#target", "notes%231.md"),
        ("notes.md#target", "notes.md"),
    ] {
        assert_eq!(
            resolve_local(&source, target).unwrap(),
            root.join(expected).canonicalize().unwrap(),
            "target: {target}"
        );
    }
    assert!(resolve_local(&source, "%2e%2e/outside%231.md#target").is_err());
}
#[test]
fn untrusted_content_never_executes() {
    let d=render("<script>alert(1)</script>\n\n[x](javascript:alert(1))\n\n![secret](../../Cargo.toml)\n\n![remote](https://example.com/x.png)",Path::new("."),"test".into()).unwrap();
    assert!(!d.html.contains("<script>"));
    assert!(!d.html.contains("javascript:"));
    assert!(!d.html.contains("src=\"https"));
    assert!(d.html.contains("参照フォルダー外") || d.html.contains("画像が見つかりません"));
}
#[test]
fn size_is_bounded() {
    assert!(
        render(
            &"a".repeat(MAX_DOCUMENT as usize + 1),
            Path::new("."),
            "x".into()
        )
        .is_err()
    );
}
#[test]
fn source_bytes_preserved() {
    let p = &Path::new(env!("CARGO_MANIFEST_DIR")).join("../README.md");
    let before = std::fs::read(p).unwrap();
    load(p).unwrap();
    assert_eq!(before, std::fs::read(p).unwrap());
}

#[test]
fn bom_invalid_utf8_and_image_boundaries() {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };
    let root = std::env::temp_dir().join(format!(
        "md-viewer-tests-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    let p = root.join("日本語 空白.md");
    fs::write(&p, b"\xef\xbb\xbf# Title").unwrap();
    assert_eq!(load(&p).unwrap().headings[0].title, "Title");
    fs::write(root.join("invalid.md"), [255]).unwrap();
    assert!(load(&root.join("invalid.md")).is_err());
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("icons/icon.png"),
        root.join("image.png"),
    )
    .unwrap();
    let d = render(
        "![ok](image.png)\n\n![bad](file:///etc/passwd)",
        &root,
        "test".into(),
    )
    .unwrap();
    assert!(d.html.contains("data:image/png;base64,"));
    assert!(d.html.contains("外部画像"));
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(std::env::temp_dir(), root.join("outside")).unwrap();
        assert!(
            render("![x](outside/no.png)", &root, "test".into())
                .unwrap()
                .html
                .contains("画像が見つかりません")
        );
    }
}

#[test]
fn local_links_respect_boundary() {
    use md_viewer::document::resolve_local;
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../README.md")
        .canonicalize()
        .unwrap();
    assert!(
        resolve_local(&source, "docs/markdown-reader/PLAN.md")
            .unwrap()
            .ends_with("PLAN.md")
    );
    assert!(resolve_local(&source, "https://example.com").is_err());
    assert!(resolve_local(&source, "../leak-fence/README.md").is_err());
    assert_eq!(
        resolve_local(&source, "missing.md").unwrap_err(),
        "リンク切れです。"
    );
}

#[test]
fn encoded_paths_and_symlinks() {
    use md_viewer::document::resolve_local;
    let root = std::env::temp_dir().join(format!(
        "md-links-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    std::fs::write(root.join("入口.md"), "# Main").unwrap();
    std::fs::write(root.join("関連 資料.md"), "# 関連").unwrap();
    assert!(
        resolve_local(
            &root.join("入口.md"),
            "%E9%96%A2%E9%80%A3%20%E8%B3%87%E6%96%99.md#関連"
        )
        .unwrap()
        .ends_with("関連 資料.md")
    );
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("icons"),
            root.join("escape"),
        )
        .unwrap();
        assert!(
            render("![x](escape/icon.png)", &root, "test".into())
                .unwrap()
                .html
                .contains("参照フォルダー外")
        );
    }
}

#[test]
fn search_loading_skips_images_but_keeps_text_and_validation() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("search.md");
    std::fs::write(
        &path,
        "# Search\n\nBefore ![private](missing.png) after **match**.\n\n<script>raw</script>",
    )
    .unwrap();
    let doc = md_viewer::document::load_for_search(&path).unwrap();
    assert!(doc.html.contains("Before "));
    assert!(doc.html.contains("after <strong>match</strong>"));
    assert!(!doc.html.contains("image-error"));
    assert!(!doc.html.contains("<img"));
    assert!(!doc.html.contains("<script>"));
    assert_eq!(doc.id, path.canonicalize().unwrap().to_string_lossy());
    assert_eq!(doc.headings[0].id, "search");
    std::fs::write(&path, [0xff]).unwrap();
    assert!(md_viewer::document::load_for_search(&path).is_err());
    std::fs::write(&path, vec![b'a'; MAX_DOCUMENT as usize + 1]).unwrap();
    assert!(md_viewer::document::load_for_search(&path).is_err());
}

#[test]
fn external_links_have_no_navigable_web_href() {
    let d = render("[trusted name](https://evil.example/a?q=1&x=2)\n\n<https://example.com>\n\n[local](a.md) [section](#intro) [mail](mailto:person@example.com)", Path::new("."), "links".into()).unwrap();
    assert!(
        d.html
            .contains("href=\"\" data-external=\"https://evil.example/a?q=1&amp;x=2\"")
    );
    assert!(d.html.contains("data-external=\"https://example.com\""));
    assert!(d.html.contains("data-local=\"a.md\""));
    assert!(d.html.contains("href=\"#intro\""));
    assert!(!d.html.contains("href=\"https://"));
    assert!(!d.html.contains("data-external=\"mailto:"));
}
