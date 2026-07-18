use icons::{Check, LoaderCircle, RotateCcw};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api;
use crate::components::ui::sonner::toast_error;
use crate::i18n::I18n;
use crate::state::AppCtx;

#[component]
pub fn RestorePage() -> impl IntoView {
  let ctx = AppCtx::use_ctx();
  let loading = RwSignal::new(false);
  let success = RwSignal::new(false);

  let on_restore = move |_| {
    if loading.get_untracked() {
      return;
    }
    loading.set(true);
    success.set(false);
    let locale = ctx.locale.get_untracked();
    spawn_local(async move {
      match api::restore_theme().await {
        Ok(()) => {
          success.set(true);
          loading.set(false);
        }
        Err(e) => {
          loading.set(false);
          let i18n = I18n { locale };
          toast_error(i18n.t("restore.error"), &e);
        }
      }
    });
  };

  view! {
    <div class="flex h-full flex-col">
      <header class="mb-6">
        <h1 class="bg-gradient-to-r from-foreground via-foreground to-primary bg-clip-text text-2xl font-semibold tracking-tight text-transparent sm:text-3xl">
          {move || {
            let i18n = I18n { locale: ctx.locale.get() };
            i18n.t("restore.title")
          }}
        </h1>
        <p class="mt-1.5 max-w-xl text-sm text-muted-foreground">
          {move || {
            let i18n = I18n { locale: ctx.locale.get() };
            i18n.t("restore.subtitle")
          }}
        </p>
      </header>

      <div class="flex flex-1 items-start justify-center pt-6 sm:pt-12">
        <div class="relative w-full max-w-lg overflow-hidden rounded-3xl border border-border/70 bg-card/80 p-8 shadow-xl shadow-black/5 backdrop-blur-md">
          <div class="pointer-events-none absolute -right-10 -top-10 size-40 rounded-full bg-primary/15 blur-3xl" />
          <div class="pointer-events-none absolute -bottom-12 -left-8 size-36 rounded-full bg-chart-2/10 blur-3xl" />

          <div class="relative flex flex-col items-center text-center">
            <div class="mb-5 flex size-16 items-center justify-center rounded-2xl bg-gradient-to-br from-primary/20 to-chart-3/20 ring-1 ring-primary/25">
              <RotateCcw class="size-7 text-primary" />
            </div>

            <p class="mb-6 max-w-sm text-sm leading-relaxed text-muted-foreground">
              {move || {
                let i18n = I18n { locale: ctx.locale.get() };
                i18n.t("restore.hint")
              }}
            </p>

            <button
              type="button"
              class="inline-flex h-11 items-center justify-center gap-2 rounded-2xl bg-primary px-6 text-sm font-semibold text-primary-foreground shadow-lg shadow-primary/25 transition-all hover:bg-primary/90 hover:shadow-primary/35 active:scale-[0.98] disabled:pointer-events-none disabled:opacity-60"
              disabled=move || loading.get()
              on:click=on_restore
            >
              {move || {
                let i18n = I18n { locale: ctx.locale.get() };
                if loading.get() {
                  view! {
                    <LoaderCircle class="size-4 animate-spin" />
                    <span>{i18n.t("restore.restoring")}</span>
                  }.into_any()
                } else {
                  view! {
                    <RotateCcw class="size-4" />
                    <span>{i18n.t("restore.action")}</span>
                  }.into_any()
                }
              }}
            </button>

            <Show when=move || success.get()>
              <div class="mt-5 inline-flex items-center gap-2 rounded-xl bg-primary/10 px-3 py-2 text-sm text-primary ring-1 ring-primary/20">
                <Check class="size-4" />
                {move || {
                  let i18n = I18n { locale: ctx.locale.get() };
                  i18n.t("restore.success")
                }}
              </div>
            </Show>
          </div>
        </div>
      </div>
    </div>
  }
}
