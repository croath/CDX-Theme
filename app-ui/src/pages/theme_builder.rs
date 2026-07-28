//! Theme Builder — generate a theme with Codex (ACP), then apply into the library.

use icons::{
  ArrowLeft, Check, ImagePlus, LoaderCircle, MessageSquare, Play, RefreshCw, Trash2, WandSparkles,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::JsFuture;
use web_sys::{DragEvent, Event, File, HtmlInputElement};

use crate::api::{self, CodexSessionSummary};
use crate::components::ui::SuggestionChip;
use crate::components::ui::confirm_dialog::ConfirmDialog;
use crate::components::ui::sonner::{toast_error, toast_success};
use crate::i18n::I18n;
use crate::state::AppCtx;
use crate::types::Locale;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChatRole {
  User,
  Assistant,
  System,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ChatMessage {
  id: u64,
  role: ChatRole,
  content: String,
  pending: bool,
}

/// Home (session list) vs new generate flow vs reopened session chat.
#[derive(Clone, Debug, PartialEq, Eq)]
enum BuilderView {
  Home,
  NewBuild,
  Chat,
}

#[component]
pub fn ThemeBuilderPage() -> impl IntoView {
  let ctx = AppCtx::use_ctx();
  let view = RwSignal::new(BuilderView::Home);

  // Home state
  let sessions = RwSignal::new(Vec::<CodexSessionSummary>::new());
  let sessions_loading = RwSignal::new(false);
  let sessions_error = RwSignal::new(Option::<String>::None);
  // Bumped to invalidate in-flight session list responses (e.g. after delete).
  let sessions_list_gen = RwSignal::new(0_u64);

  // Shared workspace / session state
  let session_id = RwSignal::new(Option::<String>::None);
  let session_title = RwSignal::new(String::new());
  let workspace_id = RwSignal::new(Option::<String>::None);
  let workspace_path = RwSignal::new(Option::<String>::None);
  let workspace_ready = RwSignal::new(false);
  let workspace_error = RwSignal::new(Option::<String>::None);

  // New-build generate/apply state
  let draft = RwSignal::new(String::new());
  let generating = RwSignal::new(false);
  let applying = RwSignal::new(false);
  let build_reply = RwSignal::new(Option::<String>::None);
  let package_path = RwSignal::new(Option::<String>::None);
  let applied_name = RwSignal::new(Option::<String>::None);

  // Session chat state
  let messages = RwSignal::new(Vec::<ChatMessage>::new());
  let sending = RwSignal::new(false);
  let next_id = RwSignal::new(1_u64);
  let chat_loading = RwSignal::new(false);
  let list_ref = NodeRef::<leptos::html::Div>::new();

  let refresh_sessions = move || {
    if sessions_loading.get_untracked() {
      return;
    }
    sessions_loading.set(true);
    sessions_error.set(None);
    sessions_list_gen.update(|g| *g = g.wrapping_add(1));
    let list_req_id = sessions_list_gen.get_untracked();
    let locale = ctx.locale.get_untracked();
    spawn_local(async move {
      match api::list_codex_sessions(Some(50)).await {
        Ok(list) => {
          // Drop stale responses if a newer list/delete invalidated this request.
          if sessions_list_gen.get_untracked() == list_req_id {
            sessions.set(list);
          }
          sessions_loading.set(false);
        }
        Err(e) => {
          if sessions_list_gen.get_untracked() == list_req_id {
            sessions_error.set(Some(e.clone()));
            let i18n = I18n { locale };
            toast_error(i18n.t("builder.error"), &e);
          }
          sessions_loading.set(false);
        }
      }
    });
  };

  Effect::new(move |_| {
    if view.get() == BuilderView::Home {
      refresh_sessions();
    }
  });

  let open_new_build = move || {
    session_id.set(None);
    session_title.set(String::new());
    workspace_id.set(None);
    workspace_path.set(None);
    workspace_ready.set(false);
    workspace_error.set(None);
    draft.set(String::new());
    generating.set(false);
    applying.set(false);
    build_reply.set(None);
    package_path.set(None);
    applied_name.set(None);
    messages.set(Vec::new());
    view.set(BuilderView::NewBuild);

    // Prepare workspace in the background while the user writes a prompt.
    let locale = ctx.locale.get_untracked();
    spawn_local(async move {
      match api::start_theme_build().await {
        Ok(ws) => {
          workspace_id.set(Some(ws.workspace_id));
          workspace_path.set(Some(ws.workspace_path));
          workspace_ready.set(true);
        }
        Err(e) => {
          workspace_error.set(Some(e.clone()));
          let i18n = I18n { locale };
          toast_error(i18n.t("builder.error"), &e);
        }
      }
    });
  };

  let open_session = move |id: String, title: String| {
    session_id.set(Some(id.clone()));
    session_title.set(title);
    workspace_id.set(None);
    // workspace_path may already be set from list open
    messages.set(Vec::new());
    draft.set(String::new());
    sending.set(false);
    next_id.set(1);
    chat_loading.set(true);
    package_path.set(None);
    applied_name.set(None);
    view.set(BuilderView::Chat);
    let locale = ctx.locale.get_untracked();
    spawn_local(async move {
      match api::get_codex_session(&id).await {
        Ok(detail) => {
          if !detail.title.is_empty() {
            session_title.set(detail.title);
          }
          if let Some(wp) = detail.workspace_path.filter(|p| !p.is_empty()) {
            if workspace_id.get_untracked().is_none() {
              let id = path_basename(&wp);
              if !id.is_empty() {
                workspace_id.set(Some(id));
              }
            }
            workspace_path.set(Some(wp));
          }
          let mut msgs = Vec::new();
          let mut nid = 1_u64;
          for m in detail.messages {
            let role = match m.role.as_str() {
              "user" | "human" => ChatRole::User,
              "assistant" | "agent" => ChatRole::Assistant,
              _ => ChatRole::System,
            };
            msgs.push(ChatMessage {
              id: nid,
              role,
              content: m.content,
              pending: false,
            });
            nid += 1;
          }
          if msgs.is_empty() {
            let i18n = I18n { locale };
            msgs.push(ChatMessage {
              id: 0,
              role: ChatRole::System,
              content: i18n.t("builder.session.empty").to_string(),
              pending: false,
            });
            next_id.set(1);
          } else {
            next_id.set(nid);
          }
          messages.set(msgs);
          chat_loading.set(false);
        }
        Err(e) => {
          chat_loading.set(false);
          let i18n = I18n { locale };
          messages.set(vec![ChatMessage {
            id: 0,
            role: ChatRole::System,
            content: format!("{}: {e}", i18n.t("builder.session.load_error")),
            pending: false,
          }]);
          toast_error(i18n.t("builder.error"), &e);
        }
      }
    });
  };

  let back_home = move || {
    view.set(BuilderView::Home);
    refresh_sessions();
  };

  view! {
    <div class="flex h-full min-h-0 w-full flex-col">
      {move || match view.get() {
        BuilderView::Home => view! {
          <BuilderHome
            sessions=sessions
            loading=sessions_loading
            error=sessions_error
            sessions_list_gen=sessions_list_gen
            on_refresh=Callback::new(move |_| refresh_sessions())
            on_start=Callback::new(move |_| open_new_build())
            on_open=Callback::new(move |(id, title, wp): (String, String, Option<String>)| {
              if let Some(path) = wp.filter(|p| !p.is_empty()) {
                let basename = path_basename(&path);
                if !basename.is_empty() {
                  workspace_id.set(Some(basename));
                }
                workspace_path.set(Some(path));
              }
              open_session(id, title);
            })
          />
        }.into_any(),
        BuilderView::NewBuild => view! {
          <BuilderNewBuild
            session_id=session_id
            session_title=session_title
            workspace_id=workspace_id
            workspace_path=workspace_path
            workspace_ready=workspace_ready
            workspace_error=workspace_error
            draft=draft
            generating=generating
            applying=applying
            build_reply=build_reply
            package_path=package_path
            applied_name=applied_name
            on_back=Callback::new(move |_| back_home())
          />
        }.into_any(),
        BuilderView::Chat => view! {
          <BuilderChat
            session_id=session_id
            session_title=session_title
            workspace_id=workspace_id
            workspace_path=workspace_path
            messages=messages
            draft=draft
            sending=sending
            next_id=next_id
            chat_loading=chat_loading
            package_path=package_path
            applying=applying
            applied_name=applied_name
            list_ref=list_ref
            on_back=Callback::new(move |_| back_home())
          />
        }.into_any(),
      }}
    </div>
  }
}

#[component]
fn BuilderHome(
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

/// New theme build: hero image + description → Generate → reply + Apply.
#[component]
fn BuilderNewBuild(
  session_id: RwSignal<Option<String>>,
  session_title: RwSignal<String>,
  workspace_id: RwSignal<Option<String>>,
  workspace_path: RwSignal<Option<String>>,
  workspace_ready: RwSignal<bool>,
  workspace_error: RwSignal<Option<String>>,
  draft: RwSignal<String>,
  generating: RwSignal<bool>,
  applying: RwSignal<bool>,
  build_reply: RwSignal<Option<String>>,
  package_path: RwSignal<Option<String>>,
  applied_name: RwSignal<Option<String>>,
  on_back: Callback<()>,
) -> impl IntoView {
  let ctx = AppCtx::use_ctx();

  // Local hero upload state (reset when this page is opened).
  let hero_preview = RwSignal::new(Option::<String>::None);
  let hero_file_name = RwSignal::new(Option::<String>::None);
  let hero_data_url = RwSignal::new(Option::<String>::None);
  let hero_drag_over = RwSignal::new(false);
  let hero_input: NodeRef<leptos::html::Input> = NodeRef::new();

  let accept_hero_file = move |file: File, locale: Locale| {
    let name = file.name();
    let i18n = I18n { locale };
    if !is_allowed_hero_file(&name, file.type_().as_str()) {
      toast_error(i18n.t("builder.error"), i18n.t("builder.hero.invalid"));
      return;
    }
    if file.size() as u64 > 8 * 1024 * 1024 {
      toast_error(i18n.t("builder.error"), i18n.t("builder.hero.invalid"));
      return;
    }
    spawn_local(async move {
      match read_file_data_url(&file).await {
        Ok(data_url) => {
          hero_preview.set(Some(data_url.clone()));
          hero_data_url.set(Some(data_url));
          hero_file_name.set(Some(name));
        }
        Err(e) => {
          let i18n = I18n { locale };
          toast_error(i18n.t("builder.error"), &e);
        }
      }
    });
  };

  let on_hero_input = move |ev: Event| {
    let locale = ctx.locale.get_untracked();
    let Some(input) = ev
      .target()
      .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
    else {
      return;
    };
    if let Some(files) = input.files() {
      if let Some(file) = files.get(0) {
        accept_hero_file(file, locale);
      }
    }
    // Allow re-selecting the same file later.
    input.set_value("");
  };

  let run_generate = move |text: String, locale: Locale| {
    let text = text.trim().to_string();
    if generating.get_untracked() || applying.get_untracked() {
      return;
    }
    let i18n = I18n { locale };
    if hero_data_url.get_untracked().is_none() {
      toast_error(i18n.t("builder.error"), i18n.t("builder.hero.required"));
      return;
    }
    if text.is_empty() {
      toast_error(i18n.t("builder.error"), i18n.t("builder.desc.required"));
      return;
    }
    if let Some(err) = workspace_error.get_untracked() {
      toast_error(i18n.t("builder.error"), &err);
      return;
    }

    if session_title.get_untracked().trim().is_empty() {
      let t: String = text.chars().take(48).collect();
      session_title.set(t);
    }

    generating.set(true);
    package_path.set(None);
    applied_name.set(None);
    build_reply.set(Some(String::new()));

    let hero_payload = hero_data_url.get_untracked();
    let hero_name = hero_file_name
      .get_untracked()
      .unwrap_or_else(|| "hero.png".into());

    spawn_local(async move {
      // Live ACP stream → update reply as chunks arrive.
      let stream_handle = match api::listen_theme_builder_acp_stream(move |ev| {
        if !ev.text.is_empty() {
          build_reply.set(Some(ev.text));
        }
      })
      .await
      {
        Ok(h) => Some(h),
        Err(e) => {
          // Non-fatal: generation still works without live streaming.
          web_sys::console::warn_1(&JsValue::from_str(&format!(
            "ACP stream listen failed: {e}"
          )));
          None
        }
      };

      // Wait for workspace prep if still running.
      if !workspace_ready.get_untracked() && workspace_path.get_untracked().is_none() {
        match api::start_theme_build().await {
          Ok(ws) => {
            workspace_id.set(Some(ws.workspace_id));
            workspace_path.set(Some(ws.workspace_path));
            workspace_ready.set(true);
            workspace_error.set(None);
          }
          Err(e) => {
            if let Some(h) = stream_handle {
              h.unlisten();
            }
            generating.set(false);
            let i18n = I18n { locale };
            toast_error(i18n.t("builder.error"), &e);
            build_reply.set(Some(e));
            return;
          }
        }
      }

      let Some(ws_path) = workspace_path.get_untracked() else {
        if let Some(h) = stream_handle {
          h.unlisten();
        }
        generating.set(false);
        let i18n = I18n { locale };
        toast_error(i18n.t("builder.error"), "Workspace not ready.");
        return;
      };

      // Persist hero into theme/assets before asking Codex to build.
      let hero_asset = if let Some(data_url) = hero_payload {
        match api::save_theme_builder_hero(&ws_path, &hero_name, data_url).await {
          Ok(saved) => Some(saved),
          Err(e) => {
            if let Some(h) = stream_handle {
              h.unlisten();
            }
            generating.set(false);
            let i18n = I18n { locale };
            toast_error(i18n.t("builder.error"), &e);
            build_reply.set(Some(e));
            return;
          }
        }
      } else {
        None
      };

      let prompt = match hero_asset {
        Some(h) => format!(
          "Create a Codex theme from my hero image and description.\n\n\
           Hero image (already in the theme-dir):\n\
           - file: theme/{asset}\n\
           - set theme.json images.hero to \"{asset}\"\n\
           - use var(--cdxtheme-image-hero) for Chat/Work home hero backgrounds\n\
           - derive accent/surface/ink colors that complement the image\n\n\
           Description:\n{text}\n\n\
           Reply with a brief summary only when done (no long explanations or full CSS dumps).",
          asset = h.theme_asset_path,
          text = text
        ),
        None => format!(
          "{text}\n\nReply with a brief summary only when done (no long explanations or full CSS dumps)."
        ),
      };

      let resume = session_id.get_untracked();
      let ws_id = workspace_id.get_untracked();

      match api::codex_chat(prompt, resume, Some(ws_path), ws_id, Some(180_000)).await {
        Ok(result) => {
          if let Some(sid) = result.session_id.filter(|s| !s.is_empty()) {
            session_id.set(Some(sid));
          }
          // Prefer streamed ACP reply; fall back to status message.
          let mut body = result.reply.trim().to_string();
          if body.is_empty() {
            body = result.message.trim().to_string();
          }
          if body.is_empty() {
            body = "(No assistant text was returned from ACP.)".to_string();
          }
          let status = result.message.trim();
          if !status.is_empty()
            && status != body
            && !body.contains(status)
            && status != "Codex reply ready"
          {
            body = format!("{body}\n\n— {status}");
          }
          build_reply.set(Some(body));
          package_path.set(result.package_path.filter(|p| !p.trim().is_empty()));
        }
        Err(e) => {
          let partial = build_reply.get_untracked().unwrap_or_default();
          let body = if partial.trim().is_empty() {
            e.clone()
          } else {
            format!("{partial}\n\n— Error: {e}")
          };
          build_reply.set(Some(body));
          package_path.set(None);
          let i18n = I18n { locale };
          toast_error(i18n.t("builder.error"), &e);
        }
      }
      if let Some(h) = stream_handle {
        h.unlisten();
      }
      generating.set(false);
    });
  };

  let chip_dispatch = move |text: String, locale: Locale| {
    draft.set(text.clone());
    run_generate(text, locale);
  };

  let run_apply = move |locale: Locale| {
    if applying.get_untracked() || generating.get_untracked() {
      return;
    }
    let Some(ws) = workspace_path.get_untracked() else {
      let i18n = I18n { locale };
      toast_error(i18n.t("builder.error"), "Workspace not ready.");
      return;
    };
    let pkg = package_path.get_untracked();
    applying.set(true);
    spawn_local(async move {
      match api::apply_built_theme(ws, pkg).await {
        Ok(result) => {
          applied_name.set(Some(result.theme_name.clone()));
          let i18n = I18n { locale };
          toast_success(i18n.t("builder.apply.success"), &result.theme_name);
        }
        Err(e) => {
          let i18n = I18n { locale };
          toast_error(i18n.t("builder.error"), &e);
        }
      }
      applying.set(false);
    });
  };

  let stream_ref = NodeRef::<leptos::html::Div>::new();
  Effect::new(move |_| {
    let _ = build_reply.get();
    if let Some(el) = stream_ref.get_untracked() {
      let _ = el.set_scroll_top(el.scroll_height());
    }
  });

  view! {
    // No local page background — use the app shell mesh/background underneath.
    <div class="relative flex h-full min-h-0 w-full flex-1 flex-col overflow-hidden">
      <header class="mb-4 flex shrink-0 items-center gap-3">
        <button
          type="button"
          class="inline-flex size-10 shrink-0 items-center justify-center rounded-2xl border border-border/60 bg-card/80 text-muted-foreground shadow-sm backdrop-blur transition-colors hover:bg-accent hover:text-foreground"
          on:click=move |_| on_back.run(())
          aria-label=move || {
            let i18n = I18n { locale: ctx.locale.get() };
            i18n.t("builder.back")
          }
        >
          <ArrowLeft class="size-4" />
        </button>
        <div class="min-w-0 flex-1">
          <div class="flex items-center gap-2">
            <span class="flex size-8 items-center justify-center rounded-xl bg-gradient-to-br from-primary/30 to-chart-2/25 text-primary ring-1 ring-primary/30">
              <WandSparkles class="size-4" />
            </span>
            <h1 class="truncate bg-gradient-to-r from-foreground to-primary bg-clip-text text-xl font-semibold tracking-tight text-transparent sm:text-2xl">
              {move || {
                let t = session_title.get();
                if t.trim().is_empty() {
                  let i18n = I18n { locale: ctx.locale.get() };
                  i18n.t("builder.start.title").to_string()
                } else {
                  t
                }
              }}
            </h1>
          </div>
          <p class="mt-0.5 truncate font-mono text-[11px] text-muted-foreground">
            {move || {
              let i18n = I18n { locale: ctx.locale.get() };
              if workspace_error.get().is_some() {
                return i18n.t("builder.error").to_string();
              }
              match (workspace_id.get(), session_id.get()) {
                (Some(wid), Some(sid)) => {
                  format!("{} · session {}", short_id(&wid), short_id(&sid))
                }
                (Some(wid), None) => format!("workspace {}", short_id(&wid)),
                (None, Some(sid)) => format!("session {}", short_id(&sid)),
                _ => i18n.t("builder.generate.hint").to_string(),
              }
            }}
          </p>
        </div>
      </header>

      // Step pills
      <div class="mb-4 flex shrink-0 flex-wrap gap-2">
        <span class=move || {
          step_pill_class(hero_data_url.get().is_some())
        }>
          "1 · Hero"
        </span>
        <span class=move || {
          step_pill_class(!draft.get().trim().is_empty())
        }>
          "2 · Prompt"
        </span>
        <span class=move || {
          step_pill_class(generating.get() || build_reply.get().as_ref().is_some_and(|s| !s.is_empty()))
        }>
          "3 · Generate"
        </span>
        <span class=move || {
          step_pill_class(package_path.get().is_some() || applied_name.get().is_some())
        }>
          "4 · Apply"
        </span>
      </div>

      <div class="grid min-h-0 flex-1 gap-4 overflow-hidden lg:grid-cols-2">
        // ── Left: create form ────────────────────────────────────────
        <div class="flex min-h-0 flex-col overflow-hidden rounded-3xl border border-border/60 bg-card/75 shadow-xl shadow-black/5 backdrop-blur-xl">
          <div class="border-b border-border/40 bg-gradient-to-r from-primary/10 via-transparent to-chart-2/10 px-5 py-3">
            <p class="text-xs font-medium text-muted-foreground">
              {move || {
                let i18n = I18n { locale: ctx.locale.get() };
                i18n.t("builder.generate.hint")
              }}
            </p>
          </div>

          <div class="min-h-0 flex-1 space-y-4 overflow-y-auto p-4 sm:p-5">
            // Hero upload
            <div class="space-y-2">
              <label class="flex items-center gap-1.5 text-sm font-semibold text-foreground">
                <ImagePlus class="size-3.5 text-primary" />
                {move || {
                  let i18n = I18n { locale: ctx.locale.get() };
                  i18n.t("builder.hero.title")
                }}
                <span class="text-primary">"*"</span>
              </label>
              <div
                class=move || {
                  let base = "relative flex min-h-[150px] cursor-pointer flex-col items-center justify-center overflow-hidden rounded-2xl border-2 border-dashed px-4 py-6 transition-all";
                  if hero_drag_over.get() {
                    format!("{base} scale-[1.01] border-primary bg-primary/15 shadow-lg shadow-primary/10")
                  } else if hero_preview.get().is_some() {
                    format!("{base} border-primary/40 bg-gradient-to-br from-primary/5 to-chart-2/5")
                  } else {
                    format!("{base} border-border/70 bg-background/40 hover:border-primary/50 hover:bg-primary/5")
                  }
                }
                on:click=move |_| {
                  if generating.get_untracked() || applying.get_untracked() {
                    return;
                  }
                  if let Some(input) = hero_input.get_untracked() {
                    input.click();
                  }
                }
                on:dragover=move |ev: DragEvent| {
                  ev.prevent_default();
                  hero_drag_over.set(true);
                }
                on:dragleave=move |ev: DragEvent| {
                  ev.prevent_default();
                  hero_drag_over.set(false);
                }
                on:drop=move |ev: DragEvent| {
                  ev.prevent_default();
                  hero_drag_over.set(false);
                  if generating.get_untracked() || applying.get_untracked() {
                    return;
                  }
                  let locale = ctx.locale.get_untracked();
                  if let Some(dt) = ev.data_transfer() {
                    if let Some(files) = dt.files() {
                      if let Some(file) = files.get(0) {
                        accept_hero_file(file, locale);
                      }
                    }
                  }
                }
              >
                <Show when=move || hero_preview.get().is_none()>
                  <div class="pointer-events-none flex flex-col items-center gap-2.5 text-center">
                    <span class="flex size-14 items-center justify-center rounded-2xl bg-gradient-to-br from-primary/25 to-chart-2/20 text-primary shadow-inner ring-1 ring-primary/25">
                      <ImagePlus class="size-6" />
                    </span>
                    <p class="text-sm font-semibold text-foreground">
                      {move || {
                        let i18n = I18n { locale: ctx.locale.get() };
                        i18n.t("builder.hero.upload")
                      }}
                    </p>
                    <p class="max-w-xs text-[11px] leading-relaxed text-muted-foreground">
                      {move || {
                        let i18n = I18n { locale: ctx.locale.get() };
                        i18n.t("builder.hero.hint")
                      }}
                    </p>
                  </div>
                </Show>
                <Show when=move || hero_preview.get().is_some()>
                  <div class="flex w-full flex-col items-center gap-3 sm:flex-row">
                    <div class="relative overflow-hidden rounded-2xl shadow-lg ring-2 ring-primary/30">
                      <img
                        src=move || hero_preview.get().unwrap_or_default()
                        alt="Hero preview"
                        class="pointer-events-none h-32 w-full max-w-[240px] object-cover sm:h-28 sm:w-44"
                      />
                      <div class="pointer-events-none absolute inset-0 bg-gradient-to-t from-black/30 to-transparent" />
                    </div>
                    <div class="min-w-0 flex-1 text-center sm:text-left">
                      <p class="truncate text-sm font-semibold text-foreground">
                        {move || hero_file_name.get().unwrap_or_default()}
                      </p>
                      <p class="mt-1 text-xs font-medium text-primary">
                        {move || {
                          let i18n = I18n { locale: ctx.locale.get() };
                          i18n.t("builder.hero.change")
                        }}
                      </p>
                    </div>
                  </div>
                </Show>
                <input
                  node_ref=hero_input
                  type="file"
                  accept="image/jpeg,image/png,image/webp,image/gif,.jpg,.jpeg,.png,.webp,.gif"
                  class="hidden"
                  prop:disabled=move || generating.get() || applying.get()
                  on:change=on_hero_input
                />
              </div>
            </div>

            // Description
            <div class="space-y-2">
              <label class="text-sm font-semibold text-foreground">
                {move || {
                  let i18n = I18n { locale: ctx.locale.get() };
                  i18n.t("builder.desc.title")
                }}
                <span class="ml-1 text-primary">"*"</span>
              </label>
              <textarea
                class="no-drag select-text min-h-[110px] w-full resize-y rounded-2xl border border-border/70 bg-background/80 px-3.5 py-3 text-sm leading-relaxed text-foreground shadow-inner outline-none ring-1 ring-transparent transition-shadow placeholder:text-muted-foreground focus:border-primary/40 focus:ring-primary/20 disabled:opacity-60"
                rows="4"
                prop:value=move || draft.get()
                prop:disabled=move || generating.get() || applying.get()
                placeholder=move || {
                  let i18n = I18n { locale: ctx.locale.get() };
                  i18n.t("builder.placeholder")
                }
                on:input=move |ev| {
                  if let Some(t) = ev
                    .target()
                    .and_then(|t| t.dyn_into::<web_sys::HtmlTextAreaElement>().ok())
                  {
                    draft.set(t.value());
                  }
                }
                on:keydown=move |ev: web_sys::KeyboardEvent| {
                  if ev.key() == "Enter" && (ev.meta_key() || ev.ctrl_key()) {
                    ev.prevent_default();
                    run_generate(draft.get_untracked(), ctx.locale.get_untracked());
                  }
                }
              />
            </div>

            <Show when=move || {
              !generating.get()
                && build_reply
                  .get()
                  .as_ref()
                  .map(|s| s.is_empty())
                  .unwrap_or(true)
                && package_path.get().is_none()
            }>
              <div class="flex flex-wrap gap-2">
                <SuggestionChip
                  label_key="builder.suggest.neon"
                  prompt_key="builder.suggest.neon.prompt"
                  dispatch=chip_dispatch
                  locale=ctx.locale
                />
                <SuggestionChip
                  label_key="builder.suggest.minimal"
                  prompt_key="builder.suggest.minimal.prompt"
                  dispatch=chip_dispatch
                  locale=ctx.locale
                />
                <SuggestionChip
                  label_key="builder.suggest.composer"
                  prompt_key="builder.suggest.composer.prompt"
                  dispatch=chip_dispatch
                  locale=ctx.locale
                />
              </div>
            </Show>
          </div>

          <div class="border-t border-border/50 bg-background/30 px-4 py-3 sm:px-5">
            <button
              type="button"
              class="inline-flex h-12 w-full items-center justify-center gap-2 rounded-2xl bg-gradient-to-r from-primary to-primary/85 px-5 text-sm font-semibold text-primary-foreground shadow-lg shadow-primary/30 transition-all hover:brightness-110 active:scale-[0.99] disabled:pointer-events-none disabled:opacity-50"
              disabled=move || {
                generating.get()
                  || applying.get()
                  || draft.get().trim().is_empty()
                  || hero_data_url.get().is_none()
                  || workspace_error.get().is_some()
              }
              on:click=move |_| {
                run_generate(draft.get_untracked(), ctx.locale.get_untracked());
              }
            >
              {move || {
                if generating.get() {
                  view! {
                    <LoaderCircle class="size-4 animate-spin" />
                    <span>
                      {move || {
                        let i18n = I18n { locale: ctx.locale.get() };
                        i18n.t("builder.generating")
                      }}
                    </span>
                  }.into_any()
                } else {
                  view! {
                    <WandSparkles class="size-4" />
                    <span>
                      {move || {
                        let i18n = I18n { locale: ctx.locale.get() };
                        i18n.t("builder.generate")
                      }}
                    </span>
                  }.into_any()
                }
              }}
            </button>
          </div>
        </div>

        // ── Right: live ACP stream ───────────────────────────────────
        <div class="flex min-h-0 flex-col overflow-hidden rounded-3xl border border-border/60 bg-card/75 shadow-xl shadow-black/5 backdrop-blur-xl">
          <div class="flex items-center justify-between gap-2 border-b border-border/40 bg-gradient-to-r from-chart-2/10 via-transparent to-primary/10 px-5 py-3">
            <div class="flex items-center gap-2">
              <span class=move || {
                if generating.get() {
                  "size-2 animate-pulse rounded-full bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.8)]"
                } else if build_reply.get().as_ref().is_some_and(|s| !s.is_empty()) {
                  "size-2 rounded-full bg-primary"
                } else {
                  "size-2 rounded-full bg-muted-foreground/40"
                }
              } />
              <p class="text-sm font-semibold text-foreground">
                {move || {
                  let i18n = I18n { locale: ctx.locale.get() };
                  i18n.t("builder.response")
                }}
              </p>
            </div>
            <Show when=move || generating.get()>
              <span class="inline-flex items-center gap-1.5 rounded-full bg-emerald-500/15 px-2.5 py-0.5 text-[11px] font-medium text-emerald-600 dark:text-emerald-400">
                <LoaderCircle class="size-3 animate-spin" />
                "live"
              </span>
            </Show>
          </div>

          <div
            node_ref=stream_ref
            class="no-drag select-text min-h-0 flex-1 overflow-y-auto px-4 py-4 sm:px-5"
          >
            <Show when=move || {
              build_reply
                .get()
                .as_ref()
                .map(|s| s.is_empty())
                .unwrap_or(true)
                && !generating.get()
            }>
              <div class="flex h-full min-h-[220px] flex-col items-center justify-center gap-3 px-4 text-center">
                <div class="flex size-16 items-center justify-center rounded-3xl bg-gradient-to-br from-primary/20 to-chart-2/15 text-primary ring-1 ring-primary/25">
                  <WandSparkles class="size-7" />
                </div>
                <p class="max-w-sm text-sm leading-relaxed text-muted-foreground">
                  {move || {
                    let i18n = I18n { locale: ctx.locale.get() };
                    i18n.t("builder.stream.empty")
                  }}
                </p>
              </div>
            </Show>

            <Show when=move || {
              generating.get()
                && build_reply
                  .get()
                  .as_ref()
                  .map(|s| s.is_empty())
                  .unwrap_or(true)
            }>
              <div class="flex flex-col gap-3 py-6">
                <div class="flex items-center gap-2 text-sm text-muted-foreground">
                  <LoaderCircle class="size-4 animate-spin text-primary" />
                  {move || {
                    let i18n = I18n { locale: ctx.locale.get() };
                    i18n.t("builder.generating")
                  }}
                </div>
                <div class="space-y-2">
                  <div class="h-2.5 w-3/4 animate-pulse rounded-full bg-muted/80" />
                  <div class="h-2.5 w-1/2 animate-pulse rounded-full bg-muted/60" />
                  <div class="h-2.5 w-2/3 animate-pulse rounded-full bg-muted/50" />
                </div>
              </div>
            </Show>

            <Show when=move || {
              build_reply
                .get()
                .as_ref()
                .is_some_and(|s| !s.is_empty())
            }>
              <div class="rounded-2xl border border-border/50 bg-background/70 p-4 shadow-inner">
                <pre class="select-text whitespace-pre-wrap break-words font-sans text-sm leading-relaxed text-foreground">
                  {move || build_reply.get().unwrap_or_default()}
                  <Show when=move || generating.get()>
                    <span class="ml-0.5 inline-block h-4 w-1.5 animate-pulse bg-primary align-middle" />
                  </Show>
                </pre>
              </div>
            </Show>
          </div>

          // Apply bar
          <Show when=move || package_path.get().is_some() && !generating.get()>
            <div class="border-t border-primary/20 bg-gradient-to-r from-primary/10 to-chart-2/10 px-4 py-3 sm:px-5">
              <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                <div class="min-w-0">
                  <p class="text-sm font-medium text-foreground">
                    {move || {
                      if let Some(name) = applied_name.get() {
                        format!("✓ {name}")
                      } else {
                        let i18n = I18n { locale: ctx.locale.get() };
                        i18n.t("builder.package.ready").to_string()
                      }
                    }}
                  </p>
                  <p class="mt-0.5 truncate font-mono text-[11px] text-muted-foreground">
                    {move || {
                      package_path
                        .get()
                        .as_deref()
                        .map(path_basename)
                        .unwrap_or_default()
                    }}
                  </p>
                </div>
                <button
                  type="button"
                  class="inline-flex h-11 shrink-0 items-center justify-center gap-2 rounded-2xl bg-primary px-5 text-sm font-semibold text-primary-foreground shadow-lg shadow-primary/25 transition-all hover:bg-primary/90 active:scale-[0.98] disabled:opacity-50"
                  disabled=move || applying.get()
                  on:click=move |_| run_apply(ctx.locale.get_untracked())
                >
                  {move || {
                    if applying.get() {
                      view! {
                        <LoaderCircle class="size-4 animate-spin" />
                        <span>
                          {move || {
                            let i18n = I18n { locale: ctx.locale.get() };
                            i18n.t("builder.applying")
                          }}
                        </span>
                      }.into_any()
                    } else if applied_name.get().is_some() {
                      view! {
                        <Check class="size-4" />
                        <span>
                          {move || {
                            let i18n = I18n { locale: ctx.locale.get() };
                            i18n.t("builder.apply")
                          }}
                        </span>
                      }.into_any()
                    } else {
                      view! {
                        <Play class="size-4" />
                        <span>
                          {move || {
                            let i18n = I18n { locale: ctx.locale.get() };
                            i18n.t("builder.apply")
                          }}
                        </span>
                      }.into_any()
                    }
                  }}
                </button>
              </div>
            </div>
          </Show>

          <Show when=move || {
            !generating.get()
              && package_path.get().is_none()
              && build_reply
                .get()
                .as_ref()
                .is_some_and(|s| !s.is_empty())
          }>
            <div class="border-t border-border/40 px-4 py-3 text-xs leading-relaxed text-muted-foreground sm:px-5">
              {move || {
                let i18n = I18n { locale: ctx.locale.get() };
                i18n.t("builder.package.missing")
              }}
            </div>
          </Show>
        </div>
      </div>
    </div>
  }
}

fn step_pill_class(active: bool) -> &'static str {
  if active {
    "inline-flex items-center rounded-full bg-primary/15 px-2.5 py-1 text-[11px] font-semibold text-primary ring-1 ring-primary/25"
  } else {
    "inline-flex items-center rounded-full bg-muted/50 px-2.5 py-1 text-[11px] font-medium text-muted-foreground ring-1 ring-border/50"
  }
}

/// Reopened session: chat + optional apply when a package exists in the workspace.
#[component]
fn BuilderChat(
  session_id: RwSignal<Option<String>>,
  session_title: RwSignal<String>,
  workspace_id: RwSignal<Option<String>>,
  workspace_path: RwSignal<Option<String>>,
  messages: RwSignal<Vec<ChatMessage>>,
  draft: RwSignal<String>,
  sending: RwSignal<bool>,
  next_id: RwSignal<u64>,
  chat_loading: RwSignal<bool>,
  package_path: RwSignal<Option<String>>,
  applying: RwSignal<bool>,
  applied_name: RwSignal<Option<String>>,
  list_ref: NodeRef<leptos::html::Div>,
  on_back: Callback<()>,
) -> impl IntoView {
  let ctx = AppCtx::use_ctx();

  let scroll_to_bottom = move || {
    if let Some(el) = list_ref.get_untracked() {
      let _ = el.set_scroll_top(el.scroll_height());
    }
  };

  Effect::new(move |_| {
    let _ = messages.get();
    scroll_to_bottom();
  });

  let dispatch_prompt = move |text: String, locale: Locale| {
    let text = text.trim().to_string();
    if text.is_empty() || sending.get_untracked() || chat_loading.get_untracked() {
      return;
    }
    if workspace_path.get_untracked().is_none() && session_id.get_untracked().is_none() {
      let i18n = I18n { locale };
      toast_error(
        i18n.t("builder.error"),
        "Workspace not ready. Go back and start theme build again.",
      );
      return;
    }

    let user_id = next_id.get_untracked();
    next_id.update(|n| *n += 1);
    let assistant_id = next_id.get_untracked();
    next_id.update(|n| *n += 1);

    messages.update(|msgs| {
      msgs.push(ChatMessage {
        id: user_id,
        role: ChatRole::User,
        content: text.clone(),
        pending: false,
      });
      msgs.push(ChatMessage {
        id: assistant_id,
        role: ChatRole::Assistant,
        content: String::new(),
        pending: true,
      });
    });
    draft.set(String::new());
    sending.set(true);
    package_path.set(None);

    if session_title.get_untracked().trim().is_empty() {
      let t: String = text.chars().take(48).collect();
      session_title.set(t);
    }

    let resume = session_id.get_untracked();
    let ws_path = workspace_path.get_untracked();
    let ws_id = workspace_id.get_untracked();

    spawn_local(async move {
      match api::codex_chat(text, resume.clone(), ws_path, ws_id, Some(180_000)).await {
        Ok(result) => {
          if let Some(sid) = result.session_id.filter(|s| !s.is_empty()) {
            session_id.set(Some(sid));
          }
          let body = if result.reply.trim().is_empty() {
            result.message.clone()
          } else {
            result.reply.clone()
          };
          let body = if body.trim().is_empty() {
            "(Empty Codex reply)".to_string()
          } else {
            body
          };
          messages.update(|msgs| {
            if let Some(m) = msgs.iter_mut().find(|m| m.id == assistant_id) {
              m.pending = false;
              m.content = body;
            } else if let Some(m) = msgs
              .iter_mut()
              .rev()
              .find(|m| m.role == ChatRole::Assistant && m.pending)
            {
              m.pending = false;
              m.content = body;
            }
          });
          package_path.set(result.package_path.filter(|p| !p.trim().is_empty()));
        }
        Err(e) => {
          messages.update(|msgs| {
            if let Some(m) = msgs.iter_mut().find(|m| m.id == assistant_id) {
              m.pending = false;
              m.content = e.clone();
            } else if let Some(m) = msgs
              .iter_mut()
              .rev()
              .find(|m| m.role == ChatRole::Assistant && m.pending)
            {
              m.pending = false;
              m.content = e.clone();
            }
          });
          let i18n = I18n { locale };
          toast_error(i18n.t("builder.error"), &e);
        }
      }
      sending.set(false);
    });
  };

  let run_apply = move |locale: Locale| {
    if applying.get_untracked() || sending.get_untracked() {
      return;
    }
    let Some(ws) = workspace_path.get_untracked() else {
      let i18n = I18n { locale };
      toast_error(i18n.t("builder.error"), "Workspace not ready.");
      return;
    };
    let pkg = package_path.get_untracked();
    applying.set(true);
    spawn_local(async move {
      match api::apply_built_theme(ws, pkg).await {
        Ok(result) => {
          applied_name.set(Some(result.theme_name.clone()));
          let i18n = I18n { locale };
          toast_success(i18n.t("builder.apply.success"), &result.theme_name);
        }
        Err(e) => {
          let i18n = I18n { locale };
          toast_error(i18n.t("builder.error"), &e);
        }
      }
      applying.set(false);
    });
  };

  view! {
    <header class="mb-3 flex shrink-0 items-center gap-2 sm:mb-4">
      <button
        type="button"
        class="inline-flex size-9 shrink-0 items-center justify-center rounded-xl border border-border/60 bg-card/80 text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
        on:click=move |_| on_back.run(())
        aria-label=move || {
          let i18n = I18n { locale: ctx.locale.get() };
          i18n.t("builder.back")
        }
      >
        <ArrowLeft class="size-4" />
      </button>
      <div class="min-w-0 flex-1">
        <h1 class="truncate text-lg font-semibold tracking-tight text-foreground sm:text-xl">
          {move || {
            let t = session_title.get();
            if t.trim().is_empty() {
              let i18n = I18n { locale: ctx.locale.get() };
              i18n.t("builder.chat.new").to_string()
            } else {
              t
            }
          }}
        </h1>
        <p class="truncate font-mono text-[11px] text-muted-foreground">
          {move || {
            let sid = session_id.get();
            let wid = workspace_id.get();
            match (wid, sid) {
              (Some(wid), Some(sid)) => {
                format!("{} · session {}", short_id(&wid), short_id(&sid))
              }
              (Some(wid), None) => format!("workspace {}", short_id(&wid)),
              (None, Some(sid)) => format!("session {}", short_id(&sid)),
              (None, None) => {
                let i18n = I18n { locale: ctx.locale.get() };
                i18n.t("builder.chat.unsaved").to_string()
              }
            }
          }}
        </p>
      </div>
    </header>

    <div class="relative flex min-h-0 flex-1 flex-col overflow-hidden rounded-3xl border border-border/70 bg-card/70 shadow-xl shadow-black/5 backdrop-blur-md">
      <div class="pointer-events-none absolute -right-16 -top-16 size-48 rounded-full bg-primary/10 blur-3xl" />
      <div class="pointer-events-none absolute -bottom-20 -left-10 size-44 rounded-full bg-chart-2/10 blur-3xl" />

      <div
        node_ref=list_ref
        class="no-drag select-text relative z-10 min-h-0 flex-1 space-y-3 overflow-y-auto px-4 py-4 sm:px-5 sm:py-5"
      >
        <Show when=move || chat_loading.get()>
          <div class="flex items-center justify-center gap-2 py-16 text-sm text-muted-foreground">
            <LoaderCircle class="size-4 animate-spin" />
            {move || {
              let i18n = I18n { locale: ctx.locale.get() };
              i18n.t("builder.session.loading")
            }}
          </div>
        </Show>

        <Show when=move || !chat_loading.get()>
          <For
            each=move || messages.get()
            key=|m| m.id
            children=move |m| {
              let id = m.id;
              let role = m.role;
              view! {
                <div class=move || match role {
                  ChatRole::User => "flex justify-end",
                  ChatRole::Assistant | ChatRole::System => "flex justify-start",
                }>
                  <div class=move || match role {
                    ChatRole::User => {
                      "no-drag select-text max-w-[88%] rounded-2xl rounded-br-md bg-primary px-3.5 py-2.5 text-sm leading-relaxed text-primary-foreground shadow-md shadow-primary/20 sm:max-w-[75%] cursor-text"
                    }
                    ChatRole::Assistant => {
                      "no-drag select-text max-w-[88%] rounded-2xl rounded-bl-md border border-border/60 bg-background/80 px-3.5 py-2.5 text-sm leading-relaxed text-foreground shadow-sm sm:max-w-[75%] cursor-text"
                    }
                    ChatRole::System => {
                      "no-drag select-text max-w-[92%] rounded-2xl border border-dashed border-border/70 bg-muted/40 px-3.5 py-2.5 text-xs leading-relaxed text-muted-foreground sm:max-w-[85%] cursor-text"
                    }
                  }>
                    {move || {
                      let snapshot = messages
                        .get()
                        .into_iter()
                        .find(|x| x.id == id);
                      match snapshot {
                        Some(msg) if msg.pending => view! {
                          <span class="inline-flex items-center gap-2 text-muted-foreground">
                            <LoaderCircle class="size-3.5 animate-spin" />
                            {move || {
                              let i18n = I18n { locale: ctx.locale.get() };
                              i18n.t("builder.thinking")
                            }}
                          </span>
                        }.into_any(),
                        Some(msg) => view! {
                          <p class="select-text whitespace-pre-wrap break-words">{msg.content}</p>
                        }.into_any(),
                        None => view! { <></> }.into_any(),
                      }
                    }}
                  </div>
                </div>
              }
            }
          />
        </Show>
      </div>

      <Show when=move || package_path.get().is_some() && !sending.get()>
        <div class="relative z-10 border-t border-primary/20 bg-primary/5 px-3 py-3 sm:px-4">
          <div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
            <p class="text-xs text-muted-foreground">
              {move || {
                if let Some(name) = applied_name.get() {
                  format!("✓ {name}")
                } else {
                  let i18n = I18n { locale: ctx.locale.get() };
                  i18n.t("builder.package.ready").to_string()
                }
              }}
            </p>
            <button
              type="button"
              class="inline-flex h-10 items-center justify-center gap-2 rounded-xl bg-primary px-4 text-sm font-semibold text-primary-foreground shadow-md shadow-primary/25 transition-all hover:bg-primary/90 disabled:opacity-50"
              disabled=move || applying.get()
              on:click=move |_| run_apply(ctx.locale.get_untracked())
            >
              {move || {
                if applying.get() {
                  view! {
                    <LoaderCircle class="size-4 animate-spin" />
                    <span>
                      {move || {
                        let i18n = I18n { locale: ctx.locale.get() };
                        i18n.t("builder.applying")
                      }}
                    </span>
                  }.into_any()
                } else {
                  view! {
                    <Play class="size-4" />
                    <span>
                      {move || {
                        let i18n = I18n { locale: ctx.locale.get() };
                        i18n.t("builder.apply")
                      }}
                    </span>
                  }.into_any()
                }
              }}
            </button>
          </div>
        </div>
      </Show>

      <div class="no-drag relative z-10 border-t border-border/50 bg-background/40 px-3 py-3 sm:px-4">
        <div class="flex items-end gap-2 rounded-2xl border border-border/70 bg-card/90 p-2 shadow-inner shadow-black/5 ring-1 ring-transparent transition-shadow focus-within:ring-primary/25">
          <textarea
            class="no-drag select-text max-h-36 min-h-[44px] flex-1 resize-none bg-transparent px-2 py-2 text-sm leading-relaxed text-foreground outline-none placeholder:text-muted-foreground disabled:opacity-60"
            rows="2"
            prop:value=move || draft.get()
            prop:disabled=move || sending.get() || chat_loading.get() || applying.get()
            placeholder=move || {
              let i18n = I18n { locale: ctx.locale.get() };
              i18n.t("builder.placeholder")
            }
            on:input=move |ev| {
              if let Some(t) = ev
                .target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlTextAreaElement>().ok())
              {
                draft.set(t.value());
              }
            }
            on:keydown=move |ev: web_sys::KeyboardEvent| {
              if ev.key() == "Enter" && !ev.shift_key() {
                ev.prevent_default();
                dispatch_prompt(draft.get_untracked(), ctx.locale.get_untracked());
              }
            }
          />
          <button
            type="button"
            class="inline-flex h-10 shrink-0 items-center justify-center gap-1.5 rounded-xl bg-primary px-3 text-sm font-medium text-primary-foreground shadow-md shadow-primary/25 transition-all hover:bg-primary/90 active:scale-[0.97] disabled:pointer-events-none disabled:opacity-50"
            disabled=move || {
              sending.get()
                || chat_loading.get()
                || applying.get()
                || draft.get().trim().is_empty()
            }
            on:click=move |_| {
              dispatch_prompt(draft.get_untracked(), ctx.locale.get_untracked());
            }
          >
            {move || {
              if sending.get() {
                view! { <LoaderCircle class="size-4 animate-spin" /> }.into_any()
              } else {
                view! {
                  <WandSparkles class="size-4" />
                  <span class="hidden sm:inline">
                    {move || {
                      let i18n = I18n { locale: ctx.locale.get() };
                      i18n.t("builder.generate")
                    }}
                  </span>
                }.into_any()
              }
            }}
          </button>
        </div>
      </div>
    </div>
  }
}

fn short_id(id: &str) -> String {
  let id = id.trim();
  if id.is_empty() {
    return String::new();
  }
  if id.len() > 12 {
    format!("{}…", &id.chars().take(8).collect::<String>())
  } else {
    id.to_string()
  }
}

fn is_allowed_hero_file(name: &str, mime: &str) -> bool {
  let mime = mime.to_ascii_lowercase();
  if mime.starts_with("image/jpeg")
    || mime.starts_with("image/png")
    || mime.starts_with("image/webp")
    || mime.starts_with("image/gif")
  {
    return true;
  }
  // Some browsers leave mime empty for drag-drop; fall back to extension.
  let lower = name.to_ascii_lowercase();
  lower.ends_with(".jpg")
    || lower.ends_with(".jpeg")
    || lower.ends_with(".png")
    || lower.ends_with(".webp")
    || lower.ends_with(".gif")
}

async fn read_file_data_url(file: &File) -> Result<String, String> {
  let reader = web_sys::FileReader::new().map_err(|_| "FileReader unavailable".to_string())?;
  reader
    .read_as_data_url(file)
    .map_err(|_| "failed to start reading image".to_string())?;

  let reader_clone = reader.clone();
  let promise = js_sys::Promise::new(&mut |resolve, reject| {
    let reader_ok = reader_clone.clone();
    let resolve_ok = resolve.clone();
    let reject_err = reject.clone();
    let reject_load = reject.clone();

    let onload = Closure::once_into_js(move || match reader_ok.result() {
      Ok(v) => {
        let _ = resolve_ok.call1(&wasm_bindgen::JsValue::NULL, &v);
      }
      Err(e) => {
        let _ = reject_err.call1(&wasm_bindgen::JsValue::NULL, &e);
      }
    });

    let onerror = Closure::once_into_js(move || {
      let _ = reject_load.call1(
        &wasm_bindgen::JsValue::NULL,
        &wasm_bindgen::JsValue::from_str("failed to read image"),
      );
    });

    reader_clone.set_onload(Some(onload.as_ref().unchecked_ref()));
    reader_clone.set_onerror(Some(onerror.as_ref().unchecked_ref()));
  });

  let result = JsFuture::from(promise)
    .await
    .map_err(|_| "failed to read image".to_string())?;
  result
    .as_string()
    .ok_or_else(|| "image content is not a data URL".into())
}

/// Last path segment only (no full theme_builder path).
fn path_basename(path: &str) -> String {
  let path = path.trim().trim_end_matches(['/', '\\']);
  path
    .rsplit(['/', '\\'])
    .next()
    .filter(|s| !s.is_empty())
    .unwrap_or(path)
    .to_string()
}

fn format_session_meta(id: &str, updated_at: &str) -> String {
  let sid = short_id(id);
  let when = if updated_at.is_empty() {
    String::new()
  } else {
    updated_at
      .split('T')
      .next()
      .unwrap_or(updated_at)
      .to_string()
  };
  if when.is_empty() {
    sid
  } else {
    format!("{when} · {sid}")
  }
}
