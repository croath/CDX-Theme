//! Library page — installed / downloaded themes under local_data/themes (+ builtin).

use crate::api;
use crate::components::theme_card::ThemeCard;
use crate::components::ui::sonner::toast_error;
use crate::i18n::I18n;
use crate::state::AppCtx;
use cdx_theme_types::ThemeMetadata;
use icons::LoaderCircle;
use leptos::prelude::*;

#[component]
pub fn LibraryPage() -> impl IntoView {
  let ctx = AppCtx::use_ctx();

  let themes = LocalResource::new(|| async move { api::retrieve_local_theme_list().await });
  let load_error_toasted = RwSignal::new(false);

  Effect::new(move |_| match themes.get() {
    Some(Err(e)) if !load_error_toasted.get_untracked() => {
      load_error_toasted.set(true);
      let i18n = I18n {
        locale: ctx.locale.get_untracked(),
      };
      toast_error(i18n.t("library.error"), &e);
    }
    Some(Ok(_)) => load_error_toasted.set(false),
    _ => {}
  });

  view! {
    <div class="flex h-full flex-col">
      <header class="mb-6">
        <h1 class="bg-gradient-to-r from-foreground via-foreground to-primary bg-clip-text text-2xl font-semibold tracking-tight text-transparent sm:text-3xl">
          {move || {
            let i18n = I18n { locale: ctx.locale.get() };
            i18n.t("library.title")
          }}
        </h1>
        <p class="mt-1.5 max-w-xl text-sm text-muted-foreground">
          {move || {
            let i18n = I18n { locale: ctx.locale.get() };
            i18n.t("library.subtitle")
          }}
        </p>
      </header>

      <div class="min-h-0 flex-1 overflow-y-auto pr-1">
        <Suspense fallback=move || {
          view! {
            <div class="flex h-48 flex-col items-center justify-center gap-3 text-muted-foreground">
              <LoaderCircle class="size-8 animate-spin text-primary" />
              <span class="text-sm">
                {move || {
                  let i18n = I18n { locale: ctx.locale.get() };
                  i18n.t("library.loading")
                }}
              </span>
            </div>
          }
        }>
          {move || {
            match themes.get() {
              Some(Ok(list)) if list.is_empty() => {
                view! { <EmptyState /> }.into_any()
              }
              Some(Ok(list)) => {
                let applied = list
                  .iter()
                  .find(|t| t.is_applied)
                  .map(|t| t.id.clone());
                view! {
                  <ThemeGrid themes=list applied_theme_id=applied />
                }.into_any()
              }
              Some(Err(_)) => {
                view! {
                  <div class="flex h-48 flex-col items-center justify-center gap-2 rounded-2xl border border-dashed border-border bg-muted/30 text-muted-foreground">
                    <span class="text-sm">
                      {move || {
                        let i18n = I18n { locale: ctx.locale.get() };
                        i18n.t("library.error")
                      }}
                    </span>
                  </div>
                }.into_any()
              }
              None => view! { <div /> }.into_any(),
            }
          }}
        </Suspense>
      </div>
    </div>
  }
}

#[component]
fn ThemeGrid(themes: Vec<ThemeMetadata>, applied_theme_id: Option<String>) -> impl IntoView {
  let themes = RwSignal::new(themes);
  let applied_theme_id = RwSignal::new(applied_theme_id);
  view! {
    <Show
      when=move || !themes.get().is_empty()
      fallback=move || view! { <EmptyState /> }
    >
      <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-3">
        {move || {
          themes
            .get()
            .into_iter()
            .map(|theme| {
              view! {
                <ThemeCard
                  theme=theme
                  applied_theme_id=applied_theme_id
                  themes=themes
                />
              }
            })
            .collect_view()
        }}
      </div>
    </Show>
  }
}

#[component]
fn EmptyState() -> impl IntoView {
  let ctx = AppCtx::use_ctx();
  view! {
    <div class="flex h-48 flex-col items-center justify-center gap-2 rounded-2xl border border-dashed border-border bg-muted/30 text-muted-foreground">
      <span class="text-sm">
        {move || {
          let i18n = I18n { locale: ctx.locale.get() };
          i18n.t("library.empty")
        }}
      </span>
    </div>
  }
}
