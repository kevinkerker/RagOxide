//! Guards the capabilities of the main window: only what the user interface needs
//! (`ui/api/tauri.ts`: `invoke` with channels, `listen`), never `core:default`.

use std::path::Path;

/// Everything the webview may use. App commands and channels need no permission.
const ALLOWED: &[&str] = &["core:event:allow-listen", "core:event:allow-unlisten"];

fn capability() -> serde_json::Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("capabilities/default.json");
    let text = std::fs::read_to_string(&path).expect("capabilities/default.json is readable");
    serde_json::from_str(&text).expect("capabilities/default.json is JSON")
}

#[test]
fn the_main_window_gets_only_listen_and_unlisten() {
    let capability = capability();
    let permissions = capability["permissions"]
        .as_array()
        .expect("permissions is a list");
    assert!(
        !permissions.is_empty(),
        "the user interface needs listen and unlisten"
    );
    for permission in permissions {
        let name = permission
            .as_str()
            .unwrap_or_else(|| panic!("permission without scope expected, got {permission}"));
        assert_ne!(
            name, "core:default",
            "core:default allows emit, path, image, menu and tray"
        );
        assert!(
            ALLOWED.contains(&name),
            "{name} is not in the allowed list of app/tests/capabilities.rs"
        );
    }
    assert_eq!(capability["windows"], serde_json::json!(["main"]));
}

#[test]
fn there_is_no_other_capability_file() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("capabilities");
    let files: Vec<String> = std::fs::read_dir(dir)
        .expect("capabilities is a folder")
        .map(|entry| {
            entry
                .expect("readable entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(
        files,
        ["default.json"],
        "every capability file needs its own guard"
    );
}
