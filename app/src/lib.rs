//! Desktop shell of the knowledge base app (Tauri 2).
//!
//! Exposes the commands and events the user interface in `ui/` is built against. In
//! Phase 0 the commands run on the mocks of `kb-core`; Phase 2 wires the real crates.
//!
//! - `api`: arguments, results, errors and the event list (data types come from
//!   `kb-core` with its feature `ts`),
//! - `commands`: the command list and the Tauri glue,
//! - `backend`: the command logic, testable without a window,
//! - `bindings`: generation of `ui/api/bindings.ts`.

pub mod api;
pub mod backend;
mod bindings;
mod commands;
pub mod mock;

use std::sync::Arc;

use kb_core::ColorScheme;
use tauri::{AppHandle, Emitter, Manager, Theme};

pub use bindings::generate_bindings;

/// Native window theme for the colour scheme setting (ADR 0024): `None` follows the
/// operating system. Applied to the title bar, which the web view cannot style.
pub fn window_theme(scheme: ColorScheme) -> Option<Theme> {
    match scheme {
        ColorScheme::System => None,
        ColorScheme::Light => Some(Theme::Light),
        ColorScheme::Dark => Some(Theme::Dark),
    }
}

fn apply_theme(app: &AppHandle, scheme: ColorScheme) {
    app.set_theme(window_theme(scheme));
}

/// Starts the application.
pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let emit: backend::EventSink =
                Arc::new(move |event: api::Event| match event.payload() {
                    Ok(payload) => {
                        if let api::Event::SettingsChanged(settings) = &event {
                            apply_theme(&handle, settings.color_scheme);
                        }
                        if let Err(error) = handle.emit(event.name(), payload) {
                            eprintln!("cannot emit {}: {error}", event.name());
                        }
                    }
                    Err(error) => eprintln!("cannot serialise {}: {error}", event.name()),
                });
            let config = mock::MockConfig {
                models_dir: app.path().app_local_data_dir()?.join("models"),
                pace: std::time::Duration::from_millis(40),
                // The app starts without models, so the setup shows.
                provided: Vec::new(),
                ..mock::MockConfig::default()
            };
            let setup = tauri::async_runtime::block_on(mock::seeded_backend(emit, &config))?;
            let settings = tauri::async_runtime::block_on(setup.backend.get_settings())
                .map_err(|e| e.message)?;
            apply_theme(app.handle(), settings.color_scheme);
            setup.backend.start();
            app.manage(setup.backend);
            Ok(())
        })
        .invoke_handler(commands::invoke_handler())
        .run(tauri::generate_context!())
}
