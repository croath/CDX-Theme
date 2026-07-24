//! Bridge to the PostHog JS snippet in `index.html` (key from build-time config).

use js_sys::{Function, Object, Reflect};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::window;

fn bridge() -> Option<Object> {
  let win = window()?;
  let v = Reflect::get(win.as_ref(), &JsValue::from_str("__cdxPosthog")).ok()?;
  v.dyn_into::<Object>().ok()
}

fn call1(method: &str, arg: &JsValue) {
  let Some(bridge) = bridge() else {
    return;
  };
  let Ok(fn_v) = Reflect::get(bridge.as_ref(), &JsValue::from_str(method)) else {
    return;
  };
  let Ok(func) = fn_v.dyn_into::<Function>() else {
    return;
  };
  let _ = func.call1(bridge.as_ref(), arg);
}

/// Sync JS capture with the Settings preference (opt-in / opt-out).
pub fn set_enabled(enabled: bool) {
  call1("setEnabled", &JsValue::from_bool(enabled));
}

/// Align posthog-js person with the install's anonymous `distinct_id`.
pub fn identify(distinct_id: &str) {
  if distinct_id.trim().is_empty() {
    return;
  }
  call1("identify", &JsValue::from_str(distinct_id));
}

/// Capture a custom event via `posthog-js` (no-op when opted out / not configured).
pub fn capture(event: &str, props: Option<&Object>) {
  let Some(bridge) = bridge() else {
    return;
  };
  let Ok(fn_v) = Reflect::get(bridge.as_ref(), &JsValue::from_str("capture")) else {
    return;
  };
  let Ok(func) = fn_v.dyn_into::<Function>() else {
    return;
  };
  let empty = Object::new();
  let props_ref = props.unwrap_or(&empty);
  let _ = func.call2(
    bridge.as_ref(),
    &JsValue::from_str(event),
    props_ref.as_ref(),
  );
}

fn page_props(page: &str) -> Object {
  let props = Object::new();
  let path = format!("/{page}");
  let url = format!("cdxtheme://{page}");
  let _ = Reflect::set(
    props.as_ref(),
    &JsValue::from_str("$current_url"),
    &JsValue::from_str(&url),
  );
  let _ = Reflect::set(
    props.as_ref(),
    &JsValue::from_str("$pathname"),
    &JsValue::from_str(&path),
  );
  let _ = Reflect::set(
    props.as_ref(),
    &JsValue::from_str("$host"),
    &JsValue::from_str("cdxtheme"),
  );
  let _ = Reflect::set(
    props.as_ref(),
    &JsValue::from_str("page"),
    &JsValue::from_str(page),
  );
  props
}

fn call_page_method(method: &str, page: &str) -> bool {
  let Some(bridge) = bridge() else {
    return false;
  };
  let Ok(fn_v) = Reflect::get(bridge.as_ref(), &JsValue::from_str(method)) else {
    return false;
  };
  let Ok(func) = fn_v.dyn_into::<Function>() else {
    return false;
  };
  let _ = func.call1(bridge.as_ref(), &JsValue::from_str(page));
  true
}

/// Capture PostHog's standard `$pageview` (required for install verification).
/// Also emits `$pageleave` for the previous page via the JS bridge.
pub fn capture_pageview(page: &str) {
  if call_page_method("capturePageview", page) {
    return;
  }
  capture("$pageview", Some(&page_props(page)));
}

/// Capture PostHog's standard `$pageleave` (navigation away / app hide).
pub fn capture_pageleave(page: Option<&str>) {
  if let Some(page) = page {
    if call_page_method("capturePageleave", page) {
      return;
    }
    capture("$pageleave", Some(&page_props(page)));
    return;
  }
  // No page arg → leave current page tracked in JS.
  let Some(bridge) = bridge() else {
    return;
  };
  if let Ok(fn_v) = Reflect::get(bridge.as_ref(), &JsValue::from_str("capturePageleave")) {
    if let Ok(func) = fn_v.dyn_into::<Function>() {
      let _ = func.call0(bridge.as_ref());
    }
  }
}

/// Apply backend analytics state to the web SDK (identify + opt-in).
/// When enabling, returns `true` so the caller can immediately send `$pageview`.
pub fn apply_state(enabled: bool, distinct_id: &str) -> bool {
  if !distinct_id.is_empty() {
    identify(distinct_id);
  }
  set_enabled(enabled);
  enabled
}
