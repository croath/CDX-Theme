fn main() {
  // Load workspace `.env` (if present) so local builds pick up POSTHOG_* without exporting.
  // Does not override variables already set in the process environment.
  // Same vars are used by app-ui/build.rs to generate public/posthog-config.js.
  load_dotenv_if_present(std::path::Path::new("../.env"));
  load_dotenv_if_present(std::path::Path::new(".env"));

  // Embed public PostHog project key / host into the native binary (posthog-rs).
  for key in ["POSTHOG_API_KEY", "POSTHOG_HOST"] {
    if let Ok(val) = std::env::var(key) {
      if !val.trim().is_empty() {
        println!("cargo:rustc-env={key}={}", val.trim());
      }
    }
    println!("cargo:rerun-if-env-changed={key}");
  }
  // Only watch env files that exist — missing paths can force rebuild-script every compile.
  for env_path in ["../.env", ".env"] {
    if std::path::Path::new(env_path).is_file() {
      println!("cargo:rerun-if-changed={env_path}");
    }
  }
  tauri_build::build()
}

/// Minimal KEY=VALUE loader (no dependency). Skips comments and blank lines.
fn load_dotenv_if_present(path: &std::path::Path) {
  let Ok(raw) = std::fs::read_to_string(path) else {
    return;
  };
  for line in raw.lines() {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
      continue;
    }
    let Some((key, value)) = line.split_once('=') else {
      continue;
    };
    let key = key.trim();
    if key.is_empty() || std::env::var_os(key).is_some() {
      continue;
    }
    let value = value.trim().trim_matches(|c| c == '"' || c == '\'');
    // SAFETY: single-threaded build script; only sets missing vars.
    unsafe { std::env::set_var(key, value) };
  }
}
