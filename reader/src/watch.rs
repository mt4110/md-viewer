//! One foreground document, OS events, bounded debounce/retries. No directory scans.
use crate::document::{Document, load};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

#[derive(Default)]
pub struct WatchHub {
    watcher: Option<RecommendedWatcher>,
    generation: Arc<AtomicU64>,
}
impl WatchHub {
    pub fn stop(&mut self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
        self.watcher = None;
    }
    pub fn start(
        &mut self,
        path: PathBuf,
        callback: impl Fn(Result<Document, String>) + Send + 'static,
    ) -> Result<(), String> {
        self.stop();
        let generation = self.generation.load(Ordering::SeqCst);
        let current = self.generation.clone();
        let parent = path.parent().ok_or("監視フォルダーがありません。")?;
        let target = path.clone();
        let (tx, rx) = mpsc::sync_channel(1);
        let mut watcher = notify::recommended_watcher(move |event: notify::Result<Event>| {
            let (relevant, failed) = match event {
                Ok(e) => (
                    !matches!(e.kind, EventKind::Access(_))
                        && (e.paths.is_empty() || e.paths.iter().any(|p| p == &target)),
                    false,
                ),
                Err(_) => (true, true),
            };
            if relevant {
                let _ = tx.try_send(failed);
            }
        })
        .map_err(|_| "ファイルの更新監視を開始できません。")?;
        watcher
            .watch(parent, RecursiveMode::NonRecursive)
            .map_err(|_| "ファイルの更新監視を開始できません。")?;
        self.watcher = Some(watcher);
        thread::spawn(move || {
            // Recheck immediately: also catches saves while this tab was in the background.
            let mut failed = false;
            loop {
                if current.load(Ordering::SeqCst) != generation {
                    break;
                }
                let mut result = if failed {
                    Err("更新監視でエラーが発生しました。再読込してください。".into())
                } else {
                    load(&path)
                };
                for delay in [80, 160, 320] {
                    if failed || result.is_ok() || current.load(Ordering::SeqCst) != generation {
                        break;
                    }
                    thread::sleep(Duration::from_millis(delay));
                    result = load(&path);
                }
                if current.load(Ordering::SeqCst) != generation {
                    break;
                }
                callback(result);
                failed = match rx.recv() {
                    Ok(value) => value,
                    Err(_) => break,
                };
                let start = Instant::now();
                while start.elapsed() < Duration::from_secs(1) {
                    match rx.recv_timeout(Duration::from_millis(200)) {
                        Ok(value) => {
                            failed |= value;
                        }
                        Err(mpsc::RecvTimeoutError::Timeout) => break,
                        Err(_) => return,
                    }
                }
            }
        });
        Ok(())
    }
}
impl Drop for WatchHub {
    fn drop(&mut self) {
        self.stop();
    }
}
