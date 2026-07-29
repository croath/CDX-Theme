//! Theme Builder gate: require host `bunx` or `npx` (or `codex-acp`) before use.

use icons::{Download, LoaderCircle, Terminal, WandSparkles};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, ThemeBuilderRuntimeStatus};
use crate::components::ui::sonner::{toast_error, toast_success};
use crate::i18n::I18n;
use crate::state::AppCtx;

/// Shown when the host has neither Bun (`bunx`) nor Node (`npx`) nor a local `codex-acp`.
#[component]
pub(super) fn BuilderRuntimeSetup(
  status: RwSignal<Option<ThemeBuilderRuntimeStatus>>,
  on_ready: Callback<ThemeBuilderRuntimeStatus>,
  on_recheck: Callback<()>,
) -> impl IntoView {
  let ctx = AppCtx::use_ctx();
  let installing = RwSignal::new(false);
  let install_log = RwSignal::new(Option::<String>::None);

  let run_install = move |_| {
    if installing.get_untracked() {
      return;
    }
    installing.set(true);
    install_log.set(None);
    let locale = ctx.locale.get_untracked();
    spawn_local(async move {
      let i18n = I18n { locale };
      match api::install_bun_for_theme_builder().await {
        Ok(next) => {
          status.set(Some(next.clone()));
          installing.set(false);
          if next.ready {
            toast_success(i18n.t("builder.runtime.install.success"), &next.message);
            on_ready.run(next);
          } else {
            let msg = next.message.clone();
            install_log.set(Some(msg.clone()));
            toast_error(i18n.t("builder.error"), &msg);
          }
        }
        Err(e) => {
          installing.set(false);
          install_log.set(Some(e.clone()));
          toast_error(i18n.t("builder.error"), &e);
        }
      }
    });
  };

  view! {
    <div class="flex h-full min-h-0 w-full flex-1 flex-col">
      <header class="mb-5 shrink-0">
        <div class="flex items-start gap-3">
          <div class="mt-0.5 flex size-10 shrink-0 items-center justify-center rounded-2xl bg-gradient-to-br from-primary/25 to-chart-2/20 text-primary ring-1 ring-primary/25">
            <WandSparkles class="size-5" />
          </div>
          <div class="min-w-0 flex-1">
            <h1 class="bg-gradient-to-r from-foreground via-foreground to-primary bg-clip-text text-2xl font-semibold tracking-tight text-transparent sm:text-3xl">
              {move || {
                let i18n = I18n { locale: ctx.locale.get() };
                i18n.t("builder.runtime.title")
              }}
            </h1>
            <p class="mt-1 max-w-2xl text-sm text-muted-foreground">
              {move || {
                let i18n = I18n { locale: ctx.locale.get() };
                i18n.t("builder.runtime.subtitle")
              }}
            </p>
          </div>
        </div>
      </header>

      <div class="mx-auto flex w-full max-w-xl flex-1 flex-col justify-center pb-8">
        <div class="overflow-hidden rounded-3xl border border-border/70 bg-card/80 shadow-xl shadow-black/5 backdrop-blur-xl">
          <div class="border-b border-border/40 bg-gradient-to-r from-primary/10 via-transparent to-chart-2/10 px-5 py-4">
            <div class="flex items-center gap-2">
              <Terminal class="size-4 text-primary" />
              <h2 class="text-sm font-semibold text-foreground">
                {move || {
                  let i18n = I18n { locale: ctx.locale.get() };
                  i18n.t("builder.runtime.need.title")
                }}
              </h2>
            </div>
          </div>

          <div class="space-y-4 px-5 py-5">
            <p class="text-sm leading-relaxed text-muted-foreground">
              {move || {
                let i18n = I18n { locale: ctx.locale.get() };
                i18n.t("builder.runtime.need.body")
              }}
            </p>

            <ul class="space-y-2 rounded-2xl border border-border/50 bg-background/50 px-4 py-3 text-xs text-muted-foreground">
              <li class="flex gap-2">
                <span class="font-semibold text-foreground">"1."</span>
                <span>
                  {move || {
                    let i18n = I18n { locale: ctx.locale.get() };
                    i18n.t("builder.runtime.step.install")
                  }}
                </span>
              </li>
              <li class="flex gap-2">
                <span class="font-semibold text-foreground">"2."</span>
                <span>
                  {move || {
                    let i18n = I18n { locale: ctx.locale.get() };
                    i18n.t("builder.runtime.step.mirrors")
                  }}
                </span>
              </li>
              <li class="flex gap-2">
                <span class="font-semibold text-foreground">"3."</span>
                <span>
                  {move || {
                    let i18n = I18n { locale: ctx.locale.get() };
                    i18n.t("builder.runtime.step.continue")
                  }}
                </span>
              </li>
            </ul>

            <div class="rounded-2xl border border-dashed border-border/60 bg-muted/20 px-3.5 py-2.5 font-mono text-[11px] leading-relaxed text-muted-foreground">
              {move || {
                status
                  .get()
                  .map(|s| s.message)
                  .unwrap_or_else(|| {
                    let i18n = I18n { locale: ctx.locale.get() };
                    i18n.t("builder.runtime.checking").to_string()
                  })
              }}
            </div>

            <Show when=move || install_log.get().is_some()>
              <div class="max-h-28 overflow-y-auto rounded-2xl border border-destructive/30 bg-destructive/5 px-3.5 py-2.5 font-mono text-[11px] leading-relaxed text-destructive">
                {move || install_log.get().unwrap_or_default()}
              </div>
            </Show>

            <div class="flex flex-col gap-2 sm:flex-row">
              <button
                type="button"
                class="inline-flex h-11 flex-1 items-center justify-center gap-2 rounded-2xl bg-primary px-4 text-sm font-semibold text-primary-foreground shadow-lg shadow-primary/25 transition-all hover:bg-primary/90 active:scale-[0.98] disabled:pointer-events-none disabled:opacity-50"
                prop:disabled=move || installing.get()
                on:click=run_install
              >
                {move || {
                  if installing.get() {
                    view! {
                      <LoaderCircle class="size-4 animate-spin" />
                      <span>
                        {move || {
                          let i18n = I18n { locale: ctx.locale.get() };
                          i18n.t("builder.runtime.installing")
                        }}
                      </span>
                    }.into_any()
                  } else {
                    view! {
                      <Download class="size-4" />
                      <span>
                        {move || {
                          let i18n = I18n { locale: ctx.locale.get() };
                          i18n.t("builder.runtime.install")
                        }}
                      </span>
                    }.into_any()
                  }
                }}
              </button>
              <button
                type="button"
                class="inline-flex h-11 items-center justify-center gap-2 rounded-2xl border border-border/70 bg-background/70 px-4 text-sm font-medium text-foreground transition-colors hover:bg-accent disabled:opacity-50"
                prop:disabled=move || installing.get()
                on:click=move |_| on_recheck.run(())
              >
                {move || {
                  let i18n = I18n { locale: ctx.locale.get() };
                  i18n.t("builder.runtime.recheck")
                }}
              </button>
            </div>

            <p class="text-[11px] leading-relaxed text-muted-foreground">
              {move || {
                let i18n = I18n { locale: ctx.locale.get() };
                i18n.t("builder.runtime.manual")
              }}
            </p>
          </div>
        </div>
      </div>
    </div>
  }
}
