use crate::api;
use crate::components::ui::confirm_dialog::ConfirmDialog;
use crate::components::ui::sonner::{toast_error, toast_success};
use crate::i18n::I18n;
use crate::state::AppCtx;
use cdx_theme_types::{ThemeMetadata, ThemeSource};
use icons::{Check, Download, LoaderCircle, PackagePlus, Sparkles, Trash2};
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn ThemeCard(
  theme: ThemeMetadata,
  /// Shared signal of currently applied theme id (from parent list).
  applied_theme_id: RwSignal<Option<String>>,
  /// Shared list so delete can remove the card without a full reload.
  themes: RwSignal<Vec<ThemeMetadata>>,
  /// When false (e.g. Recommend page), hide delete even for installed entries.
  #[prop(default = true)]
  allow_delete: bool,
) -> impl IntoView {
  let ctx = AppCtx::use_ctx();
  let applying = RwSignal::new(false);
  let downloading = RwSignal::new(false);
  let deleting = RwSignal::new(false);
  let confirm_delete_open = RwSignal::new(false);

  // Own copies for each reactive closure so nothing is moved twice.
  let this_id = StoredValue::new(theme.id.clone());
  let theme_name = StoredValue::new(theme.name.clone());
  let theme_url = StoredValue::new(theme.theme_url.clone());
  let display_name = theme.name.clone();
  let source = theme.source;
  let update_available = theme.update_available;
  let local_version = theme.version;
  let remote_version = theme.remote_version;
  let can_delete = allow_delete && source == ThemeSource::Installed;

  // Preview: prefer already-local data URLs; resolve remote HTTP(S) through disk cache.
  let preview_src = RwSignal::new(Option::<String>::None);
  {
    let initial = theme.preview_img.clone();
    if let Some(src) = initial.clone().filter(|s| !s.trim().is_empty()) {
      let is_remote = src.starts_with("https://") || src.starts_with("http://");
      if is_remote {
        spawn_local(async move {
          match api::resolve_cached_image(src).await {
            Ok(local) if !local.is_empty() => preview_src.set(Some(local)),
            Ok(_) | Err(_) => {
              // Keep gradient fallback when cache/network fails.
            }
          }
        });
      } else {
        preview_src.set(Some(src));
      }
    }
  }
  // Download for not-yet-installed remotes, or when a newer remote version is available.
  let can_download = theme
    .theme_url
    .as_ref()
    .is_some_and(|u| !u.trim().is_empty())
    && (source == ThemeSource::Remote || update_available);
  let busy = move || applying.get() || downloading.get() || deleting.get();

  let colors: Vec<String> = if theme.preview_colors.is_empty() {
    vec!["#84CC16".into(), "#F7FEE7".into(), "#1A2E05".into()]
  } else {
    theme.preview_colors.clone()
  };

  let accent = colors.first().cloned().unwrap_or_else(|| "#84CC16".into());
  let surface = colors.get(1).cloned().unwrap_or_else(|| "#F7FEE7".into());
  let ink = colors.get(2).cloned().unwrap_or_else(|| "#1A2E05".into());
  let gradient =
    format!("background: linear-gradient(135deg, {accent} 0%, {surface} 55%, {ink} 100%);");
  let color_swatches = colors.clone();

  let is_this_applied = move || {
    applied_theme_id
      .get()
      .as_ref()
      .is_some_and(|id| id == &this_id.get_value())
  };

  let on_apply = move |_| {
    if applying.get_untracked() || downloading.get_untracked() || deleting.get_untracked() {
      return;
    }
    applying.set(true);
    let name = theme_name.get_value();
    let id = this_id.get_value();
    let url = theme_url.get_value();
    let locale = ctx.locale.get_untracked();
    spawn_local(async move {
      match api::apply_theme(id.clone(), url).await {
        Ok(_) => {
          applied_theme_id.set(Some(id.clone()));
          // Apply downloads remote packages — mark card as installed / up to date.
          themes.update(|list| {
            if let Some(t) = list.iter_mut().find(|t| t.id == id) {
              t.source = ThemeSource::Installed;
              t.is_applied = true;
              if let Some(rv) = t.remote_version {
                t.version = Some(rv);
              }
              t.update_available = false;
            }
            for t in list.iter_mut() {
              if t.id != id {
                t.is_applied = false;
              }
            }
          });
          applying.set(false);
          let i18n = I18n { locale };
          toast_success(i18n.t("recommend.apply.success"), &name);
        }
        Err(e) => {
          applying.set(false);
          let i18n = I18n { locale };
          toast_error(&format!("{} — {}", i18n.t("recommend.apply"), name), &e);
        }
      }
    });
  };

  let on_download = move |_| {
    if !can_download
      || applying.get_untracked()
      || downloading.get_untracked()
      || deleting.get_untracked()
    {
      return;
    }
    let Some(url) = theme_url.get_value().filter(|u| !u.trim().is_empty()) else {
      return;
    };
    downloading.set(true);
    let name = theme_name.get_value();
    let id = this_id.get_value();
    let locale = ctx.locale.get_untracked();
    spawn_local(async move {
      let i18n = I18n { locale };
      match api::download_theme(url).await {
        Ok(meta) => {
          themes.update(|list| {
            if let Some(t) = list.iter_mut().find(|t| t.id == id || t.id == meta.id) {
              t.source = ThemeSource::Installed;
              t.location = meta.location.clone();
              // Prefer version from the downloaded package; fall back to remote catalog.
              t.version = meta.version.or(t.remote_version).or(t.version);
              t.update_available = false;
              if t.preview_img.is_none() {
                t.preview_img = meta.preview_img.clone();
              }
              if t.preview_colors.is_empty() {
                t.preview_colors = meta.preview_colors.clone();
              }
              t.is_applied = applied_theme_id
                .get_untracked()
                .as_ref()
                .is_some_and(|a| a == &t.id);
            }
          });
          downloading.set(false);
          toast_success(i18n.t("recommend.download.success"), &name);
        }
        Err(e) => {
          downloading.set(false);
          toast_error(
            &format!("{} — {}", i18n.t("recommend.download.error"), name),
            &e,
          );
        }
      }
    });
  };

  let open_delete_confirm = move |_| {
    if !can_delete || busy() {
      return;
    }
    confirm_delete_open.set(true);
  };

  let perform_delete = move |_: ()| {
    if !can_delete || busy() {
      return;
    }
    deleting.set(true);
    let name = theme_name.get_value();
    let id = this_id.get_value();
    let locale = ctx.locale.get_untracked();
    spawn_local(async move {
      let i18n = I18n { locale };
      match api::delete_theme(id.clone()).await {
        Ok(_) => {
          themes.update(|list| list.retain(|t| t.id != id));
          if applied_theme_id.get_untracked().as_deref() == Some(id.as_str()) {
            applied_theme_id.set(None);
          }
          deleting.set(false);
          confirm_delete_open.set(false);
          toast_success(i18n.t("recommend.delete.success"), &name);
        }
        Err(e) => {
          deleting.set(false);
          toast_error(
            &format!("{} — {}", i18n.t("recommend.delete.error"), name),
            &e,
          );
        }
      }
    });
  };

  let dialog_title = Signal::derive(move || {
    let i18n = I18n {
      locale: ctx.locale.get(),
    };
    i18n.t("recommend.delete.confirm.title").to_string()
  });
  let dialog_body = Signal::derive(move || {
    let i18n = I18n {
      locale: ctx.locale.get(),
    };
    let name = theme_name.get_value();
    format!("{} — {}", name, i18n.t("recommend.delete.confirm.body"))
  });
  let dialog_ok = Signal::derive(move || {
    let i18n = I18n {
      locale: ctx.locale.get(),
    };
    if deleting.get() {
      i18n.t("recommend.deleting").to_string()
    } else {
      i18n.t("recommend.delete.confirm.ok").to_string()
    }
  });
  let dialog_cancel = Signal::derive(move || {
    let i18n = I18n {
      locale: ctx.locale.get(),
    };
    i18n.t("recommend.delete.confirm.cancel").to_string()
  });

  view! {
    <article class=move || {
      if is_this_applied() {
        "theme-card group relative flex flex-col overflow-hidden rounded-2xl border border-primary/50 bg-card/80 shadow-md shadow-primary/10 backdrop-blur-sm transition-all duration-300 ring-1 ring-primary/25"
      } else {
        "theme-card group relative flex flex-col overflow-hidden rounded-2xl border border-border/70 bg-card/80 shadow-sm backdrop-blur-sm transition-all duration-300 hover:-translate-y-1 hover:border-primary/40 hover:shadow-xl hover:shadow-primary/10"
      }
    }>
      <div class="relative h-36 w-full overflow-hidden" style=gradient>
        {move || {
          match preview_src.get() {
            Some(src) if !src.is_empty() => view! {
              <img
                src=src
                alt=""
                class="absolute inset-0 size-full object-cover"
                // Hint browser not to revalidate; bytes are already local data URLs.
                loading="lazy"
                decoding="async"
              />
              <div class="absolute inset-0 bg-gradient-to-t from-black/45 via-black/10 to-transparent" />
            }.into_any(),
            _ => view! {
              <div class="absolute inset-0 opacity-30 mix-blend-overlay bg-[radial-gradient(circle_at_20%_20%,white,transparent_45%),radial-gradient(circle_at_80%_70%,white,transparent_40%)]" />
              <div class="absolute bottom-3 left-3 flex gap-1.5">
                {color_swatches
                  .clone()
                  .into_iter()
                  .map(|c| {
                    view! {
                      <span
                        class="size-4 rounded-full border border-white/50 shadow-sm ring-1 ring-black/10"
                        style=format!("background-color: {c}")
                      />
                    }
                  })
                  .collect_view()}
              </div>
            }.into_any(),
          }
        }}

        <div class="absolute right-3 top-3 flex flex-wrap items-center justify-end gap-1.5">
          {if update_available {
            view! {
              <span class="inline-flex items-center gap-1 rounded-full bg-amber-500 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wider text-white shadow-sm">
                {move || {
                  let i18n = I18n { locale: ctx.locale.get() };
                  i18n.t("recommend.tag.update")
                }}
              </span>
            }.into_any()
          } else {
            view! { <span class="hidden" /> }.into_any()
          }}
          <Show when=is_this_applied>
            <span class="inline-flex items-center gap-1 rounded-full bg-primary px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wider text-primary-foreground shadow-sm">
              <Check class="size-3" />
              {move || {
                let i18n = I18n { locale: ctx.locale.get() };
                i18n.t("recommend.applied")
              }}
            </span>
          </Show>
          <span class=move || {
            match source {
              ThemeSource::Installed => {
                "rounded-full bg-chart-2/90 px-2 py-0.5 text-[10px] font-medium uppercase tracking-wider text-white shadow-sm backdrop-blur-md"
              }
              ThemeSource::Remote => {
                "rounded-full bg-chart-4/90 px-2 py-0.5 text-[10px] font-medium uppercase tracking-wider text-white shadow-sm backdrop-blur-md"
              }
              ThemeSource::Builtin => {
                "rounded-full bg-black/30 px-2 py-0.5 text-[10px] font-medium uppercase tracking-wider text-white/90 backdrop-blur-md"
              }
            }
          }>
            <span class="inline-flex items-center gap-1">
              {match source {
                ThemeSource::Installed => view! { <PackagePlus class="size-3" /> }.into_any(),
                ThemeSource::Remote | ThemeSource::Builtin => {
                  view! { <Sparkles class="size-3" /> }.into_any()
                }
              }}
              {move || {
                let i18n = I18n { locale: ctx.locale.get() };
                match source {
                  ThemeSource::Installed => i18n.t("recommend.tag.install"),
                  ThemeSource::Remote => i18n.t("recommend.tag.remote"),
                  ThemeSource::Builtin => i18n.t("recommend.tag.builtin"),
                }
              }}
            </span>
          </span>
        </div>
      </div>

      <div class="flex flex-1 flex-col gap-3 p-4">
        <div class="min-w-0">
          <h3 class="truncate text-base font-semibold tracking-tight text-foreground">
            {display_name}
          </h3>
          {if update_available {
            let local = local_version.map(|v| v.to_string()).unwrap_or_else(|| "—".into());
            let remote = remote_version.map(|v| v.to_string()).unwrap_or_else(|| "—".into());
            view! {
              <p class="mt-1 text-xs text-amber-600 dark:text-amber-400">
                {move || {
                  let i18n = I18n { locale: ctx.locale.get() };
                  format!(
                    "{} (v{local} → v{remote})",
                    i18n.t("recommend.update.hint")
                  )
                }}
              </p>
            }.into_any()
          } else if let Some(v) = remote_version.or(local_version) {
            view! {
              <p class="mt-1 text-xs text-muted-foreground">
                {format!("v{v}")}
              </p>
            }.into_any()
          } else {
            view! { <span class="hidden" /> }.into_any()
          }}
        </div>

        <div class="mt-auto flex flex-wrap items-center justify-end gap-2 pt-1">
          {if can_delete {
            view! {
              <button
                type="button"
                class="inline-flex h-9 items-center justify-center gap-1.5 rounded-xl border border-destructive/30 bg-destructive/10 px-3 text-sm font-medium text-destructive transition-all hover:bg-destructive/15 active:scale-[0.97] disabled:pointer-events-none disabled:opacity-60"
                disabled=busy
                on:click=open_delete_confirm
                title=move || {
                  let i18n = I18n { locale: ctx.locale.get() };
                  i18n.t("recommend.delete")
                }
              >
                <Trash2 class="size-4" />
                <span class="hidden sm:inline">
                  {move || {
                    let i18n = I18n { locale: ctx.locale.get() };
                    i18n.t("recommend.delete")
                  }}
                </span>
              </button>
            }.into_any()
          } else {
            view! { <span class="hidden" /> }.into_any()
          }}

          {if can_download {
            view! {
              <button
                type="button"
                class="inline-flex h-9 items-center justify-center gap-1.5 rounded-xl border border-border bg-background/80 px-3 text-sm font-medium text-foreground shadow-sm transition-all hover:bg-accent active:scale-[0.97] disabled:pointer-events-none disabled:opacity-60"
                disabled=busy
                on:click=on_download
                title=move || {
                  let i18n = I18n { locale: ctx.locale.get() };
                  i18n.t("recommend.download")
                }
              >
                {move || {
                  let i18n = I18n { locale: ctx.locale.get() };
                  if downloading.get() {
                    view! {
                      <LoaderCircle class="size-4 animate-spin" />
                      <span>{i18n.t("recommend.downloading")}</span>
                    }.into_any()
                  } else if update_available {
                    view! {
                      <Download class="size-4" />
                      <span>{i18n.t("recommend.update")}</span>
                    }.into_any()
                  } else {
                    view! {
                      <Download class="size-4" />
                      <span>{i18n.t("recommend.download")}</span>
                    }.into_any()
                  }
                }}
              </button>
            }.into_any()
          } else {
            view! { <span class="hidden" /> }.into_any()
          }}

          <button
            type="button"
            class="inline-flex h-9 items-center justify-center gap-1.5 rounded-xl bg-primary px-3.5 text-sm font-medium text-primary-foreground shadow-sm transition-all hover:bg-primary/90 active:scale-[0.97] disabled:pointer-events-none disabled:opacity-60"
            disabled=busy
            on:click=on_apply
          >
            {move || {
              let i18n = I18n { locale: ctx.locale.get() };
              if applying.get() {
                view! {
                  <LoaderCircle class="size-4 animate-spin" />
                  <span>{i18n.t("recommend.applying")}</span>
                }.into_any()
              } else {
                // Always re-applyable, even when this theme is already marked applied.
                view! {
                  <span>{i18n.t("recommend.apply")}</span>
                }.into_any()
              }
            }}
          </button>
        </div>
      </div>

      {if can_delete {
        view! {
          <ConfirmDialog
            open=confirm_delete_open
            title=dialog_title
            description=dialog_body
            confirm_label=dialog_ok
            cancel_label=dialog_cancel
            confirming=Signal::derive(move || deleting.get())
            destructive=true
            on_confirm=Callback::new(perform_delete)
          />
        }.into_any()
      } else {
        view! { <span class="hidden" /> }.into_any()
      }}
    </article>
  }
}
