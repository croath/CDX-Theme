use crate::api;
use crate::components::theme_card::ThemeCard;
use crate::components::ui::sonner::{toast_error, toast_success};
use crate::i18n::I18n;
use crate::state::AppCtx;
use cdx_theme_types::ThemeMetadata;
use icons::{LoaderCircle, RefreshCw};
use leptos::prelude::*;

#[component]
pub fn RecommendPage() -> impl IntoView {
  let ctx = AppCtx::use_ctx();

  // Bump to re-run the resource; tick > 0 forces a cache-busting network fetch.
  let refresh_tick = RwSignal::new(0u32);
  let refreshing = RwSignal::new(false);

  let themes = LocalResource::new(move || {
    let tick = refresh_tick.get();
    async move {
      let force = tick > 0;
      if force {
        refreshing.set(true);
      }
      let result = api::fetch_remote_theme_catalog(force).await;
      refreshing.set(false);
      result
    }
  });
  let load_error_toasted = RwSignal::new(false);
  let update_notice_toasted = RwSignal::new(false);

  // Surface load failures once via sonner; notify when local themes have updates.
  Effect::new(move |_| match themes.get() {
    Some(Err(e)) if !load_error_toasted.get_untracked() => {
      load_error_toasted.set(true);
      let i18n = I18n {
        locale: ctx.locale.get_untracked(),
      };
      toast_error(i18n.t("recommend.error"), &e);
    }
    Some(Ok(list)) => {
      load_error_toasted.set(false);
      if !update_notice_toasted.get_untracked() {
        let updates: Vec<_> = list
          .iter()
          .filter(|t| t.update_available)
          .map(|t| t.name.as_str())
          .collect();
        if !updates.is_empty() {
          update_notice_toasted.set(true);
          let i18n = I18n {
            locale: ctx.locale.get_untracked(),
          };
          let detail = if updates.len() == 1 {
            updates[0].to_string()
          } else {
            format!("{} (+{})", updates[0], updates.len() - 1)
          };
          toast_success(i18n.t("recommend.update.notify"), &detail);
        }
      }
    }
    _ => {}
  });

  let on_refresh = move |_| {
    if refreshing.get_untracked() {
      return;
    }
    load_error_toasted.set(false);
    update_notice_toasted.set(false);
    refresh_tick.update(|n| *n = n.saturating_add(1));
  };

  view! {
    <div class="flex h-full flex-col">
      <header class="mb-6 flex flex-wrap items-start justify-between gap-3">
        <div class="min-w-0 flex-1">
          <h1 class="bg-gradient-to-r from-foreground via-foreground to-primary bg-clip-text text-2xl font-semibold tracking-tight text-transparent sm:text-3xl">
            {move || {
              let i18n = I18n { locale: ctx.locale.get() };
              i18n.t("recommend.title")
            }}
          </h1>
          <p class="mt-1.5 max-w-xl text-sm text-muted-foreground">
            {move || {
              let i18n = I18n { locale: ctx.locale.get() };
              i18n.t("recommend.subtitle")
            }}
          </p>
        </div>
        <button
          type="button"
          class="inline-flex h-9 shrink-0 items-center justify-center gap-1.5 rounded-xl border border-border bg-background/80 px-3 text-sm font-medium text-foreground shadow-sm transition-all hover:bg-accent active:scale-[0.97] disabled:pointer-events-none disabled:opacity-60"
          disabled=move || refreshing.get() || themes.get().is_none()
          on:click=on_refresh
          title=move || {
            let i18n = I18n { locale: ctx.locale.get() };
            i18n.t("recommend.refresh")
          }
        >
          {move || {
            let i18n = I18n { locale: ctx.locale.get() };
            let spinning = refreshing.get();
            view! {
              <RefreshCw class=if spinning {
                "size-4 animate-spin"
              } else {
                "size-4"
              } />
              <span>
                {if spinning {
                  i18n.t("recommend.refreshing")
                } else {
                  i18n.t("recommend.refresh")
                }}
              </span>
            }
          }}
        </button>
      </header>

      // Full-width scroll so the scrollbar sits on the main panel's right edge
      // (cancels parent px-5/7/8 on the right, then restores content inset with matching pr).
      <div class="content-scroll -mr-5 min-h-0 flex-1 overflow-y-auto pr-5 sm:-mr-7 sm:pr-7 lg:-mr-8 lg:pr-8">
        <Suspense fallback=move || {
          view! {
            <div class="flex h-48 flex-col items-center justify-center gap-3 text-muted-foreground">
              <LoaderCircle class="size-8 animate-spin text-primary" />
              <span class="text-sm">
                {move || {
                  let i18n = I18n { locale: ctx.locale.get() };
                  i18n.t("recommend.loading")
                }}
              </span>
            </div>
          }
        }>
          {move || {
            match themes.get() {
              Some(Ok(list)) if list.is_empty() => {
                view! {
                  <EmptyState />
                }.into_any()
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
                  <div class="flex h-48 flex-col items-center justify-center gap-3 rounded-2xl border border-dashed border-border bg-muted/30 text-muted-foreground">
                    <span class="text-sm">
                      {move || {
                        let i18n = I18n { locale: ctx.locale.get() };
                        i18n.t("recommend.error")
                      }}
                    </span>
                    <button
                      type="button"
                      class="inline-flex h-9 items-center gap-1.5 rounded-xl bg-primary px-3.5 text-sm font-medium text-primary-foreground shadow-sm hover:bg-primary/90"
                      on:click=on_refresh
                    >
                      <RefreshCw class="size-4" />
                      {move || {
                        let i18n = I18n { locale: ctx.locale.get() };
                        i18n.t("recommend.refresh")
                      }}
                    </button>
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
                  allow_delete=false
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
          i18n.t("recommend.empty")
        }}
      </span>
    </div>
  }
}
