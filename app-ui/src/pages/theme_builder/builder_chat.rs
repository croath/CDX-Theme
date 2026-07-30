use icons::{ArrowLeft, LoaderCircle, Play, WandSparkles};
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;

use crate::api::{self, CodexModelOption};
use crate::components::ui::sonner::{toast_error, toast_success};
use crate::i18n::I18n;
use crate::state::AppCtx;
use crate::types::Locale;

use super::{BuilderModelSelect, ChatMessage, ChatRole, short_id};

/// Reopened session: chat + optional apply when a package exists in the workspace.
#[component]
pub(super) fn BuilderChat(
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
  models: RwSignal<Vec<CodexModelOption>>,
  selected_model: RwSignal<String>,
  list_ref: NodeRef<leptos::html::Div>,
  on_back: Callback<()>,
) -> impl IntoView {
  let ctx = AppCtx::use_ctx();

  let scroll_to_bottom = move || {
    if let Some(el) = list_ref.get_untracked() {
      el.set_scroll_top(el.scroll_height());
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
    let model = {
      let m = selected_model.get_untracked();
      let m = m.trim().to_string();
      if m.is_empty() { None } else { Some(m) }
    };

    spawn_local(async move {
      match api::codex_chat(text, resume.clone(), ws_path, ws_id, Some(180_000), model).await {
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
      <BuilderModelSelect
        models=models
        selected_model=selected_model
        disabled=Signal::derive(move || sending.get() || chat_loading.get() || applying.get())
      />
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
                        None => {
                            let _: () = view! { <></> };
                            ().into_any()
                        },
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
