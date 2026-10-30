use serde_json::Value;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

// Deliberately retained under ignored out/: no automatic fixture deletion.
pub struct Project {
    pub root: PathBuf,
}

impl Project {
    pub fn new() -> Self {
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("out/test-runs");
        fs::create_dir_all(&base).unwrap();
        let root = base.join(format!(
            "{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let target = root.join("fixtures/demo");
        copy_new(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/demo"),
            &target,
        );
        Self { root }
    }

    pub fn fixtures(&self) -> PathBuf {
        self.root.join("fixtures/demo")
    }

    pub fn edit(&self, relative: &str, edit: impl FnOnce(&mut Value)) {
        let path = self.fixtures().join(relative);
        let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        edit(&mut value);
        fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    }

    pub fn write(&self, relative: &str, bytes: impl AsRef<[u8]>) {
        fs::write(self.fixtures().join(relative), bytes).unwrap();
    }
}

fn copy_new(source: &Path, target: &Path) {
    fs::create_dir_all(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let dest = target.join(entry.file_name());
        if path.is_dir() {
            copy_new(&path, &dest);
        } else {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(dest)
                .unwrap();
            file.write_all(&fs::read(path).unwrap()).unwrap();
        }
    }
}
