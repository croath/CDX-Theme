use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;

use crate::api;
use crate::types::{Locale, Page};

const THEME_KEY: &str = "ctl-theme";
const LOCALE_KEY: &str = "ctl-locale";

#[derive(Clone, Copy)]
pub struct AppCtx {
  pub page: RwSignal<Page>,
  pub is_dark: RwSignal<bool>,
  pub locale: RwSignal<Locale>,
}

impl AppCtx {
  pub fn provide() -> Self {
    let is_dark = RwSignal::new(load_is_dark());
    let locale = RwSignal::new(load_locale());
    let page = RwSignal::new(Page::Recommend);

    let ctx = Self {
      page,
      is_dark,
      locale,
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

    provide_context(ctx);
    ctx
  }

  pub fn use_ctx() -> Self {
    use_context::<Self>().expect("AppCtx not provided")
  }

  pub fn set_theme(self, dark: bool) {
    self.is_dark.set(dark);
  }
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
  if let Some(s) = storage() {
    if let Ok(Some(v)) = s.get_item(THEME_KEY) {
      return v == "dark";
    }
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
