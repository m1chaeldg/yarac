use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeKind {
    Created,
    Modified,
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceChangeEvent {
    pub path: PathBuf,
    pub kind: ChangeKind,
    pub content: Option<String>,
}

/// Thread-safe file watcher with write echo suppression and debouncing
pub struct WorkspaceWatcher {
    _watcher: RecommendedWatcher,
    suppression_registry: Arc<Mutex<HashMap<PathBuf, Instant>>>,
    #[allow(dead_code)]
    last_event_times: Arc<Mutex<HashMap<PathBuf, Instant>>>,
    receiver: Mutex<Receiver<WorkspaceChangeEvent>>,
}

impl WorkspaceWatcher {
    /// Initializes a new WorkspaceWatcher for the target directory
    pub fn new(watch_dir: &Path) -> Result<Self, notify::Error> {
        let (tx, rx) = channel::<WorkspaceChangeEvent>();
        let suppression_registry = Arc::new(Mutex::new(HashMap::<PathBuf, Instant>::new()));
        let last_event_times = Arc::new(Mutex::new(HashMap::<PathBuf, Instant>::new()));

        let suppression_clone = Arc::clone(&suppression_registry);
        let last_events_clone = Arc::clone(&last_event_times);
        let watch_dir_buf = watch_dir.to_path_buf();

        let event_handler = move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                Self::process_notify_event(
                    event,
                    &watch_dir_buf,
                    &suppression_clone,
                    &last_events_clone,
                    &tx,
                );
            }
        };

        let mut watcher = RecommendedWatcher::new(event_handler, Config::default())?;
        watcher.watch(watch_dir, RecursiveMode::Recursive)?;

        Ok(Self {
            _watcher: watcher,
            suppression_registry,
            last_event_times,
            receiver: Mutex::new(rx),
        })
    }

    /// Registers a path written by Yarac itself to suppress echo change events for a short duration
    pub fn mark_self_write(&self, path: &Path, window: Duration) {
        if let Ok(mut reg) = self.suppression_registry.lock() {
            reg.insert(path.to_path_buf(), Instant::now() + window);
        }
    }

    /// Non-blocking check for incoming file change events
    pub fn try_recv(&self) -> Option<WorkspaceChangeEvent> {
        self.receiver.lock().ok()?.try_recv().ok()
    }

    /// Blocking wait for the next file change event
    pub fn recv_timeout(&self, timeout: Duration) -> Option<WorkspaceChangeEvent> {
        self.receiver.lock().ok()?.recv_timeout(timeout).ok()
    }

    fn process_notify_event(
        event: Event,
        _watch_dir: &Path,
        suppression_registry: &Arc<Mutex<HashMap<PathBuf, Instant>>>,
        last_event_times: &Arc<Mutex<HashMap<PathBuf, Instant>>>,
        tx: &Sender<WorkspaceChangeEvent>,
    ) {
        let kind = match event.kind {
            EventKind::Create(_) => ChangeKind::Created,
            EventKind::Modify(_) => ChangeKind::Modified,
            EventKind::Remove(_) => ChangeKind::Deleted,
            _ => return,
        };

        let now = Instant::now();

        for path in event.paths {
            // Only observe .yarac, .env.yarac, and workspace.json files
            let is_relevant = path.extension().and_then(|e| e.to_str()).map_or(false, |ext| {
                ext == "yarac" || ext == "json"
            }) || path.file_name().and_then(|n| n.to_str()).map_or(false, |n| {
                n.ends_with(".env.yarac") || n == "workspace.json"
            });

            if !is_relevant {
                continue;
            }

            // Check self-write suppression
            if let Ok(mut reg) = suppression_registry.lock() {
                reg.retain(|_, expiry| *expiry > now);
                if let Some(expiry) = reg.get(&path) {
                    if *expiry > now {
                        continue; // Echo suppressed!
                    }
                }
            }

            // Debounce rapid duplicate writes within 100ms
            if let Ok(mut last_times) = last_event_times.lock() {
                last_times.retain(|_, time| now.duration_since(*time) < Duration::from_secs(5));
                if let Some(last_time) = last_times.get(&path) {
                    if now.duration_since(*last_time) < Duration::from_millis(120) {
                        continue; // Debounced
                    }
                }
                last_times.insert(path.clone(), now);
            }

            let content = if kind != ChangeKind::Deleted && path.is_file() {
                std::fs::read_to_string(&path).ok()
            } else {
                None
            };

            let _ = tx.send(WorkspaceChangeEvent {
                path,
                kind: kind.clone(),
                content,
            });
        }
    }
}
