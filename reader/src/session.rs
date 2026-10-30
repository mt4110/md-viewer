use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ReadingAnchor {
    pub block_hash: String,
    pub heading_hash: Option<String>,
    pub occurrence: u32,
    pub count: u32,
    pub top: f64,
    pub scroll: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Tab {
    pub path: String,
    pub anchor: Option<ReadingAnchor>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    pub restore_enabled: bool,
    pub selected: Option<String>,
    pub tabs: Vec<Tab>,
}
impl Default for Session {
    fn default() -> Self {
        Self {
            version: 1,
            restore_enabled: true,
            selected: None,
            tabs: vec![],
        }
    }
}
#[derive(Serialize)]
pub struct LoadedSession {
    pub session: Session,
    pub warning: Option<String>,
}
fn hash_valid(s: &str) -> bool {
    s.len() == 16 && s.bytes().all(|b| b.is_ascii_hexdigit())
}
pub fn valid_path(s: &str) -> bool {
    let p = Path::new(s);
    if !p.is_absolute() || s.len() > 32768 || s.contains('\0') {
        return false;
    }
    #[cfg(windows)]
    if !matches!(p.components().next(),Some(std::path::Component::Prefix(prefix)) if matches!(prefix.kind(),std::path::Prefix::Disk(_)|std::path::Prefix::VerbatimDisk(_)))
    {
        return false;
    }
    matches!(
        p.extension()
            .and_then(|s| s.to_str())
            .map(str::to_lowercase)
            .as_deref(),
        Some("md" | "markdown")
    )
}
impl Session {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || self.tabs.len() > 20 {
            return Err("復元情報の形式を確認できません。".into());
        }
        let mut seen = HashSet::new();
        for tab in &self.tabs {
            if !valid_path(&tab.path) || !seen.insert(&tab.path) {
                return Err("復元するファイルの情報が不正です。".into());
            }
            if let Some(a) = &tab.anchor
                && (!hash_valid(&a.block_hash)
                    || a.heading_hash.as_ref().is_some_and(|h| !hash_valid(h))
                    || a.count == 0
                    || a.count > 100000
                    || a.occurrence >= a.count
                    || !a.top.is_finite()
                    || a.top.abs() > 1e8
                    || !a.scroll.is_finite()
                    || !(0.0..=1e8).contains(&a.scroll))
            {
                return Err("読書位置の情報が不正です。".into());
            }
        }
        if self.selected.as_ref().is_some_and(|s| !seen.contains(s)) {
            return Err("選択タブの情報が不正です。".into());
        }
        Ok(())
    }
}
pub struct SessionStore {
    path: PathBuf,
}
impl SessionStore {
    pub fn new(directory: &Path) -> Self {
        Self {
            path: directory.join("session.json"),
        }
    }
    pub fn load(&self) -> Result<LoadedSession, String> {
        let file = match fs::File::open(&self.path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(LoadedSession {
                    session: Session::default(),
                    warning: None,
                });
            }
            Err(_) => return Err("復元情報を読み込めません。".into()),
        };
        let mut bytes = Vec::new();
        file.take(1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "復元情報を読み込めません。")?;
        if bytes.len() > 1024 * 1024 {
            return Err(
                "復元情報が大きすぎるため読み込みません。元の情報は保持しています。".into(),
            );
        }
        let session = serde_json::from_slice::<Session>(&bytes)
            .ok()
            .filter(|s| bytes.len() <= 1024 * 1024 && s.validate().is_ok());
        if let Some(mut session) = session {
            if !session.restore_enabled {
                session.tabs.clear();
                session.selected = None;
            }
            return Ok(LoadedSession {
                session,
                warning: None,
            });
        }
        // Never silently overwrite the evidence of a malformed/unsupported state file.
        let mut backup = tempfile::Builder::new()
            .prefix("session-invalid-")
            .suffix(".json")
            .tempfile_in(self.path.parent().unwrap())
            .map_err(|_| "破損した復元情報を保全できません。")?;
        backup
            .write_all(&bytes)
            .map_err(|_| "復元情報を保全できません。")?;
        backup.keep().map_err(|_| "復元情報を保全できません。")?;
        Ok(LoadedSession {
            session: Session::default(),
            warning: Some("復元情報が壊れていたため、保全して空画面で開始しました。".into()),
        })
    }
    pub fn save(&self, session: &Session) -> Result<(), String> {
        session.validate()?;
        if fs::metadata(&self.path).is_ok_and(|m| m.len() > 1024 * 1024) {
            return Err("大きすぎる復元情報を上書きしません。".into());
        }
        let mut stored = session.clone();
        if !stored.restore_enabled {
            stored.tabs.clear();
            stored.selected = None;
        }
        let bytes = serde_json::to_vec(&stored).map_err(|_| "復元情報を作成できません。")?;
        let parent = self.path.parent().unwrap();
        fs::create_dir_all(parent).map_err(|_| "復元情報の保存先を作成できません。")?;
        let mut file = tempfile::Builder::new()
            .prefix("session-write-")
            .tempfile_in(parent)
            .map_err(|_| "復元情報を保存できません。")?;
        file.write_all(&bytes)
            .and_then(|_| file.as_file().sync_all())
            .map_err(|_| "復元情報を書き込めません。")?;
        file.persist(&self.path)
            .map_err(|_| "復元情報を置き換えられません。前回の情報を保持しています。")?;
        Ok(())
    }
}
