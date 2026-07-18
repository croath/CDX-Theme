#![allow(dead_code)]
use std::sync::atomic::{AtomicU64, Ordering};

use icons::{Check, CircleAlert, CircleX, Info, LoaderCircle, X};
use leptos::prelude::*;
use leptos::task::spawn_local;
use tw_merge::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::JsFuture;

#[derive(Clone, Copy, PartialEq, Eq, Default, strum::Display)]
pub enum ToastType {
  #[default]
  Default,
  Success,
  Error,
  Warning,
  Info,
  Loading,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, strum::Display)]
pub enum SonnerPosition {
  TopLeft,
  TopCenter,
  TopRight,
  #[default]
  BottomRight,
  BottomCenter,
  BottomLeft,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, strum::Display)]
pub enum SonnerDirection {
  TopDown,
  #[default]
  BottomUp,
}

#[derive(Clone, PartialEq)]
struct ToastItem {
  id: u64,
  title: String,
  description: String,
  variant: ToastType,
}

static TOAST_SEQ: AtomicU64 = AtomicU64::new(1);

thread_local! {
  static TOASTS: RwSignal<Vec<ToastItem>> = RwSignal::new(Vec::new());
}

fn toasts() -> RwSignal<Vec<ToastItem>> {
  TOASTS.with(|s| *s)
}

/// Push a toast into the global sonner queue.
pub fn toast(title: &str, description: &str, variant: ToastType) {
  let id = TOAST_SEQ.fetch_add(1, Ordering::Relaxed);
  let item = ToastItem {
    id,
    title: title.to_string(),
    description: description.to_string(),
    variant,
  };

  toasts().update(|list| {
    list.push(item);
    if list.len() > 6 {
      let overflow = list.len() - 6;
      list.drain(0..overflow);
    }
  });

  // Auto-dismiss (loading toasts stay until dismissed)
  if !matches!(variant, ToastType::Loading) {
    let duration_ms = match variant {
      ToastType::Error => 4500,
      ToastType::Warning => 4000,
      _ => 3200,
    };
    spawn_local(async move {
      sleep_ms(duration_ms).await;
      dismiss_toast(id);
    });
  }
}

/// Convenience helper for error toasts.
pub fn toast_error(title: &str, description: &str) {
  toast(title, description, ToastType::Error);
}

/// Convenience helper for success toasts.
pub fn toast_success(title: &str, description: &str) {
  toast(title, description, ToastType::Success);
}

pub fn dismiss_toast(id: u64) {
  toasts().update(|list| {
    list.retain(|t| t.id != id);
  });
}

async fn sleep_ms(ms: i32) {
  let Some(window) = web_sys::window() else {
    return;
  };

  let promise = js_sys::Promise::new(&mut |resolve, _reject| {
    let resolve = resolve.clone();
    let cb = Closure::once_into_js(move || {
      let _ = resolve.call0(&wasm_bindgen::JsValue::NULL);
    });
    let _ =
      window.set_timeout_with_callback_and_timeout_and_arguments_0(cb.as_ref().unchecked_ref(), ms);
  });

  let _ = JsFuture::from(promise).await;
}

#[component]
pub fn SonnerTrigger(
  children: Children,
  #[prop(into, optional)] class: String,
  #[prop(optional, default = ToastType::default())] variant: ToastType,
  #[prop(into)] title: String,
  #[prop(into)] description: String,
  #[prop(into, optional)] _position: String,
) -> impl IntoView {
  let variant_classes = match variant {
    ToastType::Default => "bg-primary text-primary-foreground shadow-xs hover:bg-primary/90",
    ToastType::Success => "bg-success text-success-foreground hover:bg-success/90",
    ToastType::Error => {
      "bg-destructive text-white shadow-xs hover:bg-destructive/90 dark:bg-destructive/60"
    }
    ToastType::Warning => "bg-warning text-warning-foreground hover:bg-warning/90",
    ToastType::Info => "bg-info text-info-foreground shadow-xs hover:bg-info/90",
    ToastType::Loading => "bg-secondary text-secondary-foreground shadow-xs hover:bg-secondary/80",
  };

  let merged_class = tw_merge!(
    "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium transition-all disabled:pointer-events-none disabled:opacity-50 outline-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] w-fit cursor-pointer h-9 px-4 py-2",
    variant_classes,
    class
  );

  let title_c = title.clone();
  let description_c = description.clone();

  view! {
      <button
          class=merged_class
          data-name="SonnerTrigger"
          data-variant=variant.to_string()
          data-toast-title=title
          data-toast-description=description
          on:click=move |_| toast(&title_c, &description_c, variant)
      >
          {children()}
      </button>
  }
}

#[component]
pub fn SonnerContainer(
  children: Children,
  #[prop(into, optional)] class: String,
  #[prop(optional, default = SonnerPosition::default())] position: SonnerPosition,
) -> impl IntoView {
  let merged_class = tw_merge!("toast__container fixed z-[100] pointer-events-none", class);

  view! {
      <div class=merged_class data-position=position.to_string()>
          {children()}
      </div>
  }
}

#[component]
pub fn SonnerList(
  children: Children,
  #[prop(into, optional)] class: String,
  #[prop(optional, default = SonnerPosition::default())] position: SonnerPosition,
  #[prop(optional, default = SonnerDirection::default())] direction: SonnerDirection,
  #[prop(into, default = "false".to_string())] expanded: String,
  #[prop(into, optional)] style: String,
) -> impl IntoView {
  let flex_dir = match direction {
    SonnerDirection::TopDown => "flex-col",
    SonnerDirection::BottomUp => "flex-col-reverse",
  };

  let merged_class = tw_merge!(
    "flex relative gap-3 w-[min(400px,calc(100vw-2rem))] max-h-[70vh] overflow-visible pointer-events-none [&>*]:pointer-events-auto",
    flex_dir,
    class
  );

  view! {
      <ol
          class=merged_class
          data-name="SonnerList"
          data-sonner-toaster="true"
          data-sonner-theme="light"
          data-position=position.to_string()
          data-expanded=expanded
          data-direction=direction.to_string()
          style=style
      >
          {children()}
      </ol>
  }
}

#[component]
fn ToastCard(item: ToastItem) -> impl IntoView {
  let id = item.id;
  let variant = item.variant;
  let title = item.title.clone();
  let description = item.description.clone();
  let has_description = !description.is_empty();

  let (shell, accent, icon) = match variant {
    ToastType::Error => (
      "border-destructive/30 bg-card text-foreground shadow-destructive/10",
      "bg-destructive/15 text-destructive",
      view! { <CircleX class="size-4" /> }.into_any(),
    ),
    ToastType::Success => (
      "border-primary/30 bg-card text-foreground shadow-primary/10",
      "bg-primary/15 text-primary",
      view! { <Check class="size-4" /> }.into_any(),
    ),
    ToastType::Warning => (
      "border-amber-500/30 bg-card text-foreground",
      "bg-amber-500/15 text-amber-600 dark:text-amber-400",
      view! { <CircleAlert class="size-4" /> }.into_any(),
    ),
    ToastType::Info => (
      "border-sky-500/30 bg-card text-foreground",
      "bg-sky-500/15 text-sky-600 dark:text-sky-400",
      view! { <Info class="size-4" /> }.into_any(),
    ),
    ToastType::Loading => (
      "border-border/70 bg-card text-foreground",
      "bg-muted text-muted-foreground",
      view! { <LoaderCircle class="size-4 animate-spin" /> }.into_any(),
    ),
    ToastType::Default => (
      "border-border/70 bg-card text-foreground",
      "bg-primary/12 text-primary",
      view! { <Info class="size-4" /> }.into_any(),
    ),
  };

  let shell_class = tw_merge!(
    "pointer-events-auto flex w-full items-start gap-3 rounded-2xl border p-3.5 shadow-xl backdrop-blur-md animate-in fade-in-0 slide-in-from-bottom-2 duration-200",
    shell
  );
  let accent_class = tw_merge!(
    "mt-0.5 flex size-8 shrink-0 items-center justify-center rounded-xl",
    accent
  );

  view! {
    <li class=shell_class data-toast-id=id.to_string() role="status">
      <span class=accent_class>{icon}</span>
      <div class="min-w-0 flex-1 pt-0.5">
        <p class="text-sm font-semibold leading-tight tracking-tight">{title}</p>
        <Show when=move || has_description>
          <p class="mt-1 text-xs leading-relaxed text-muted-foreground">{description.clone()}</p>
        </Show>
      </div>
      <button
        type="button"
        class="inline-flex size-7 shrink-0 items-center justify-center rounded-lg text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
        aria-label="Dismiss"
        on:click=move |_| dismiss_toast(id)
      >
        <X class="size-3.5" />
      </button>
    </li>
  }
}

#[component]
pub fn SonnerToaster(
  #[prop(default = SonnerPosition::default())] position: SonnerPosition,
) -> impl IntoView {
  let direction = match position {
    SonnerPosition::TopLeft | SonnerPosition::TopCenter | SonnerPosition::TopRight => {
      SonnerDirection::TopDown
    }
    _ => SonnerDirection::BottomUp,
  };

  let container_class = match position {
    SonnerPosition::TopLeft => "left-4 top-4 sm:left-6 sm:top-6",
    SonnerPosition::TopRight => "right-4 top-4 sm:right-6 sm:top-6",
    SonnerPosition::TopCenter => "left-1/2 -translate-x-1/2 top-4 sm:top-6",
    SonnerPosition::BottomCenter => "left-1/2 -translate-x-1/2 bottom-4 sm:bottom-6",
    SonnerPosition::BottomLeft => "left-4 bottom-4 sm:left-6 sm:bottom-6",
    SonnerPosition::BottomRight => "right-4 bottom-4 sm:right-6 sm:bottom-6",
  };

  let items = toasts();

  view! {
      <SonnerContainer class=container_class position=position>
          <SonnerList position=position direction=direction>
              <For
                  each=move || items.get()
                  key=|t| t.id
                  children=move |item| {
                      view! { <ToastCard item=item /> }
                  }
              />
          </SonnerList>
      </SonnerContainer>
  }
}
