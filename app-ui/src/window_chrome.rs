//! Window drag helpers for seamless transparent/overlay titlebars.
//! Prefer `startDragging()` over CSS `data-tauri-drag-region` (which does not apply to children).

use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::MouseEvent;

/// Interactive targets that should not initiate window drag.
fn is_interactive_target(target: &web_sys::EventTarget) -> bool {
  let Some(el) = target.dyn_ref::<web_sys::Element>() else {
    return false;
  };

  // Only real controls block drag — not the whole main/scroll surface,
  // so the right-hand content area can still move the window.
  let selector = concat!(
    "button, a, input, textarea, select, option, label, summary,",
    "[role='button'], [role='link'], [role='menuitem'], [role='switch'],",
    "[role='tab'], [role='checkbox'], [role='radio'], [role='listbox'],",
    "[role='option'], [role='slider'], [role='textbox'], [role='alertdialog'],",
    "[role='dialog'], [contenteditable='true'], .no-drag,",
    "iframe, video, canvas"
  );

  if el.closest(selector).ok().flatten().is_some() {
    return true;
  }

  false
}

/// Primary-button mousedown → start native window drag (unless interactive).
/// Double-click → toggle maximize.
pub fn on_window_pointer_down(ev: MouseEvent) {
  // Only left button
  if ev.button() != 0 {
    return;
  }

  let Some(target) = ev.target() else {
    return;
  };
  if is_interactive_target(&target) {
    return;
  }

  // Don't steal text selection in inputs (already filtered) or multi-click quirks.
  if ev.detail() >= 2 {
    spawn_local(async move {
      let _ = toggle_maximize().await;
    });
    return;
  }

  spawn_local(async move {
    let _ = start_dragging().await;
  });
}

async fn start_dragging() -> Result<(), String> {
  let win = current_window()?;
  call_async_method(&win, "startDragging").await
}

async fn toggle_maximize() -> Result<(), String> {
  let win = current_window()?;
  call_async_method(&win, "toggleMaximize").await
}

fn current_window() -> Result<JsValue, String> {
  let window = web_sys::window().ok_or_else(|| "no window".to_string())?;
  let tauri = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__"))
    .map_err(|_| "__TAURI__ missing".to_string())?;
  if tauri.is_undefined() || tauri.is_null() {
    return Err("__TAURI__ missing".into());
  }
  let win_ns = js_sys::Reflect::get(&tauri, &JsValue::from_str("window"))
    .map_err(|_| "window ns missing".to_string())?;
  let get_current = js_sys::Reflect::get(&win_ns, &JsValue::from_str("getCurrentWindow"))
    .map_err(|_| "getCurrentWindow missing".to_string())?;
  let get_fn = get_current
    .dyn_into::<js_sys::Function>()
    .map_err(|_| "getCurrentWindow not a function".to_string())?;
  get_fn
    .call0(&win_ns)
    .map_err(|_| "getCurrentWindow() failed".to_string())
}

async fn call_async_method(this: &JsValue, name: &str) -> Result<(), String> {
  let method =
    js_sys::Reflect::get(this, &JsValue::from_str(name)).map_err(|_| format!("{name} missing"))?;
  let func = method
    .dyn_into::<js_sys::Function>()
    .map_err(|_| format!("{name} not a function"))?;
  let result = func.call0(this).map_err(|_| format!("{name}() failed"))?;

  // Methods return a Promise
  if let Ok(promise) = result.dyn_into::<js_sys::Promise>() {
    wasm_bindgen_futures::JsFuture::from(promise)
      .await
      .map_err(|_| format!("{name} rejected"))?;
  }
  Ok(())
}
