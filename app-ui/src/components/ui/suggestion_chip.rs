//! Compact suggestion chip for chat-style starters (Theme Builder, etc.).

use leptos::prelude::*;

use crate::i18n::I18n;
use crate::state::AppCtx;
use crate::types::Locale;

/// Pill button that resolves i18n label/prompt keys and calls `dispatch` with the prompt.
#[component]
pub fn SuggestionChip<F>(
  label_key: &'static str,
  prompt_key: &'static str,
  dispatch: F,
  locale: RwSignal<Locale>,
) -> impl IntoView
where
  F: Fn(String, Locale) + Copy + 'static,
{
  let ctx = AppCtx::use_ctx();
  view! {
    <button
      type="button"
      class="rounded-full border border-border/70 bg-background/70 px-3 py-1.5 text-xs font-medium text-foreground transition-colors hover:border-primary/40 hover:bg-primary/10 hover:text-primary"
      on:click=move |_| {
        let i18n = I18n {
          locale: locale.get_untracked(),
        };
        dispatch(i18n.t(prompt_key).to_string(), locale.get_untracked());
      }
    >
      {move || {
        let i18n = I18n { locale: ctx.locale.get() };
        i18n.t(label_key)
      }}
    </button>
  }
}
