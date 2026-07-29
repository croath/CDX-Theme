use icons::{LoaderCircle, MessageSquare, Play, RefreshCw, Trash2, WandSparkles};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, CodexSessionSummary};
use crate::components::ui::confirm_dialog::ConfirmDialog;
use crate::components::ui::sonner::{toast_error, toast_success};
use crate::i18n::I18n;
use crate::state::AppCtx;

use super::{format_session_meta, short_id};

#[component]
pub(super) fn BuilderHome(
  sessions: RwSignal<Vec<CodexSessionSummary>>,
  loading: RwSignal<bool>,
  error: RwSignal<Option<String>>,
  sessions_list_gen: RwSignal<u64>,
  on_refresh: Callback<()>,
  on_start: Callback<()>,
  on_open: Callback<(String, String, Option<String>)>,
) -> impl IntoView {
  let ctx = AppCtx::use_ctx();
  let confirm_delete_open = RwSignal::new(false);
  let pending_delete_id = RwSignal::new(Option::<String>::None);
  let pending_delete_title = RwSignal::new(String::new());
  let deleting = RwSignal::new(false);

  let open_delete_confirm = move |id: String, title: String| {
    if deleting.get_untracked() {
      return;
    }
    pending_delete_id.set(Some(id));
    pending_delete_title.set(title);
    confirm_delete_open.set(true);
  };

  let perform_delete = move |_: ()| {
    if deleting.get_untracked() {
      return;
    }
    let Some(id) = pending_delete_id.get_untracked() else {
      confirm_delete_open.set(false);
      return;
    };
    deleting.set(true);
    let locale = ctx.locale.get_untracked();
    let title = pending_delete_title.get_untracked();
    spawn_local(async move {
      let i18n = I18n { locale };
      match api::delete_theme_builder_session(id.clone()).await {
        Ok(_) => {
          // Invalidate any in-flight list fetch so it cannot reinsert this session.
          sessions_list_gen.update(|g| *g = g.wrapping_add(1));
          let next: Vec<_> = sessions
            .get_untracked()
            .into_iter()
            .filter(|s| s.id != id)
            .collect();
          sessions.set(next);
          deleting.set(false);
          confirm_delete_open.set(false);
          pending_delete_id.set(None);
          let label = if title.trim().is_empty() {
            short_id(&id)
          } else {
            title
          };
          toast_success(i18n.t("builder.sessions.delete.success"), &label);
        }
        Err(e) => {
          deleting.set(false);
          toast_error(i18n.t("builder.error"), &e);
        }
      }
    });
  };

  let dialog_title = Signal::derive(move || {
    let i18n = I18n {
      locale: ctx.locale.get(),
    };
    i18n.t("builder.sessions.delete").to_string()
  });
  let dialog_body = Signal::derive(move || {
    let i18n = I18n {
      locale: ctx.locale.get(),
    };
    let name = pending_delete_title.get();
    if name.trim().is_empty() {
      i18n.t("builder.sessions.delete.confirm").to_string()
    } else {
      format!("{} — {}", name, i18n.t("builder.sessions.delete.confirm"))
    }
  });
  let dialog_ok = Signal::derive(move || {
    let i18n = I18n {
      locale: ctx.locale.get(),
    };
    i18n.t("builder.sessions.delete").to_string()
  });
  let dialog_cancel = Signal::derive(move || {
    let i18n = I18n {
      locale: ctx.locale.get(),
    };
    i18n.t("recommend.delete.confirm.cancel").to_string()
  });

  view! {
    <header class="mb-4 shrink-0 sm:mb-5">
      <div class="flex items-start gap-3">
        <div class="mt-0.5 flex size-10 shrink-0 items-center justify-center rounded-2xl bg-gradient-to-br from-primary/25 to-chart-2/20 text-primary ring-1 ring-primary/25">
          <WandSparkles class="size-5" />
        </div>
        <div class="min-w-0 flex-1">
          <h1 class="bg-gradient-to-r from-foreground via-foreground to-primary bg-clip-text text-2xl font-semibold tracking-tight text-transparent sm:text-3xl">
            {move || {
              let i18n = I18n { locale: ctx.locale.get() };
              i18n.t("builder.title")
            }}
          </h1>
          <p class="mt-1 max-w-2xl text-sm text-muted-foreground">
            {move || {
              let i18n = I18n { locale: ctx.locale.get() };
              i18n.t("builder.subtitle")
            }}
          </p>
        </div>
      </div>
    </header>

    <div class="mb-5 shrink-0 overflow-hidden rounded-3xl border border-border/70 bg-card/80 p-5 shadow-sm backdrop-blur-md sm:p-6">
      <div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div class="min-w-0">
          <h2 class="text-sm font-semibold text-foreground">
            {move || {
              let i18n = I18n { locale: ctx.locale.get() };
              i18n.t("builder.start.title")
            }}
          </h2>
          <p class="mt-1 max-w-lg text-xs leading-relaxed text-muted-foreground">
            {move || {
              let i18n = I18n { locale: ctx.locale.get() };
              i18n.t("builder.start.hint")
            }}
          </p>
        </div>
        <button
          type="button"
          class="inline-flex h-11 shrink-0 items-center justify-center gap-2 rounded-2xl bg-primary px-5 text-sm font-semibold text-primary-foreground shadow-lg shadow-primary/25 transition-all hover:bg-primary/90 active:scale-[0.98]"
          on:click=move |_| on_start.run(())
        >
          <Play class="size-4" />
          {move || {
            let i18n = I18n { locale: ctx.locale.get() };
            i18n.t("builder.start.action")
          }}
        </button>
      </div>
    </div>

    <div class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-3xl border border-border/70 bg-card/70 shadow-sm backdrop-blur-md">
      <div class="flex items-center justify-between gap-3 border-b border-border/50 px-4 py-3 sm:px-5">
        <div class="flex items-center gap-2">
          <MessageSquare class="size-4 text-primary" />
          <h2 class="text-sm font-semibold text-foreground">
            {move || {
              let i18n = I18n { locale: ctx.locale.get() };
              i18n.t("builder.sessions.title")
            }}
          </h2>
        </div>
        <button
          type="button"
          class="inline-flex size-8 items-center justify-center rounded-lg text-muted-foreground transition-colors hover:bg-accent hover:text-foreground disabled:opacity-50"
          disabled=move || loading.get()
          on:click=move |_| on_refresh.run(())
          aria-label=move || {
            let i18n = I18n { locale: ctx.locale.get() };
            i18n.t("builder.sessions.refresh")
          }
        >
          <span class=move || {
            if loading.get() {
              "inline-flex animate-spin"
            } else {
              "inline-flex"
            }
          }>
            <RefreshCw class="size-4" />
          </span>
        </button>
      </div>

      <div class="min-h-0 flex-1 overflow-y-auto px-3 py-3 sm:px-4">
        <Show when=move || loading.get() && sessions.get().is_empty()>
          <div class="flex items-center justify-center gap-2 py-12 text-sm text-muted-foreground">
            <LoaderCircle class="size-4 animate-spin" />
            {move || {
              let i18n = I18n { locale: ctx.locale.get() };
              i18n.t("builder.sessions.loading")
            }}
          </div>
        </Show>

        <Show when=move || !loading.get() && sessions.get().is_empty()>
          <div class="px-2 py-10 text-center">
            <p class="text-sm text-muted-foreground">
              {move || {
                let i18n = I18n { locale: ctx.locale.get() };
                if error.get().is_some() {
                  i18n.t("builder.sessions.error")
                } else {
                  i18n.t("builder.sessions.empty")
                }
              }}
            </p>
          </div>
        </Show>

        <ul class="space-y-2">
          <For
            each=move || sessions.get()
            key=|s| s.id.clone()
            children=move |s| {
              let id = s.id.clone();
              let title = if s.title.trim().is_empty() {
                s.id.clone()
              } else {
                s.title.clone()
              };
              let updated = s.updated_at.clone();
              let id_open = id.clone();
              let title_open = title.clone();
              let wp_open = s.workspace_path.clone();
              let id_delete = id.clone();
              let title_delete = title.clone();
              view! {
                <li class="group flex items-stretch gap-1.5">
                  <button
                    type="button"
                    class="flex min-w-0 flex-1 items-center gap-3 rounded-2xl border border-border/50 bg-background/50 px-3.5 py-3 text-left transition-all hover:border-primary/30 hover:bg-primary/5"
                    on:click=move |_| {
                      on_open.run((id_open.clone(), title_open.clone(), wp_open.clone()))
                    }
                  >
                    <span class="flex size-9 shrink-0 items-center justify-center rounded-xl bg-muted/80 text-muted-foreground group-hover:bg-primary/15 group-hover:text-primary">
                      <MessageSquare class="size-4" />
                    </span>
                    <span class="min-w-0 flex-1">
                      <span class="block truncate text-sm font-medium text-foreground">{title}</span>
                      <span class="mt-0.5 block truncate font-mono text-[11px] text-muted-foreground">
                        {format_session_meta(&id, &updated)}
                      </span>
                    </span>
                    <span class="shrink-0 rounded-full bg-primary/10 px-2.5 py-1 text-[11px] font-medium text-primary ring-1 ring-primary/15">
                      {move || {
                        let i18n = I18n { locale: ctx.locale.get() };
                        i18n.t("builder.sessions.open")
                      }}
                    </span>
                  </button>
                  <button
                    type="button"
                    class="inline-flex size-11 shrink-0 items-center justify-center self-center rounded-xl border border-border/50 bg-background/50 text-muted-foreground transition-colors hover:border-destructive/40 hover:bg-destructive/10 hover:text-destructive disabled:pointer-events-none disabled:opacity-50"
                    prop:disabled=move || deleting.get()
                    aria-label=move || {
                      let i18n = I18n { locale: ctx.locale.get() };
                      i18n.t("builder.sessions.delete")
                    }
                    title=move || {
                      let i18n = I18n { locale: ctx.locale.get() };
                      i18n.t("builder.sessions.delete")
                    }
                    on:click=move |ev| {
                      ev.stop_propagation();
                      open_delete_confirm(id_delete.clone(), title_delete.clone());
                    }
                  >
                    <Trash2 class="size-4" />
                  </button>
                </li>
              }
            }
          />
        </ul>
      </div>
    </div>

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
  }
}
