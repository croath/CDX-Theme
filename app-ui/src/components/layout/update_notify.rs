use icons::{Check, Download, LoaderCircle};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, AppUpdateStatus};
use crate::i18n::I18n;
use crate::state::AppCtx;

/// Sonner-style app update card fixed to the bottom of the left nav.
///
/// States: available (Download) → downloading (progress) → ready (Install) → installing.
#[component]
pub fn UpdateNotify() -> impl IntoView {
  let ctx = AppCtx::use_ctx();
  let status = RwSignal::new(AppUpdateStatus::default());
  let action_busy = RwSignal::new(false);
  let subscribed = RwSignal::new(false);

  // Subscribe once (sidebar is app-lifetime).
  Effect::new(move |_| {
    if subscribed.get_untracked() {
      return;
    }
    subscribed.set(true);
    spawn_local(async move {
      if let Ok(s) = api::get_app_update_status().await {
        status.set(s);
      }
      match api::listen_app_update(move |ev| {
        let phase = ev.phase.clone();
        // Once downloaded, never drop install status for idle/empty events
        // (backend also enforces this; this is a UI safety net).
        let cur = status.get_untracked();
        let keep_ready = matches!(cur.phase.as_str(), "ready" | "installing")
          && (phase == "idle" || phase.is_empty());
        if keep_ready {
          return;
        }
        status.set(ev);
        if phase != "downloading" && phase != "installing" {
          action_busy.set(false);
        }
      })
      .await
      {
        // Keep the unlisten handle alive for the app session.
        Ok(handle) => std::mem::forget(handle),
        Err(e) => {
          web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(&format!(
            "app-update listen failed: {e}"
          )));
        }
      }
    });
  });

  let on_download = move |_| {
    if action_busy.get_untracked() {
      return;
    }
    action_busy.set(true);
    spawn_local(async move {
      if let Err(e) = api::download_app_update().await {
        status.update(|s| {
          s.phase = "error".into();
          s.error = Some(e);
        });
        action_busy.set(false);
      }
    });
  };

  let on_install = move |_| {
    if action_busy.get_untracked() {
      return;
    }
    action_busy.set(true);
    spawn_local(async move {
      if let Err(e) = api::install_app_update().await {
        status.update(|s| {
          // Stay on ready so user can retry Install.
          s.phase = "ready".into();
          s.error = Some(e);
        });
        action_busy.set(false);
      }
    });
  };

  view! {
    <Show when=move || status.get().is_visible()>
      <div class="relative z-10 px-3 pb-3 pt-1">
        <div
          class="pointer-events-auto flex w-full flex-col gap-2.5 rounded-2xl border border-primary/25 bg-card/95 p-3 shadow-xl shadow-primary/10 backdrop-blur-md animate-in fade-in-0 slide-in-from-bottom-2 duration-200"
          role="status"
          aria-live="polite"
        >
          <div class="flex items-start gap-2.5">
            <span class="mt-0.5 flex size-8 shrink-0 items-center justify-center rounded-xl bg-primary/15 text-primary">
              {move || {
                let phase = status.get().phase;
                match phase.as_str() {
                  "downloading" | "installing" => {
                    view! { <LoaderCircle class="size-4 animate-spin" /> }.into_any()
                  }
                  "ready" => view! { <Check class="size-4" /> }.into_any(),
                  _ => view! { <Download class="size-4" /> }.into_any(),
                }
              }}
            </span>

            <div class="min-w-0 flex-1 pt-0.5">
              <p class="text-sm font-semibold leading-tight tracking-tight text-foreground">
                {move || {
                  let i18n = I18n { locale: ctx.locale.get() };
                  let phase = status.get().phase;
                  match phase.as_str() {
                    "downloading" => i18n.t("update.downloading"),
                    "ready" => i18n.t("update.ready"),
                    "installing" => i18n.t("update.installing"),
                    "error" => i18n.t("update.error"),
                    _ => i18n.t("update.available"),
                  }
                }}
              </p>
              <p class="mt-0.5 text-[11px] leading-relaxed text-muted-foreground">
                {move || {
                  let i18n = I18n { locale: ctx.locale.get() };
                  let s = status.get();
                  if let Some(err) = s.error.as_ref().filter(|e| !e.is_empty()) {
                    return err.clone();
                  }
                  let hint = match s.phase.as_str() {
                    "ready" | "installing" => i18n.t("update.hint.ready"),
                    "downloading" => i18n.t("update.downloading"),
                    _ => i18n.t("update.hint"),
                  };
                  if s.version.is_empty() {
                    return hint.to_string();
                  }
                  let cur = if s.current_version.is_empty() {
                    "?"
                  } else {
                    s.current_version.as_str()
                  };
                  format!("{cur} → {} · {hint}", s.version)
                }}
              </p>
            </div>
          </div>

          // Progress bar while downloading
          <Show when=move || status.get().phase == "downloading">
            <div class="space-y-1.5">
              <div class="h-1.5 w-full overflow-hidden rounded-full bg-muted">
                <div
                  class="h-full rounded-full bg-primary transition-[width] duration-200 ease-out"
                  style=move || {
                    let s = status.get();
                    let pct = s.percent.unwrap_or(0).min(100);
                    // Indeterminate-ish when total unknown: pulse via min width
                    if s.total.is_none() && s.percent.is_none() {
                      "width: 35%".to_string()
                    } else {
                      format!("width: {pct}%")
                    }
                  }
                />
              </div>
              <p class="text-[10px] font-medium tabular-nums text-muted-foreground">
                {move || {
                  let i18n = I18n { locale: ctx.locale.get() };
                  let s = status.get();
                  if let Some(pct) = s.percent {
                    format!("{pct}%")
                  } else if s.downloaded > 0 {
                    format_bytes(s.downloaded)
                  } else {
                    i18n.t("update.downloading").to_string()
                  }
                }}
              </p>
            </div>
          </Show>

          // Action: Download → Install (after download)
          <Show when=move || {
            let phase = status.get().phase;
            matches!(phase.as_str(), "available" | "ready" | "error" | "installing")
          }>
            <div class="flex items-center gap-2">
              {move || {
                let phase = status.get().phase;
                let busy = action_busy.get() || phase == "installing";
                let i18n = I18n { locale: ctx.locale.get() };

                if phase == "ready" || (phase == "error" && status.get().percent == Some(100)) {
                  // Downloaded → Install
                  view! {
                    <button
                      type="button"
                      class="inline-flex h-8 flex-1 items-center justify-center gap-1.5 rounded-xl bg-primary px-3 text-xs font-semibold text-primary-foreground shadow-sm shadow-primary/25 transition-colors hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-60"
                      disabled=busy
                      on:click=on_install
                    >
                      <Show
                        when=move || busy
                        fallback=move || {
                          view! {
                            <Check class="size-3.5" />
                            <span>{i18n.t("update.install")}</span>
                          }
                        }
                      >
                        <LoaderCircle class="size-3.5 animate-spin" />
                        <span>{i18n.t("update.installing")}</span>
                      </Show>
                    </button>
                  }
                  .into_any()
                } else if phase == "installing" {
                  view! {
                    <button
                      type="button"
                      class="inline-flex h-8 flex-1 items-center justify-center gap-1.5 rounded-xl bg-primary px-3 text-xs font-semibold text-primary-foreground opacity-80"
                      disabled=true
                    >
                      <LoaderCircle class="size-3.5 animate-spin" />
                      <span>{i18n.t("update.installing")}</span>
                    </button>
                  }
                  .into_any()
                } else {
                  // Available / download error → Download
                  view! {
                    <button
                      type="button"
                      class="inline-flex h-8 flex-1 items-center justify-center gap-1.5 rounded-xl bg-primary px-3 text-xs font-semibold text-primary-foreground shadow-sm shadow-primary/25 transition-colors hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-60"
                      disabled=busy
                      on:click=on_download
                    >
                      <Show
                        when=move || busy
                        fallback=move || {
                          view! {
                            <Download class="size-3.5" />
                            <span>{i18n.t("update.download")}</span>
                          }
                        }
                      >
                        <LoaderCircle class="size-3.5 animate-spin" />
                        <span>{i18n.t("update.downloading")}</span>
                      </Show>
                    </button>
                  }
                  .into_any()
                }
              }}
            </div>
          </Show>
        </div>
      </div>
    </Show>
  }
}

fn format_bytes(n: u64) -> String {
  const KB: f64 = 1024.0;
  const MB: f64 = KB * 1024.0;
  let n = n as f64;
  if n >= MB {
    format!("{:.1} MB", n / MB)
  } else if n >= KB {
    format!("{:.0} KB", n / KB)
  } else {
    format!("{n:.0} B")
  }
}
