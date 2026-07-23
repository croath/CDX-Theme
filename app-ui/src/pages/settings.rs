use icons::{Check, ChevronDown, Globe, Languages, Moon, Sun};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api;
use crate::components::ui::sonner::{ToastType, toast, toast_error};
use crate::i18n::I18n;
use crate::state::AppCtx;
use crate::types::Locale;

#[component]
pub fn SettingsPage() -> impl IntoView {
  let ctx = AppCtx::use_ctx();
  // Lifted so the whole language card can raise above the theme card while open
  let lang_open = RwSignal::new(false);

  view! {
    <div class="flex h-full flex-col overflow-visible">
      <header class="mb-6 shrink-0">
        <h1 class="bg-gradient-to-r from-foreground via-foreground to-primary bg-clip-text text-2xl font-semibold tracking-tight text-transparent sm:text-3xl">
          {move || {
            let i18n = I18n { locale: ctx.locale.get() };
            i18n.t("settings.title")
          }}
        </h1>
        <p class="mt-1.5 max-w-xl text-sm text-muted-foreground">
          {move || {
            let i18n = I18n { locale: ctx.locale.get() };
            i18n.t("settings.subtitle")
          }}
        </p>
      </header>

      <div class="relative isolate mx-auto w-full max-w-2xl space-y-5 overflow-visible">
        // Language card
        <section class=move || {
          if lang_open.get() {
            "relative z-30 overflow-visible rounded-3xl border border-border/70 bg-card shadow-sm"
          } else {
            "relative z-20 overflow-visible rounded-3xl border border-border/70 bg-card/80 shadow-sm backdrop-blur-md"
          }
        }>
          <div class="flex items-start gap-4 border-b border-border/50 px-5 py-4">
            <div class="flex size-10 shrink-0 items-center justify-center rounded-xl bg-primary/12 text-primary ring-1 ring-primary/20">
              <Languages class="size-5" />
            </div>
            <div>
              <h2 class="text-sm font-semibold text-foreground">
                {move || {
                  let i18n = I18n { locale: ctx.locale.get() };
                  i18n.t("settings.language")
                }}
              </h2>
              <p class="mt-0.5 text-xs text-muted-foreground">
                {move || {
                  let i18n = I18n { locale: ctx.locale.get() };
                  i18n.t("settings.language.hint")
                }}
              </p>
            </div>
          </div>

          <div class="p-4">
            <LanguageDropdown open=lang_open />
          </div>
        </section>

        // CDP port
        <section class="relative z-10 overflow-hidden rounded-3xl border border-border/70 bg-card/80 shadow-sm backdrop-blur-md">
          <div class="flex items-start gap-4 border-b border-border/50 px-5 py-4">
            <div class="flex size-10 shrink-0 items-center justify-center rounded-xl bg-primary/12 text-primary ring-1 ring-primary/20">
              <Globe class="size-5" />
            </div>
            <div>
              <h2 class="text-sm font-semibold text-foreground">
                {move || {
                  let i18n = I18n { locale: ctx.locale.get() };
                  i18n.t("settings.cdp")
                }}
              </h2>
              <p class="mt-0.5 text-xs text-muted-foreground">
                {move || {
                  let i18n = I18n { locale: ctx.locale.get() };
                  i18n.t("settings.cdp.port.hint")
                }}
              </p>
            </div>
          </div>
          <div class="p-4">
            <CdpPortSetting />
          </div>
        </section>

        // Theme card
        <section class="relative z-10 overflow-hidden rounded-3xl border border-border/70 bg-card/80 shadow-sm backdrop-blur-md">
          <div class="flex items-start gap-4 border-b border-border/50 px-5 py-4">
            <div class="flex size-10 shrink-0 items-center justify-center rounded-xl bg-primary/12 text-primary ring-1 ring-primary/20">
              <Globe class="size-5" />
            </div>
            <div>
              <h2 class="text-sm font-semibold text-foreground">
                {move || {
                  let i18n = I18n { locale: ctx.locale.get() };
                  i18n.t("settings.theme")
                }}
              </h2>
              <p class="mt-0.5 text-xs text-muted-foreground">
                {move || {
                  let i18n = I18n { locale: ctx.locale.get() };
                  i18n.t("settings.theme.hint")
                }}
              </p>
            </div>
          </div>

          <div class="grid grid-cols-2 gap-2 p-4">
            <ThemeOption dark=false />
            <ThemeOption dark=true />
          </div>
        </section>
      </div>
    </div>
  }
}

#[component]
fn CdpPortSetting() -> impl IntoView {
  let ctx = AppCtx::use_ctx();
  let port_input = RwSignal::new("9335".to_string());
  let saving = RwSignal::new(false);

  Effect::new(move |_| {
    spawn_local(async move {
      if let Ok(port) = api::get_cdp_port().await {
        port_input.set(port.to_string());
      }
    });
  });

  let on_save = move |_| {
    if saving.get_untracked() {
      return;
    }
    let locale = ctx.locale.get_untracked();
    let i18n = I18n { locale };
    let parsed = port_input.get_untracked().trim().parse::<u16>();
    let Ok(port) = parsed else {
      toast_error(i18n.t("settings.cdp"), i18n.t("settings.cdp.port.invalid"));
      return;
    };
    if !(1024..=65535).contains(&port) {
      toast_error(i18n.t("settings.cdp"), i18n.t("settings.cdp.port.invalid"));
      return;
    }

    saving.set(true);
    spawn_local(async move {
      match api::set_cdp_port(port).await {
        Ok(saved) => {
          port_input.set(saved.to_string());
          toast(
            i18n.t("settings.cdp.port.saved"),
            &format!("port {saved}"),
            ToastType::Success,
          );
        }
        Err(e) => toast_error(i18n.t("settings.cdp"), &e),
      }
      saving.set(false);
    });
  };

  view! {
    <div class="flex flex-col gap-3 sm:flex-row sm:items-end">
      <label class="min-w-0 flex-1">
        <span class="mb-1.5 block text-xs font-medium text-muted-foreground">
          {move || {
            let i18n = I18n { locale: ctx.locale.get() };
            i18n.t("settings.cdp.port")
          }}
        </span>
        <input
          type="number"
          min="1024"
          max="65535"
          class="h-10 w-full rounded-xl border border-border/70 bg-background/60 px-3 font-mono text-sm text-foreground outline-none transition-colors focus:border-primary/50 focus:ring-2 focus:ring-ring/40"
          prop:value=move || port_input.get()
          on:input=move |ev| port_input.set(event_target_value(&ev))
        />
      </label>
      <button
        type="button"
        class="inline-flex h-10 items-center justify-center gap-2 rounded-xl bg-primary px-4 text-sm font-medium text-primary-foreground shadow-sm transition-all hover:bg-primary/90 active:scale-[0.98] disabled:opacity-60"
        disabled=move || saving.get()
        on:click=on_save
      >
        {move || {
          let i18n = I18n { locale: ctx.locale.get() };
          if saving.get() {
            i18n.t("recommend.applying")
          } else {
            i18n.t("settings.cdp.port.save")
          }
        }}
      </button>
    </div>
  }
}

#[component]
fn LanguageDropdown(open: RwSignal<bool>) -> impl IntoView {
  let ctx = AppCtx::use_ctx();

  view! {
    <div class="relative w-full max-w-md">
      <Show when=move || open.get()>
        <div
          class="fixed inset-0 z-0 cursor-default"
          on:click=move |_| open.set(false)
        />
      </Show>

      <button
        type="button"
        class="group relative z-10 flex w-full items-center justify-between gap-3 rounded-2xl border border-border/70 bg-background/60 px-4 py-3 text-left shadow-sm transition-all hover:border-primary/30 hover:bg-accent/30 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50"
        aria-haspopup="listbox"
        prop:aria-expanded=move || open.get()
        on:click=move |_| open.update(|v| *v = !*v)
      >
        <div class="flex min-w-0 items-center gap-3">
          <span class="flex size-9 shrink-0 items-center justify-center rounded-xl bg-primary/12 text-primary ring-1 ring-primary/15">
            <Languages class="size-4" />
          </span>
          <div class="min-w-0">
            <div class="truncate text-sm font-medium text-foreground">
              {move || ctx.locale.get().label()}
            </div>
            <div class="truncate font-mono text-[11px] text-muted-foreground">
              {move || ctx.locale.get().code()}
            </div>
          </div>
        </div>
        <span class=move || {
          if open.get() {
            "inline-flex shrink-0 text-muted-foreground transition-transform duration-200 rotate-180"
          } else {
            "inline-flex shrink-0 text-muted-foreground transition-transform duration-200"
          }
        }>
          <ChevronDown class="size-4" />
        </span>
      </button>

      <Show when=move || open.get()>
        <ul
          class="absolute left-0 right-0 top-full z-20 mt-2 list-none overflow-hidden rounded-2xl border border-border/70 bg-popover p-1.5 shadow-2xl shadow-black/25 ring-1 ring-border/40"
          role="listbox"
        >
          {Locale::ALL
            .into_iter()
            .map(|loc| {
              view! {
                <li>
                  <button
                    type="button"
                    role="option"
                    prop:aria-selected=move || ctx.locale.get() == loc
                    class=move || {
                      let active = ctx.locale.get() == loc;
                      if active {
                        "flex w-full items-center justify-between gap-3 rounded-xl bg-primary/12 px-3 py-2.5 text-left transition-colors"
                      } else {
                        "flex w-full items-center justify-between gap-3 rounded-xl px-3 py-2.5 text-left transition-colors hover:bg-accent/60"
                      }
                    }
                    on:click=move |_| {
                      ctx.locale.set(loc);
                      open.set(false);
                    }
                  >
                    <div class="min-w-0">
                      <div class="truncate text-sm font-medium text-foreground">{loc.label()}</div>
                      <div class="truncate font-mono text-[11px] text-muted-foreground">{loc.code()}</div>
                    </div>
                    <Show when=move || ctx.locale.get() == loc>
                      <Check class="size-4 shrink-0 text-primary" />
                    </Show>
                  </button>
                </li>
              }
            })
            .collect_view()}
        </ul>
      </Show>
    </div>
  }
}

#[component]
fn ThemeOption(dark: bool) -> impl IntoView {
  let ctx = AppCtx::use_ctx();

  view! {
    <button
      type="button"
      class=move || {
        let active = ctx.is_dark.get() == dark;
        if active {
          "flex flex-col items-center gap-3 rounded-2xl border border-primary/40 bg-primary/10 px-4 py-5 transition-all ring-1 ring-primary/20"
        } else {
          "flex flex-col items-center gap-3 rounded-2xl border border-border/60 bg-background/40 px-4 py-5 transition-all hover:border-border hover:bg-accent/40"
        }
      }
      on:click=move |_| ctx.set_theme(dark)
    >
      <div class=move || {
        let active = ctx.is_dark.get() == dark;
        if active {
          "flex size-12 items-center justify-center rounded-2xl bg-primary text-primary-foreground shadow-md shadow-primary/30"
        } else {
          "flex size-12 items-center justify-center rounded-2xl bg-muted text-muted-foreground"
        }
      }>
        {if dark {
          view! { <Moon class="size-5" /> }.into_any()
        } else {
          view! { <Sun class="size-5" /> }.into_any()
        }}
      </div>
      <span class="text-sm font-medium text-foreground">
        {move || {
          let i18n = I18n { locale: ctx.locale.get() };
          if dark {
            i18n.t("theme.dark")
          } else {
            i18n.t("theme.light")
          }
        }}
      </span>
    </button>
  }
}
