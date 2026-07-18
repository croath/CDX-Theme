// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  // rustls 0.23+ requires an explicit process-wide crypto provider before any TLS client.
  // Used by reqwest and tauri-plugin-updater.
  let _ = rustls::crypto::ring::default_provider().install_default();

  cdx_theme_app_lib::run()
}
