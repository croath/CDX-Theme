use crate::api;
use crate::components::ui::sonner::{toast_error, toast_success};
use crate::i18n::I18n;
use crate::state::AppCtx;
use crate::types::Page;
use icons::{LoaderCircle, PackagePlus};
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::JsFuture;
use web_sys::{DragEvent, Event, File, HtmlInputElement};

#[component]
pub fn InstallPage() -> impl IntoView {
  let ctx = AppCtx::use_ctx();
  let installing = RwSignal::new(false);
  let drag_over = RwSignal::new(false);
  let file_input: NodeRef<leptos::html::Input> = NodeRef::new();

  let process_file = move |file: File| {
    if installing.get_untracked() {
      return;
    }
    let name = file.name();
    let locale = ctx.locale.get_untracked();

    installing.set(true);
    spawn_local(async move {
      let i18n = I18n { locale };
      match read_file_text(&file).await {
        Ok(content) => {
          // Validity is JSON content (backend deserializes); filename is not required.
          if !looks_like_theme_package_json(&content) {
            installing.set(false);
            toast_error(i18n.t("install.error"), i18n.t("install.invalid"));
            return;
          }
          match api::install_theme(name.clone(), content).await {
            Ok(meta) => {
              installing.set(false);
              toast_success(i18n.t("install.success"), &meta.name);
              // Show it in the recommend list (install tag).
              ctx.page.set(Page::Recommend);
            }
            Err(e) => {
              installing.set(false);
              toast_error(i18n.t("install.error"), &e);
            }
          }
        }
        Err(e) => {
          installing.set(false);
          toast_error(i18n.t("install.error"), &e);
        }
      }
    });
  };

  let on_input_change = move |ev: Event| {
    let Some(input) = ev
      .target()
      .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
    else {
      return;
    };
    let Some(files) = input.files() else {
      return;
    };
    if let Some(file) = files.get(0) {
      process_file(file);
    }
    // Allow re-selecting the same file
    input.set_value("");
  };

  let on_drop = move |ev: DragEvent| {
    ev.prevent_default();
    drag_over.set(false);
    if installing.get_untracked() {
      return;
    }
    let Some(dt) = ev.data_transfer() else {
      return;
    };
    let Some(files) = dt.files() else {
      return;
    };
    if let Some(file) = files.get(0) {
      process_file(file);
    }
  };

  let on_drag_over = move |ev: DragEvent| {
    ev.prevent_default();
    drag_over.set(true);
  };

  let on_drag_leave = move |ev: DragEvent| {
    ev.prevent_default();
    drag_over.set(false);
  };

  let browse = move |_| {
    if let Some(input) = file_input.get() {
      input.click();
    }
  };

  view! {
    <div class="flex h-full flex-col">
      <header class="mb-6">
        <h1 class="bg-gradient-to-r from-foreground via-foreground to-primary bg-clip-text text-2xl font-semibold tracking-tight text-transparent sm:text-3xl">
          {move || {
            let i18n = I18n { locale: ctx.locale.get() };
            i18n.t("install.title")
          }}
        </h1>
        <p class="mt-1.5 max-w-xl text-sm text-muted-foreground">
          {move || {
            let i18n = I18n { locale: ctx.locale.get() };
            i18n.t("install.subtitle")
          }}
        </p>
      </header>

      <div class="flex min-h-0 flex-1 flex-col items-stretch justify-start">
        <div
          class=move || {
            // no-drag: allow file drop without capturing window drag
            let base = "no-drag relative flex flex-col items-center justify-center gap-4 rounded-3xl border-2 border-dashed px-6 py-16 text-center transition-all";
            if drag_over.get() {
              format!("{base} border-primary bg-primary/10 shadow-inner shadow-primary/10")
            } else if installing.get() {
              format!("{base} border-border/70 bg-muted/20 opacity-90")
            } else {
              format!("{base} border-border/70 bg-card/50 hover:border-primary/40 hover:bg-primary/5")
            }
          }
          on:dragover=on_drag_over
          on:dragleave=on_drag_leave
          on:drop=on_drop
        >
          <div class="flex size-16 items-center justify-center rounded-2xl bg-primary/15 text-primary ring-1 ring-primary/25">
            {move || {
              if installing.get() {
                view! { <LoaderCircle class="size-8 animate-spin" /> }.into_any()
              } else {
                view! { <PackagePlus class="size-8" /> }.into_any()
              }
            }}
          </div>

          <div class="space-y-1">
            <p class="text-base font-medium text-foreground">
              {move || {
                let i18n = I18n { locale: ctx.locale.get() };
                if installing.get() {
                  i18n.t("install.installing")
                } else {
                  i18n.t("install.drop")
                }
              }}
            </p>
            <p class="text-sm text-muted-foreground">
              {move || {
                let i18n = I18n { locale: ctx.locale.get() };
                i18n.t("install.or")
              }}
            </p>
          </div>

          <button
            type="button"
            class="inline-flex h-10 items-center justify-center gap-2 rounded-xl bg-primary px-5 text-sm font-medium text-primary-foreground shadow-sm transition-all hover:bg-primary/90 active:scale-[0.97] disabled:pointer-events-none disabled:opacity-60"
            disabled=move || installing.get()
            on:click=browse
          >
            {move || {
              let i18n = I18n { locale: ctx.locale.get() };
              i18n.t("install.browse")
            }}
          </button>

          <p class="max-w-sm text-xs text-muted-foreground">
            {move || {
              let i18n = I18n { locale: ctx.locale.get() };
              i18n.t("install.hint")
            }}
          </p>

          <input
            node_ref=file_input
            type="file"
            accept=".cdxtheme,.json,application/json,text/json"
            class="hidden"
            on:change=on_input_change
          />
        </div>
      </div>
    </div>
  }
}

/// Lightweight content probe before calling the backend (full validation is server-side).
fn looks_like_theme_package_json(content: &str) -> bool {
  let t = content.trim_start();
  if !t.starts_with('{') {
    return false;
  }
  // Need multi-app package markers — not a full schema check.
  let has_format =
    t.contains("\"format\"") && (t.contains("cdxtheme") || t.contains("codedrobe-theme"));
  let has_theme = t.contains("\"theme\"");
  let has_targets = t.contains("\"targets\"");
  has_format && has_theme && has_targets
}

async fn read_file_text(file: &File) -> Result<String, String> {
  let reader = web_sys::FileReader::new().map_err(|_| "FileReader unavailable".to_string())?;
  reader
    .read_as_text(file)
    .map_err(|_| "failed to start reading file".to_string())?;

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
        &wasm_bindgen::JsValue::from_str("failed to read file"),
      );
    });

    reader_clone.set_onload(Some(onload.as_ref().unchecked_ref()));
    reader_clone.set_onerror(Some(onerror.as_ref().unchecked_ref()));
  });

  let result = JsFuture::from(promise)
    .await
    .map_err(|_| "failed to read file".to_string())?;
  result
    .as_string()
    .ok_or_else(|| "file content is not text".into())
}
