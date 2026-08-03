use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;

use crate::api::{self, HostAppsDetect};
use crate::types::{Locale, Page};

const THEME_KEY: &str = "ctl-theme";
const LOCALE_KEY: &str = "ctl-locale";
const TARGET_APP_KEY: &str = "ctl-target-app";

#[derive(Clone, Copy)]
pub struct AppCtx {
  pub page: RwSignal<Page>,
  pub is_dark: RwSignal<bool>,
  pub locale: RwSignal<Locale>,
  /// Host for Apply on Recommend / Library / Theme Builder (`codex` | `workbuddy`).
  pub target_app: RwSignal<String>,
  /// Detected desktop host installs (Codex + WorkBuddy).
  pub host_apps: RwSignal<HostAppsDetect>,
  /// True after the first detect attempt finishes (success or failure).
  pub host_apps_ready: RwSignal<bool>,
}

impl AppCtx {
  pub fn provide() -> Self {
    let is_dark = RwSignal::new(load_is_dark());
    let locale = RwSignal::new(load_locale());
    let page = RwSignal::new(Page::Recommend);
    let target_app = RwSignal::new(load_target_app());
    let host_apps = RwSignal::new(HostAppsDetect::default());
    let host_apps_ready = RwSignal::new(false);

    let ctx = Self {
      page,
      is_dark,
      locale,
      target_app,
      host_apps,
      host_apps_ready,
    };

    Effect::new(move |_| {
      let dark = is_dark.get();
      apply_dark_class(dark);
      persist(THEME_KEY, if dark { "dark" } else { "light" });
      // Opaque window bg under overlay titlebar (no macOS private API).
      spawn_local(async move {
        let _ = api::set_window_appearance(dark).await;
      });
    });

    Effect::new(move |_| {
      let loc = locale.get();
      persist(LOCALE_KEY, loc.code());
    });

    Effect::new(move |_| {
      let app = target_app.get();
      if app == "codex" || app == "workbuddy" {
        persist(TARGET_APP_KEY, &app);
      }
    });

    // Detect host installs once at startup; keep target_app coherent.
    Effect::new(move |_| {
      spawn_local(async move {
        match api::detect_host_apps().await {
          Ok(detect) => {
            apply_host_detect(target_app, host_apps, detect);
            host_apps_ready.set(true);
          }
          Err(e) => {
            web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(&format!(
              "detect_host_apps failed: {e}"
            )));
            // Fallback: show select (both unknown) only after ready would force hide — keep
            // codex default and mark ready so UI does not wait forever.
            host_apps_ready.set(true);
          }
        }
      });
    });

    // Sync opt-in / identify first, then `$pageview` on each navigation.
    // JS bridge also emits `$pageleave` for the previous page (and on app hide).
    // Await sync so the first pageview is not dropped while still opted out by default.
    Effect::new(move |_| {
      let page_id = page.get().analytics_id().to_string();
      spawn_local(async move {
        let _ = api::sync_posthog_js().await;
        api::track_page_viewed(&page_id).await;
      });
    });

    provide_context(ctx);
    ctx
  }

  pub fn use_ctx() -> Self {
    use_context::<Self>().expect("AppCtx not provided")
  }

  pub fn set_theme(self, dark: bool) {
    self.is_dark.set(dark);
  }

  /// Re-run host install detection (e.g. after user installs an app).
  #[allow(dead_code)]
  pub fn refresh_host_apps(self) {
    spawn_local(async move {
      if let Ok(detect) = api::detect_host_apps().await {
        apply_host_detect(self.target_app, self.host_apps, detect);
        self.host_apps_ready.set(true);
      }
    });
  }

  /// Show the dual-host select only when both apps are installed.
  pub fn show_target_select(self) -> bool {
    self.host_apps_ready.get() && self.host_apps.get().both_installed()
  }
}

fn apply_host_detect(
  target_app: RwSignal<String>,
  host_apps: RwSignal<HostAppsDetect>,
  detect: HostAppsDetect,
) {
  host_apps.set(detect.clone());
  // When only one host is installed, pin Apply to that host.
  if let Some(sole) = detect.sole_target() {
    target_app.set(sole.to_string());
  } else if !detect.both_installed() {
    // Neither installed: keep a safe default.
    if target_app.get_untracked() != "codex" && target_app.get_untracked() != "workbuddy" {
      target_app.set("codex".into());
    }
  }
  // Both installed: keep persisted preference.
}

fn window() -> Option<web_sys::Window> {
  web_sys::window()
}

fn storage() -> Option<web_sys::Storage> {
  window()?.local_storage().ok().flatten()
}

fn persist(key: &str, value: &str) {
  if let Some(s) = storage() {
    let _ = s.set_item(key, value);
  }
}

fn load_is_dark() -> bool {
  if let Some(s) = storage()
    && let Ok(Some(v)) = s.get_item(THEME_KEY)
  {
    return v == "dark";
  }
  window()
    .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok().flatten())
    .map(|m| m.matches())
    .unwrap_or(false)
}

fn load_locale() -> Locale {
  storage()
    .and_then(|s| s.get_item(LOCALE_KEY).ok().flatten())
    .map(|c| Locale::from_code(&c))
    .unwrap_or_default()
}

fn load_target_app() -> String {
  match storage()
    .and_then(|s| s.get_item(TARGET_APP_KEY).ok().flatten())
    .as_deref()
  {
    Some("workbuddy") => "workbuddy".into(),
    _ => "codex".into(),
  }
}

fn apply_dark_class(is_dark: bool) {
  let Some(document) = window().and_then(|w| w.document()) else {
    return;
  };
  let Some(el) = document.document_element() else {
    return;
  };
  let Ok(el) = el.dyn_into::<web_sys::Element>() else {
    return;
  };
  let class_list = el.class_list();
  if is_dark {
    let _ = class_list.add_1("dark");
  } else {
    let _ = class_list.remove_1("dark");
  }
}
