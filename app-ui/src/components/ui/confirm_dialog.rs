use icons::X;
use leptos::prelude::*;

/// Simple modal confirm dialog (backdrop + card).
#[component]
pub fn ConfirmDialog(
  open: RwSignal<bool>,
  /// Dialog title.
  title: Signal<String>,
  /// Body description.
  description: Signal<String>,
  /// Confirm button label.
  confirm_label: Signal<String>,
  /// Cancel button label.
  cancel_label: Signal<String>,
  /// When true, confirm button shows loading and is disabled.
  #[prop(optional, into)]
  confirming: Option<Signal<bool>>,
  /// Destructive styling for the confirm action (delete, etc.).
  #[prop(optional, default = true)]
  destructive: bool,
  on_confirm: Callback<()>,
  #[prop(optional)] on_cancel: Option<Callback<()>>,
) -> impl IntoView {
  let confirming = confirming.unwrap_or_else(|| Signal::derive(|| false));

  let close = move || {
    if confirming.get_untracked() {
      return;
    }
    open.set(false);
    if let Some(cb) = on_cancel {
      cb.run(());
    }
  };

  let confirm_btn_class = if destructive {
    "inline-flex h-9 min-w-[5.5rem] items-center justify-center gap-1.5 rounded-xl bg-destructive px-4 text-sm font-medium text-white shadow-sm transition-all hover:bg-destructive/90 active:scale-[0.97] disabled:pointer-events-none disabled:opacity-60"
  } else {
    "inline-flex h-9 min-w-[5.5rem] items-center justify-center gap-1.5 rounded-xl bg-primary px-4 text-sm font-medium text-primary-foreground shadow-sm transition-all hover:bg-primary/90 active:scale-[0.97] disabled:pointer-events-none disabled:opacity-60"
  };

  view! {
    <Show when=move || open.get()>
      <div
        class="fixed inset-0 z-[100] flex items-center justify-center p-4"
        role="presentation"
      >
        // Backdrop
        <div
          class="absolute inset-0 bg-black/50 backdrop-blur-[2px]"
          on:click=move |_| close()
        />

        // Panel
        <div
          role="alertdialog"
          aria-modal="true"
          class="relative z-10 w-full max-w-sm overflow-hidden rounded-2xl border border-border/80 bg-card p-5 shadow-2xl shadow-black/20"
          on:click=move |ev| ev.stop_propagation()
        >
          <div class="flex items-start justify-between gap-3">
            <div class="min-w-0 space-y-1.5">
              <h2 class="text-base font-semibold tracking-tight text-foreground">
                {move || title.get()}
              </h2>
              <p class="text-sm leading-relaxed text-muted-foreground">
                {move || description.get()}
              </p>
            </div>
            <button
              type="button"
              class="inline-flex size-8 shrink-0 items-center justify-center rounded-lg text-muted-foreground transition-colors hover:bg-muted hover:text-foreground disabled:opacity-50"
              disabled=move || confirming.get()
              on:click=move |_| close()
              aria-label="Close"
            >
              <X class="size-4" />
            </button>
          </div>

          <div class="mt-5 flex items-center justify-end gap-2">
            <button
              type="button"
              class="inline-flex h-9 items-center justify-center rounded-xl border border-border bg-background px-4 text-sm font-medium text-foreground transition-all hover:bg-muted active:scale-[0.97] disabled:pointer-events-none disabled:opacity-60"
              disabled=move || confirming.get()
              on:click=move |_| close()
            >
              {move || cancel_label.get()}
            </button>
            <button
              type="button"
              class=confirm_btn_class
              disabled=move || confirming.get()
              on:click=move |_| {
                if confirming.get_untracked() {
                  return;
                }
                on_confirm.run(());
              }
            >
              {move || confirm_label.get()}
            </button>
          </div>
        </div>
      </div>
    </Show>
  }
}
