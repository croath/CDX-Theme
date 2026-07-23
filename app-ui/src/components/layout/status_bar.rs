use crate::api::{self, CdpServerStatus};
use crate::i18n::I18n;
use crate::state::AppCtx;
use leptos::prelude::*;
use leptos::task::spawn_local;

/// App version shown in the status bar (matches package / Tauri product version).
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Thin status bar for the bottom of the main (right) panel.
#[component]
pub fn StatusBar() -> impl IntoView {
  let ctx = AppCtx::use_ctx();
  let status = RwSignal::new(CdpServerStatus::default());

  Effect::new(move |_| {
    spawn_local(async move {
      if let Ok(s) = api::cdp_status().await {
        status.set(s);
      }
      loop {
        sleep_ms(2000).await;
        match api::cdp_status().await {
          Ok(s) => status.set(s),
          Err(e) => {
            status.update(|cur| {
              cur.connected = false;
              cur.target_count = 0;
              cur.targets.clear();
              cur.message = e;
            });
          }
        }
      }
    });
  });

  view! {
    <footer
      class="status-bar no-drag flex h-8 shrink-0 items-center justify-end gap-3 border-t border-border/40 bg-background/60 px-4 backdrop-blur-md"
      title=move || {
        let s = status.get();
        format!("v{APP_VERSION} · port {} — {}", s.port, s.message)
      }
    >
      <span class="font-mono text-[11px] text-muted-foreground">
        {format!("v{APP_VERSION}")}
      </span>

      <span class="text-border/80 text-[11px]" aria-hidden="true">"·"</span>

      <span class=move || {
        if status.get().connected {
          "inline-flex shrink-0 items-center gap-1.5 text-[11px] font-medium text-primary"
        } else {
          "inline-flex shrink-0 items-center gap-1.5 text-[11px] font-medium text-destructive"
        }
      }>
        <span class=move || {
          if status.get().connected {
            "size-1.5 shrink-0 rounded-full bg-primary shadow-[0_0_6px] shadow-primary"
          } else {
            "size-1.5 shrink-0 rounded-full bg-destructive"
          }
        } />
        {move || {
          let i18n = I18n { locale: ctx.locale.get() };
          if status.get().connected {
            i18n.t("settings.cdp.connected")
          } else {
            i18n.t("settings.cdp.disconnected")
          }
        }}
      </span>
    </footer>
  }
}

async fn sleep_ms(ms: i32) {
  use wasm_bindgen::JsCast;
  use wasm_bindgen::closure::Closure;
  use wasm_bindgen_futures::JsFuture;

  let Some(window) = web_sys::window() else {
    return;
  };
  let promise = js_sys::Promise::new(&mut |resolve, _reject| {
    let cb = Closure::once_into_js(move || {
      let _ = resolve.call0(&wasm_bindgen::JsValue::NULL);
    });
    let _ =
      window.set_timeout_with_callback_and_timeout_and_arguments_0(cb.as_ref().unchecked_ref(), ms);
  });
  let _ = JsFuture::from(promise).await;
}
