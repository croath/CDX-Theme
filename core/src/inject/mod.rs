//! CDP theme injector for host apps (Codex + WorkBuddy).
//!
//! Apply flow (inspired by [Codex-Dream-Skin](https://github.com/Fei-Away/Codex-Dream-Skin)):
//! wait page targets → probe shell markers → evaluate inject → register
//! `Page.addScriptToEvaluateOnNewDocument` so SPA reloads keep the skin → verify.
//!
//! Inject sets **cdxtheme-only** multi-image CSS vars: `--cdxtheme-image-{name}`.

pub mod theme;

pub use crate::cdp::{CdpTarget, TargetUrlKind, wait_for_targets, wait_for_targets_with};
pub use theme::{
  build_inject_expression, build_inject_expression_for_app, build_inject_expression_workbuddy,
  load_theme_package,
};
// Loaded theme model lives in `cdx-theme-types`.
pub use cdx_theme_types::{
  APP_CODEX, APP_WORKBUDDY, BaseTheme, BaseThemeFonts, CodexLoadedTarget, CodexTargetOptions,
  CodexVerification, LoadedTargets, LoadedTheme, PublicTheme, SelectorCheck, SemanticColors,
  ThemeCopy, VerificationContext, VerificationWhen, WorkBuddyLoadedTarget, WorkBuddyVerification,
};

use crate::cdp::CdpSession;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{LazyLock, Mutex};

/// Early-document script identifiers registered per CDP page target.
/// Cleared / replaced on next apply; removed on restore.
static EARLY_SCRIPT_IDS: LazyLock<Mutex<HashMap<String, String>>> =
  LazyLock::new(|| Mutex::new(HashMap::new()));

/// Default Codex desktop remote-debugging port.
pub const DEFAULT_CDP_PORT: u16 = 9335;
/// Default WorkBuddy desktop remote-debugging port.
pub const DEFAULT_WORKBUDDY_CDP_PORT: u16 = 9336;

#[derive(Clone, Debug)]
pub struct InjectOptions {
  pub port: u16,
  pub timeout_ms: u64,
}

impl Default for InjectOptions {
  fn default() -> Self {
    Self {
      port: DEFAULT_CDP_PORT,
      // Allow large multi-image inject payloads over CDP.
      timeout_ms: 120_000,
    }
  }
}

/// Host-specific CDP inject/restore/verify behavior.
#[derive(Clone, Copy, Debug)]
struct HostProfile {
  app_id: &'static str,
  url_kind: TargetUrlKind,
  probe_expression: &'static str,
  /// JS condition body that returns true when the shell is ready for early inject.
  early_ready_js: &'static str,
  remove_expression: &'static str,
  /// Class used for soft verify when host state is missing.
  skin_class: &'static str,
}

const CODEX_HOST: HostProfile = HostProfile {
  app_id: APP_CODEX,
  url_kind: TargetUrlKind::App,
  probe_expression: PROBE_EXPRESSION_CODEX,
  early_ready_js: r#"
    const shell = document.querySelector("main.main-surface") || document.querySelector("main");
    const sidebar = document.querySelector("aside.app-shell-left-panel") || document.querySelector("aside");
    return Boolean(shell && sidebar);
  "#,
  remove_expression: REMOVE_EXPRESSION_CODEX,
  skin_class: "cdxtheme-codex-skin",
};

const WORKBUDDY_HOST: HostProfile = HostProfile {
  app_id: APP_WORKBUDDY,
  url_kind: TargetUrlKind::File,
  probe_expression: PROBE_EXPRESSION_WORKBUDDY,
  // More `#` delimiters so JS `#root` does not end the raw string.
  early_ready_js: r##"
    const shell = document.querySelector(".teams-container") || document.querySelector("#root");
    return Boolean(shell);
  "##,
  remove_expression: REMOVE_EXPRESSION_WORKBUDDY,
  skin_class: "cdxtheme-workbuddy-skin",
};

fn host_profile(app_id: &str) -> Result<&'static HostProfile, String> {
  match app_id.trim().to_ascii_lowercase().as_str() {
    "codex" => Ok(&CODEX_HOST),
    "workbuddy" => Ok(&WORKBUDDY_HOST),
    other => Err(format!(
      "unsupported host app `{other}` (supported: codex, workbuddy)"
    )),
  }
}

/// Default CDP port for a host app id.
pub fn default_cdp_port_for_app(app_id: &str) -> u16 {
  match app_id.trim().to_ascii_lowercase().as_str() {
    "workbuddy" => DEFAULT_WORKBUDDY_CDP_PORT,
    _ => DEFAULT_CDP_PORT,
  }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetResult {
  pub target_id: String,
  pub title: String,
  pub url: String,
  pub result: Value,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InjectRunResult {
  pub mode: String,
  pub port: u16,
  /// Applied theme id when known (empty for restore/verify without expected).
  #[serde(default)]
  pub theme_id: String,
  pub targets: Vec<TargetResult>,
}

/// Codex host preflight — Dream-Skin style: require shell + sidebar.
/// Soft gate: still inject if incomplete (SPA may finish after first paint).
const PROBE_EXPRESSION_CODEX: &str = r#"(() => {
  const shell = Boolean(document.querySelector('main.main-surface') || document.querySelector('main'));
  const sidebar = Boolean(document.querySelector('aside.app-shell-left-panel') || document.querySelector('aside'));
  const composer = Boolean(document.querySelector('.composer-surface-chrome') || document.querySelector("[class*='composer']"));
  const missing = [];
  if (!shell) missing.push({ name: "shell", selectors: ["main.main-surface", "main"] });
  if (!sidebar) missing.push({ name: "sidebar", selectors: ["aside.app-shell-left-panel", "aside"] });
  return {
    appId: "codex",
    // Dream-Skin: codex = shell && sidebar (composer optional on some routes)
    compatible: shell && sidebar,
    shell,
    sidebar,
    composer,
    missing,
    readyState: document.readyState,
    href: location.href,
  };
})()"#;

/// WorkBuddy host preflight — shell landmarks from theme skill notes.
/// Uses `r##` so CSS/DOM selectors like `#root` do not terminate the raw string.
const PROBE_EXPRESSION_WORKBUDDY: &str = r##"(() => {
  const shell = Boolean(
    document.querySelector('.teams-container') || document.querySelector('#root')
  );
  const sidebar = Boolean(
    document.querySelector('.conversation-sidebar') || document.querySelector('.conversation-list')
  );
  const composer = Boolean(
    document.querySelector('[role="textbox"][contenteditable="true"]')
  );
  const missing = [];
  if (!shell) missing.push({ name: "shell", selectors: [".teams-container", "#root"] });
  return {
    appId: "workbuddy",
    // Shell is enough to inject; sidebar/composer appear on some routes only.
    compatible: shell,
    shell,
    sidebar,
    composer,
    missing,
    readyState: document.readyState,
    href: location.href,
  };
})()"##;

/// Wrap inject payload so it waits for host shell markers before applying.
/// Port of Dream-Skin `earlyPayloadFor` (Page.addScriptToEvaluateOnNewDocument).
fn early_payload_for(payload: &str, generation: &str, host: &HostProfile) -> String {
  // generation is JSON-encoded for safe embedding
  let gen_json = serde_json::to_string(generation).unwrap_or_else(|_| "\"cdxtheme\"".into());
  let ready = host.early_ready_js;
  format!(
    r#"(() => {{
  const generationKey = "__CDXTHEME_EARLY_GENERATION__";
  const appliedKey = "__CDXTHEME_EARLY_APPLIED__";
  const generation = {gen_json};
  window[generationKey] = generation;
  let observer = null;
  let timeout = null;
  const stop = () => {{
    try {{ observer && observer.disconnect(); }} catch (e) {{}}
    observer = null;
    if (timeout) clearTimeout(timeout);
    timeout = null;
  }};
  const hostReady = () => {{
    {ready}
  }};
  const install = () => {{
    if (window[generationKey] !== generation) {{ stop(); return true; }}
    if (!document.documentElement) return false;
    if (!hostReady()) return false;
    stop();
    try {{
      {payload}
      window[appliedKey] = generation;
    }} catch (e) {{
      console.error("[cdxtheme] early inject failed", e);
    }}
    return true;
  }};
  if (install()) return;
  if (typeof MutationObserver === "function" && document.documentElement) {{
    observer = new MutationObserver(install);
    observer.observe(document.documentElement, {{ childList: true, subtree: true }});
  }}
  timeout = setTimeout(stop, 15000);
}})()"#
  )
}

/// Remove Codex skin: prefer live host cleanup, else static teardown.
const REMOVE_EXPRESSION_CODEX: &str = r#"(() => {
  const appId = "codex";
  const state = window.__CDXTHEME__ && window.__CDXTHEME__.hosts && window.__CDXTHEME__.hosts[appId];
  if (state && typeof state.cleanup === "function") return state.cleanup();

  const ids = [
    "cdxtheme-theme-style-codex",
    "cdxtheme-codex-skin-chrome",
  ];
  for (const id of ids) {
    const node = document.getElementById(id);
    if (node) node.remove();
  }
  const root = document.documentElement;
  if (root) {
    root.classList.remove("cdxtheme-codex-skin", "cdxtheme-host-codex", "cdxtheme-theme");
    delete root.dataset.cdxthemeHost;
    delete root.dataset.cdxthemeTheme;
    delete root.dataset.cdxthemeThemeVersion;
    delete root.dataset.codexSkinTheme;
    delete root.dataset.codexSkinBrand;
    root.style.removeProperty("--dream-art");
    root.style.removeProperty("--cdxtheme-art");
    root.style.removeProperty("--dream-tagline");
    root.style.removeProperty("--dream-project-prefix");
    root.style.removeProperty("--dream-project-label");
    if (root.style) {
      for (let i = root.style.length - 1; i >= 0; i -= 1) {
        const name = root.style.item(i);
        if (name.startsWith("--cdxtheme-image-")) {
          root.style.removeProperty(name);
        }
      }
    }
  }
  document.querySelectorAll(".dream-home").forEach((n) => n.classList.remove("dream-home"));
  document.querySelectorAll(".dream-home-shell").forEach((n) => n.classList.remove("dream-home-shell"));
  if (window.__CDXTHEME__ && window.__CDXTHEME__.hosts) delete window.__CDXTHEME__.hosts[appId];
  return true;
})()"#;

/// Remove WorkBuddy skin: prefer live host cleanup, else static teardown.
const REMOVE_EXPRESSION_WORKBUDDY: &str = r#"(() => {
  const appId = "workbuddy";
  const state = window.__CDXTHEME__ && window.__CDXTHEME__.hosts && window.__CDXTHEME__.hosts[appId];
  if (state && typeof state.cleanup === "function") return state.cleanup();

  const ids = [
    "cdxtheme-theme-style-workbuddy",
    "cdxtheme-workbuddy-skin-chrome",
  ];
  for (const id of ids) {
    const node = document.getElementById(id);
    if (node) node.remove();
  }
  const root = document.documentElement;
  if (root) {
    root.classList.remove("cdxtheme-workbuddy-skin", "cdxtheme-host-workbuddy", "cdxtheme-theme");
    delete root.dataset.cdxthemeHost;
    delete root.dataset.cdxthemeTheme;
    delete root.dataset.cdxthemeThemeVersion;
    root.style.removeProperty("--dream-art");
    root.style.removeProperty("--cdxtheme-art");
    root.style.removeProperty("--dream-tagline");
    root.style.removeProperty("--dream-project-prefix");
    root.style.removeProperty("--dream-project-label");
    if (root.style) {
      for (let i = root.style.length - 1; i >= 0; i -= 1) {
        const name = root.style.item(i);
        if (name.startsWith("--cdxtheme-image-")) {
          root.style.removeProperty(name);
        }
      }
    }
  }
  if (window.__CDXTHEME__ && window.__CDXTHEME__.hosts) delete window.__CDXTHEME__.hosts[appId];
  return true;
})()"#;

fn verify_expression(expected_theme: Option<&PublicTheme>, host: &HostProfile) -> String {
  let expected = expected_theme
    .map(|t| serde_json::to_string(t).unwrap_or_else(|_| "null".into()))
    .unwrap_or_else(|| "null".into());
  let app_id = serde_json::to_string(host.app_id).unwrap_or_else(|_| "\"codex\"".into());
  let skin_class =
    serde_json::to_string(host.skin_class).unwrap_or_else(|_| "\"cdxtheme-codex-skin\"".into());

  // Core success: installed + style + theme id/version.
  // Profile details are warnings only (themes may overflow slightly, chrome may lag).
  format!(
    r#"(() => {{
    const appId = {app_id};
    const skinClass = {skin_class};
    const expected = {expected};
    const hosts = window.__CDXTHEME__ && window.__CDXTHEME__.hosts;
    const state = (hosts && hosts[appId]) || null;
    const profile = state && typeof state.verifyProfile === "function" ? state.verifyProfile() : null;
    const stylePresent = Boolean(
      document.getElementById("cdxtheme-theme-style-" + appId)
    );
    const classPresent =
      document.documentElement.classList.contains(skinClass) ||
      document.documentElement.classList.contains("cdxtheme-host-" + appId);
    const themeId = state
      ? state.themeId
      : (document.documentElement.dataset.cdxthemeTheme ||
         document.documentElement.dataset.codexSkinTheme ||
         null);
    const versionRaw = state ? state.version : (document.documentElement.dataset.cdxthemeThemeVersion || null);
    const version = versionRaw == null || versionRaw === "" ? null : Number(versionRaw);
    const expectedVersion = expected && expected.version != null ? Number(expected.version) : null;
    const themeMatches = !expected || (themeId === expected.id && version === expectedVersion);
    const imageNames = (state && state.imageNames) ? state.imageNames : [];
    const root = document.documentElement;
    const imageVars = {{}};
    for (const name of imageNames) {{
      imageVars[name] = Boolean(root && root.style.getPropertyValue("--cdxtheme-image-" + name));
    }}
    const result = {{
      appId,
      installed: Boolean(state) || (stylePresent && classPresent),
      themeId,
      version,
      stylePresent,
      classPresent,
      images: imageNames,
      imageVars,
      profile,
      warnings: (profile && profile.missing) ? profile.missing : [],
    }};
    result.pass = result.installed && result.stylePresent && themeMatches;
    return result;
  }})()"#
  )
}

async fn wait_for_compatibility(
  session: &CdpSession,
  timeout_ms: u64,
  host: &HostProfile,
) -> Result<Value, String> {
  let deadline = tokio::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
  let mut last = Value::Null;
  while tokio::time::Instant::now() < deadline {
    last = session.evaluate(host.probe_expression).await?;
    if last.get("compatible").and_then(|v| v.as_bool()) == Some(true) {
      return Ok(last);
    }
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
  }
  Ok(last)
}

/// Apply using an already-loaded theme (Codex target; preferred for Tauri).
pub async fn apply_loaded_theme(
  theme: &LoadedTheme,
  options: InjectOptions,
) -> Result<InjectRunResult, String> {
  apply_loaded_theme_for_app(APP_CODEX, theme, options).await
}

/// Apply a loaded theme to a specific host app (`codex` / `workbuddy`).
pub async fn apply_loaded_theme_for_app(
  app_id: &str,
  theme: &LoadedTheme,
  options: InjectOptions,
) -> Result<InjectRunResult, String> {
  let host = host_profile(app_id)?;
  let (expression, public) = build_inject_expression_for_app(theme, host.app_id)?;
  apply_expression(&expression, &public, options, host).await
}

/// Apply a theme package to the Codex host.
pub async fn apply_theme_package(
  package_path: impl AsRef<Path>,
  options: InjectOptions,
) -> Result<InjectRunResult, String> {
  apply_theme_package_for_app(APP_CODEX, package_path, options).await
}

/// Apply a theme package to a specific host app (`codex` / `workbuddy`).
pub async fn apply_theme_package_for_app(
  app_id: &str,
  package_path: impl AsRef<Path>,
  options: InjectOptions,
) -> Result<InjectRunResult, String> {
  let theme = load_theme_package(package_path)?;
  apply_loaded_theme_for_app(app_id, &theme, options).await
}

async fn apply_expression(
  expression: &str,
  public: &PublicTheme,
  options: InjectOptions,
  host: &HostProfile,
) -> Result<InjectRunResult, String> {
  let targets = wait_for_targets_with(options.port, options.timeout_ms, host.url_kind).await?;
  if targets.is_empty() {
    return Err(format!("no {} page targets found", host.app_id));
  }

  tracing::info!(
    "CDP inject: {} {} target(s) on port {}, host={}, payload {} bytes, theme={}",
    targets.len(),
    host.url_kind.label(),
    options.port,
    host.app_id,
    expression.len(),
    public.id
  );

  // Dream-Skin: one connected session per target — probe, early-script, inject, verify.
  let generation = format!("{}@{}", public.id, public.version);
  let early_source = early_payload_for(expression, &generation, host);
  let mut results = Vec::new();
  let mut any_hard_fail = false;
  let verify_expr = verify_expression(Some(public), host);
  let preflight_timeout = options.timeout_ms.clamp(2_000, 8_000);

  // Drop previous early scripts (tracked ids) by opening sessions later if needed.
  let mut next_early_ids: HashMap<String, String> = HashMap::new();

  for target in &targets {
    let session = match CdpSession::open(target, options.timeout_ms).await {
      Ok(s) => s,
      Err(e) => {
        tracing::error!("CDP open failed on {}: {e}", target.id);
        any_hard_fail = true;
        results.push(TargetResult {
          target_id: target.id.clone(),
          title: target.title.clone(),
          url: target.url.clone(),
          result: serde_json::json!({ "pass": false, "error": e }),
        });
        continue;
      }
    };

    // Wait for host shell markers; inject even if incomplete.
    match wait_for_compatibility(&session, preflight_timeout, host).await {
      Ok(value) => {
        if value.get("compatible").and_then(|v| v.as_bool()) == Some(true) {
          tracing::info!("{} probe ok on {} ({})", host.app_id, target.id, value);
        } else {
          tracing::warn!(
            "{} probe incomplete on {} (injecting anyway): {}",
            host.app_id,
            target.id,
            value
          );
        }
      }
      Err(e) => tracing::warn!("{} probe error on {}: {e}", host.app_id, target.id),
    }

    // Remove previous early script for this target if we still know the id.
    let old_early_id = EARLY_SCRIPT_IDS
      .lock()
      .ok()
      .and_then(|g| g.get(&target.id).cloned());
    if let Some(old_id) = old_early_id
      && let Err(e) = session.remove_script_on_new_document(&old_id).await
    {
      tracing::debug!("remove old early script on {}: {e}", target.id);
    }

    // Register early inject so SPA document reloads re-apply the skin.
    match session.add_script_on_new_document(&early_source).await {
      Ok(Some(id)) => {
        tracing::info!("registered early inject script on {} id={id}", target.id);
        next_early_ids.insert(target.id.clone(), id);
      }
      Ok(None) => tracing::warn!("early inject on {} returned no identifier", target.id),
      Err(e) => tracing::warn!("early inject register failed on {}: {e}", target.id),
    }

    // Immediate inject for the current document.
    if let Err(e) = session.evaluate(expression).await {
      tracing::error!("inject evaluate failed on {}: {e}", target.id);
      session.close().await;
      any_hard_fail = true;
      results.push(TargetResult {
        target_id: target.id.clone(),
        title: target.title.clone(),
        url: target.url.clone(),
        result: serde_json::json!({ "pass": false, "error": format!("inject: {e}") }),
      });
      continue;
    }
    tracing::info!("inject evaluate ok on {}", target.id);

    // Settle + verify (retry — chrome/DOM may lag after large image atob).
    let mut last_verify = Value::Null;
    let mut passed = false;
    for attempt in 0..8 {
      tokio::time::sleep(std::time::Duration::from_millis(if attempt == 0 {
        400
      } else {
        350
      }))
      .await;
      match session.evaluate(&verify_expr).await {
        Ok(value) => {
          last_verify = value;
          if last_verify.get("pass").and_then(|v| v.as_bool()) == Some(true) {
            passed = true;
            break;
          }
        }
        Err(e) => {
          last_verify = serde_json::json!({ "pass": false, "error": format!("verify: {e}") });
        }
      }
    }

    session.close().await;

    if !passed {
      let installed = last_verify
        .get("installed")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
      let style = last_verify
        .get("stylePresent")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
      if installed || style {
        tracing::warn!(
          "theme inject on {} soft-pass (skin present, verify incomplete): {}",
          target.id,
          last_verify
        );
        if let Some(obj) = last_verify.as_object_mut() {
          obj.insert("pass".into(), Value::Bool(true));
          obj.insert("softPass".into(), Value::Bool(true));
        }
      } else {
        tracing::error!("theme inject hard-fail on {}: {}", target.id, last_verify);
        any_hard_fail = true;
      }
    } else {
      tracing::info!("theme inject verified on {}", target.id);
    }

    results.push(TargetResult {
      target_id: target.id.clone(),
      title: target.title.clone(),
      url: target.url.clone(),
      result: last_verify,
    });
  }

  // Commit new early-script map (replace old).
  if let Ok(mut guard) = EARLY_SCRIPT_IDS.lock() {
    *guard = next_early_ids;
  }

  if results.is_empty() {
    return Err(format!("no {} page targets found", host.app_id));
  }
  if any_hard_fail {
    return Err(format!(
      "theme apply failed: {}",
      serde_json::to_string(&results).unwrap_or_default()
    ));
  }

  Ok(InjectRunResult {
    mode: "once".into(),
    port: options.port,
    theme_id: public.id.clone(),
    targets: results,
  })
}

/// Remove the injected skin from all live Codex renderer targets.
pub async fn restore_default_theme(options: InjectOptions) -> Result<InjectRunResult, String> {
  restore_default_theme_for_app(APP_CODEX, options).await
}

/// Remove the injected skin from a specific host app.
pub async fn restore_default_theme_for_app(
  app_id: &str,
  options: InjectOptions,
) -> Result<InjectRunResult, String> {
  let host = host_profile(app_id)?;
  let targets = wait_for_targets_with(options.port, options.timeout_ms, host.url_kind).await?;
  let mut results = Vec::new();
  let mut any_fail = false;

  // Snapshot + clear early-script ids so reloads don't re-apply the theme.
  let early_ids: HashMap<String, String> = EARLY_SCRIPT_IDS
    .lock()
    .map(|g| g.clone())
    .unwrap_or_default();
  if let Ok(mut g) = EARLY_SCRIPT_IDS.lock() {
    g.clear();
  }

  for target in &targets {
    let session = match CdpSession::open(target, options.timeout_ms).await {
      Ok(s) => s,
      Err(e) => {
        any_fail = true;
        results.push(TargetResult {
          target_id: target.id.clone(),
          title: target.title.clone(),
          url: target.url.clone(),
          result: serde_json::json!({ "pass": false, "error": e }),
        });
        continue;
      }
    };
    if let Some(id) = early_ids.get(&target.id)
      && let Err(e) = session.remove_script_on_new_document(id).await
    {
      tracing::debug!("remove early script on restore {}: {e}", target.id);
    }
    let result = session.evaluate(host.remove_expression).await;
    session.close().await;
    match result {
      Ok(value) => {
        if value.as_bool() == Some(false) {
          any_fail = true;
        }
        results.push(TargetResult {
          target_id: target.id.clone(),
          title: target.title.clone(),
          url: target.url.clone(),
          result: value,
        });
      }
      Err(e) => {
        any_fail = true;
        results.push(TargetResult {
          target_id: target.id.clone(),
          title: target.title.clone(),
          url: target.url.clone(),
          result: serde_json::json!({ "pass": false, "error": e }),
        });
      }
    }
  }

  if results.is_empty() {
    return Err(format!("no {} page targets found", host.app_id));
  }
  if any_fail {
    return Err(format!(
      "restore completed with failures: {}",
      serde_json::to_string(&results).unwrap_or_default()
    ));
  }

  Ok(InjectRunResult {
    mode: "remove".into(),
    port: options.port,
    theme_id: String::new(),
    targets: results,
  })
}

/// Verify currently injected theme state on Codex targets.
pub async fn verify_theme(
  expected: Option<&PublicTheme>,
  options: InjectOptions,
) -> Result<InjectRunResult, String> {
  verify_theme_for_app(APP_CODEX, expected, options).await
}

/// Verify currently injected theme state for a specific host app.
pub async fn verify_theme_for_app(
  app_id: &str,
  expected: Option<&PublicTheme>,
  options: InjectOptions,
) -> Result<InjectRunResult, String> {
  let host = host_profile(app_id)?;
  let targets = wait_for_targets_with(options.port, options.timeout_ms, host.url_kind).await?;
  let expr = verify_expression(expected, host);
  let mut results = Vec::new();

  for target in &targets {
    let session = CdpSession::open(target, options.timeout_ms).await?;
    let result = session.evaluate(&expr).await;
    session.close().await;
    match result {
      Ok(value) => results.push(TargetResult {
        target_id: target.id.clone(),
        title: target.title.clone(),
        url: target.url.clone(),
        result: value,
      }),
      Err(e) => results.push(TargetResult {
        target_id: target.id.clone(),
        title: target.title.clone(),
        url: target.url.clone(),
        result: serde_json::json!({ "pass": false, "error": e }),
      }),
    }
  }

  Ok(InjectRunResult {
    mode: "verify".into(),
    port: options.port,
    theme_id: expected.map(|t| t.id.clone()).unwrap_or_default(),
    targets: results,
  })
}
