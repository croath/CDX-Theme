//! Theme Builder — generate a theme with Codex (ACP), then apply into the library.

mod builder_chat;
mod builder_home;
mod builder_new_build;
mod builder_runtime_setup;

use icons::{Check, ChevronDown, LoaderCircle, WandSparkles};
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::JsFuture;
use web_sys::File;

use crate::api::{self, CodexModelOption, CodexSessionSummary, ThemeBuilderRuntimeStatus};
use crate::components::ui::sonner::toast_error;
use crate::i18n::I18n;
use crate::state::AppCtx;

use builder_chat::BuilderChat;
use builder_home::BuilderHome;
use builder_new_build::BuilderNewBuild;
use builder_runtime_setup::BuilderRuntimeSetup;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ChatRole {
  User,
  Assistant,
  System,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ChatMessage {
  pub id: u64,
  pub role: ChatRole,
  pub content: String,
  pub pending: bool,
}

/// Home (session list) vs new generate flow vs reopened session chat.
#[derive(Clone, Debug, PartialEq, Eq)]
enum BuilderView {
  Home,
  NewBuild,
  Chat,
}

/// Host runtime gate before Home / NewBuild / Chat.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RuntimeGate {
  Checking,
  NeedSetup,
  Ready,
}

#[component]
pub fn ThemeBuilderPage() -> impl IntoView {
  let ctx = AppCtx::use_ctx();
  let view = RwSignal::new(BuilderView::Home);
  let runtime_gate = RwSignal::new(RuntimeGate::Checking);
  let runtime_status = RwSignal::new(Option::<ThemeBuilderRuntimeStatus>::None);

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

  // Shared Codex model selection (NewBuild + Chat).
  let models = RwSignal::new(Vec::<CodexModelOption>::new());
  let selected_model = RwSignal::new(String::new());

  let apply_runtime_status = move |status: ThemeBuilderRuntimeStatus| {
    let ready = status.ready;
    runtime_status.set(Some(status));
    runtime_gate.set(if ready {
      RuntimeGate::Ready
    } else {
      RuntimeGate::NeedSetup
    });
  };

  let check_runtime = move || {
    runtime_gate.set(RuntimeGate::Checking);
    let locale = ctx.locale.get_untracked();
    spawn_local(async move {
      match api::check_theme_builder_runtime().await {
        Ok(status) => apply_runtime_status(status),
        Err(e) => {
          runtime_status.set(Some(ThemeBuilderRuntimeStatus {
            ready: false,
            message: e.clone(),
            ..Default::default()
          }));
          runtime_gate.set(RuntimeGate::NeedSetup);
          let i18n = I18n { locale };
          toast_error(i18n.t("builder.error"), &e);
        }
      }
    });
  };

  // First open: probe bunx / npx / codex-acp on the host.
  Effect::new(move |_| {
    check_runtime();
  });

  let refresh_sessions = move || {
    if runtime_gate.get_untracked() != RuntimeGate::Ready {
      return;
    }
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
    if runtime_gate.get() == RuntimeGate::Ready && view.get() == BuilderView::Home {
      refresh_sessions();
    }
  });

  // Load Codex model list once the host runtime is ready.
  Effect::new(move |_| {
    if runtime_gate.get() != RuntimeGate::Ready {
      return;
    }
    if !models.get_untracked().is_empty() {
      return;
    }
    spawn_local(async move {
      match api::list_codex_models().await {
        Ok(list) => {
          models.set(list.models);
          if selected_model.get_untracked().trim().is_empty() {
            if let Some(cur) = list.current.filter(|s| !s.trim().is_empty()) {
              selected_model.set(cur);
            } else if let Some(first) = models.get_untracked().first() {
              selected_model.set(first.id.clone());
            }
          }
        }
        Err(e) => {
          web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(&format!(
            "list_codex_models failed: {e}"
          )));
        }
      }
    });
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
      {move || match runtime_gate.get() {
        RuntimeGate::Checking => view! {
          <div class="flex h-full min-h-0 flex-1 flex-col items-center justify-center gap-3 text-sm text-muted-foreground">
            <div class="flex size-12 items-center justify-center rounded-2xl bg-primary/15 text-primary ring-1 ring-primary/25">
              <WandSparkles class="size-5" />
            </div>
            <div class="inline-flex items-center gap-2">
              <LoaderCircle class="size-4 animate-spin" />
              {move || {
                let i18n = I18n { locale: ctx.locale.get() };
                i18n.t("builder.runtime.checking")
              }}
            </div>
          </div>
        }.into_any(),
        RuntimeGate::NeedSetup => view! {
          <BuilderRuntimeSetup
            status=runtime_status
            on_ready=Callback::new(move |s: ThemeBuilderRuntimeStatus| {
              apply_runtime_status(s);
              view.set(BuilderView::Home);
            })
            on_recheck=Callback::new(move |_| check_runtime())
          />
        }.into_any(),
        RuntimeGate::Ready => match view.get() {
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
              models=models
              selected_model=selected_model
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
              models=models
              selected_model=selected_model
              list_ref=list_ref
              on_back=Callback::new(move |_| back_home())
            />
          }.into_any(),
        },
      }}
    </div>
  }
}

/// Compact model dropdown used on every Codex chat surface.
#[component]
pub(super) fn BuilderModelSelect(
  models: RwSignal<Vec<CodexModelOption>>,
  selected_model: RwSignal<String>,
  /// When true, the menu is non-interactive (generate / send in flight).
  disabled: Signal<bool>,
) -> impl IntoView {
  let ctx = AppCtx::use_ctx();
  let open = RwSignal::new(false);

  view! {
    <div class="relative shrink-0">
      <Show when=move || open.get()>
        <div
          class="fixed inset-0 z-20 cursor-default"
          on:click=move |_| open.set(false)
        />
      </Show>

      <button
        type="button"
        class="no-drag group relative z-30 inline-flex h-9 max-w-[11rem] items-center gap-1.5 rounded-xl border border-border/70 bg-card/90 px-2.5 text-left text-xs font-medium text-foreground shadow-sm backdrop-blur transition-all hover:border-primary/35 hover:bg-accent/50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 sm:max-w-[14rem]"
        prop:disabled=move || disabled.get() || models.get().is_empty()
        aria-haspopup="listbox"
        prop:aria-expanded=move || open.get()
        aria-label=move || {
          let i18n = I18n { locale: ctx.locale.get() };
          i18n.t("builder.model.label")
        }
        on:click=move |_| {
          if disabled.get_untracked() || models.get_untracked().is_empty() {
            return;
          }
          open.update(|v| *v = !*v);
        }
      >
        <span class="min-w-0 flex-1 truncate">
          {move || {
            let id = selected_model.get();
            let list = models.get();
            if let Some(m) = list.iter().find(|m| m.id == id) {
              m.name.clone()
            } else if !id.is_empty() {
              id
            } else {
              let i18n = I18n { locale: ctx.locale.get() };
              i18n.t("builder.model.label").to_string()
            }
          }}
        </span>
        <span class=move || {
          if open.get() {
            "inline-flex shrink-0 text-muted-foreground transition-transform duration-200 rotate-180"
          } else {
            "inline-flex shrink-0 text-muted-foreground transition-transform duration-200"
          }
        }>
          <ChevronDown class="size-3.5" />
        </span>
      </button>

      <Show when=move || open.get() && !models.get().is_empty()>
        <ul
          class="absolute right-0 top-full z-40 mt-1.5 max-h-64 w-56 list-none overflow-y-auto rounded-2xl border border-border/70 bg-popover p-1.5 shadow-2xl shadow-black/25 ring-1 ring-border/40 sm:w-64"
          role="listbox"
        >
          <For
            each=move || models.get()
            key=|m| m.id.clone()
            children=move |m| {
              let id = m.id.clone();
              let name = m.name.clone();
              let desc = m.description.clone().unwrap_or_default();
              let has_desc = !desc.is_empty();
              let id_for_active = id.clone();
              let id_for_class = id.clone();
              let id_for_click = id.clone();
              let id_for_check = id;
              view! {
                <li>
                  <button
                    type="button"
                    role="option"
                    prop:aria-selected=move || selected_model.get() == id_for_active
                    class=move || {
                      let active = selected_model.get() == id_for_class;
                      if active {
                        "flex w-full items-start justify-between gap-2 rounded-xl bg-primary/12 px-2.5 py-2 text-left transition-colors"
                      } else {
                        "flex w-full items-start justify-between gap-2 rounded-xl px-2.5 py-2 text-left transition-colors hover:bg-accent/60"
                      }
                    }
                    on:click=move |_| {
                      selected_model.set(id_for_click.clone());
                      open.set(false);
                    }
                  >
                    <div class="min-w-0">
                      <div class="truncate text-xs font-medium text-foreground">{name}</div>
                      <Show when=move || has_desc>
                        <div class="mt-0.5 line-clamp-2 text-[10px] leading-snug text-muted-foreground">
                          {desc.clone()}
                        </div>
                      </Show>
                    </div>
                    <Show when=move || selected_model.get() == id_for_check>
                      <Check class="mt-0.5 size-3.5 shrink-0 text-primary" />
                    </Show>
                  </button>
                </li>
              }
            }
          />
        </ul>
      </Show>
    </div>
  }
}

pub(super) fn short_id(id: &str) -> String {
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

pub(super) fn is_allowed_hero_file(name: &str, mime: &str) -> bool {
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

pub(super) async fn read_file_data_url(file: &File) -> Result<String, String> {
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
pub(super) fn path_basename(path: &str) -> String {
  let path = path.trim().trim_end_matches(['/', '\\']);
  path
    .rsplit(['/', '\\'])
    .next()
    .filter(|s| !s.is_empty())
    .unwrap_or(path)
    .to_string()
}

pub(super) fn format_session_meta(id: &str, updated_at: &str) -> String {
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
