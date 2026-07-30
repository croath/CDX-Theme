use icons::{ArrowLeft, Check, ImagePlus, LoaderCircle, Play, WandSparkles};
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{DragEvent, Event, File, HtmlInputElement};

use crate::api;
use crate::components::ui::SuggestionChip;
use crate::components::ui::sonner::{toast_error, toast_success};
use crate::i18n::I18n;
use crate::state::AppCtx;
use crate::types::Locale;

use crate::api::CodexModelOption;

use super::{
  BuilderModelSelect, is_allowed_hero_file, path_basename, read_file_data_url, short_id,
};

/// New theme build: hero image + description → Generate → reply + Apply.
#[component]
pub(super) fn BuilderNewBuild(
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
  models: RwSignal<Vec<CodexModelOption>>,
  selected_model: RwSignal<String>,
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
    if let Some(files) = input.files()
      && let Some(file) = files.get(0) {
        accept_hero_file(file, locale);
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
           When done, reply with a short plain-text summary only (theme name, mood/colors, what changed). \
           No code, no CSS, no actions, no file paths.",
          asset = h.theme_asset_path,
          text = text
        ),
        None => format!(
          "{text}\n\nWhen done, reply with a short plain-text summary only. \
           No code, no CSS, no actions, no file paths."
        ),
      };

      let resume = session_id.get_untracked();
      let ws_id = workspace_id.get_untracked();
      let model = {
        let m = selected_model.get_untracked();
        let m = m.trim().to_string();
        if m.is_empty() { None } else { Some(m) }
      };

      match api::codex_chat(prompt, resume, Some(ws_path), ws_id, Some(180_000), model).await {
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
      el.set_scroll_top(el.scroll_height());
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
        <BuilderModelSelect
          models=models
          selected_model=selected_model
          disabled=Signal::derive(move || generating.get() || applying.get())
        />
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
                  if let Some(dt) = ev.data_transfer()
                    && let Some(files) = dt.files()
                      && let Some(file) = files.get(0) {
                        accept_hero_file(file, locale);
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
