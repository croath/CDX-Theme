//! Page-level host app select (Codex / ChatGPT vs WorkBuddy) for Apply.
//!
//! Only rendered when **both** host apps are detected as installed.

use crate::i18n::I18n;
use crate::state::AppCtx;
use leptos::prelude::*;

/// Segmented control: Codex / ChatGPT | WorkBuddy.
///
/// Hidden unless detect reports both apps installed.
#[component]
pub fn TargetAppSelect(
  /// Bound host id (`codex` | `workbuddy`).
  target_app: RwSignal<String>,
) -> impl IntoView {
  let ctx = AppCtx::use_ctx();

  view! {
    <Show when=move || ctx.show_target_select()>
      <div class="flex flex-col items-end gap-1 sm:flex-row sm:items-center sm:gap-2">
        <span class="text-[11px] font-medium text-muted-foreground sm:text-xs">
          {move || {
            let i18n = I18n { locale: ctx.locale.get() };
            i18n.t("recommend.apply.target")
          }}
        </span>
        <div
          class="inline-flex h-9 items-center rounded-xl border border-border/70 bg-card/80 p-0.5 shadow-sm backdrop-blur-sm"
          role="group"
          aria-label=move || {
            let i18n = I18n { locale: ctx.locale.get() };
            i18n.t("recommend.apply.target")
          }
        >
          <button
            type="button"
            class=move || {
              if target_app.get() == "codex" {
                "inline-flex h-8 items-center rounded-[10px] bg-primary px-3 text-xs font-semibold text-primary-foreground shadow-sm transition-colors sm:text-sm"
              } else {
                "inline-flex h-8 items-center rounded-[10px] px-3 text-xs font-medium text-muted-foreground transition-colors hover:text-foreground sm:text-sm"
              }
            }
            on:click=move |_| target_app.set("codex".into())
          >
            {move || {
              let i18n = I18n { locale: ctx.locale.get() };
              i18n.t("recommend.apply.codex")
            }}
          </button>
          <button
            type="button"
            class=move || {
              if target_app.get() == "workbuddy" {
                "inline-flex h-8 items-center rounded-[10px] bg-primary px-3 text-xs font-semibold text-primary-foreground shadow-sm transition-colors sm:text-sm"
              } else {
                "inline-flex h-8 items-center rounded-[10px] px-3 text-xs font-medium text-muted-foreground transition-colors hover:text-foreground sm:text-sm"
              }
            }
            on:click=move |_| target_app.set("workbuddy".into())
          >
            {move || {
              let i18n = I18n { locale: ctx.locale.get() };
              i18n.t("recommend.apply.workbuddy")
            }}
          </button>
        </div>
      </div>
    </Show>
  }
}
