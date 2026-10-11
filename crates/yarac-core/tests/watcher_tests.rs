use std::fs;
use std::time::Duration;
use yarac_core::models::*;
use yarac_core::watcher::{ChangeKind, WorkspaceWatcher};

#[test]
fn test_workspace_manifest_serialization() {
    let manifest = WorkspaceManifest {
        id: "ws_test".into(),
        name: "Test Workspace".into(),
        collections: vec![
            CollectionMeta {
                id: "col_1".into(),
                name: "Auth API".into(),
                path: None,
                parent_id: None,
                request_files: vec!["login.yarac".into(), "refresh.yarac".into()],
            },
            CollectionMeta {
                id: "col_2".into(),
                name: "Users API".into(),
                path: None,
                parent_id: None,
                request_files: vec!["list_users.yarac".into()],
            },
        ],
        active_environment_id: Some("env_local".into()),
        open_tabs: vec!["col_1/login.yarac".into()],
        active_tab: Some("col_1/login.yarac".into()),
    };

    let serialized = serde_json::to_string_pretty(&manifest).expect("serialize manifest");
    let deserialized: WorkspaceManifest = serde_json::from_str(&serialized).expect("deserialize manifest");

    assert_eq!(manifest.id, deserialized.id);
    assert_eq!(manifest.collections.len(), 2);
    assert_eq!(manifest.collections[0].request_files.len(), 2);
}

#[test]
fn test_watcher_detects_external_file_modifications() {
    let temp_dir = std::env::temp_dir().join(format!("yarac_watch_test_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).expect("create temp dir");

    let watcher = WorkspaceWatcher::new(&temp_dir).expect("start watcher");

    // Write a .yarac file (simulating Notepad saving a new file)
    let test_file = temp_dir.join("create_user.yarac");
    let content_v1 = "version = \"1.0\"\nname = \"Create User\"\nmethod = \"POST\"\nurl = \"https://example.com/api\"\n";
    fs::write(&test_file, content_v1).expect("write initial file");

    // Allow background watcher to pick up file creation/modification
    let mut detected = false;
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_millis(1500) {
        if let Some(event) = watcher.recv_timeout(Duration::from_millis(300)) {
            if event.path == test_file && (event.kind == ChangeKind::Created || event.kind == ChangeKind::Modified) {
                detected = true;
                break;
            }
        }
    }
    assert!(detected, "Watcher should detect external file creation/write");

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_watcher_echo_suppression_ignores_internal_saves() {
    let temp_dir = std::env::temp_dir().join(format!("yarac_suppress_test_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).expect("create temp dir");

    let watcher = WorkspaceWatcher::new(&temp_dir).expect("start watcher");
    let test_file = temp_dir.join("internal_save.yarac");

    // Mark self-write BEFORE writing file
    watcher.mark_self_write(&test_file, Duration::from_millis(800));
    fs::write(&test_file, "version = \"1.0\"\nname = \"Self Saved\"\n").expect("write self file");

    // Should NOT receive event within suppression window
    let event = watcher.recv_timeout(Duration::from_millis(300));
    assert!(event.is_none(), "Internal save should be suppressed by WorkspaceWatcher");

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}
