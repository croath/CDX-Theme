use icons::{Library, PackagePlus, RotateCcw, Settings, Sparkles};
use leptos::prelude::*;

use crate::i18n::I18n;
use crate::state::AppCtx;
use crate::types::Page;

#[component]
pub fn Sidebar() -> impl IntoView {
  let ctx = AppCtx::use_ctx();

  view! {
    <aside class="sidebar relative flex h-full w-[260px] shrink-0 flex-col border-r border-border/40 bg-sidebar/45 backdrop-blur-2xl">
      // Ambient glow (shows through transparent bottom status + overlay titlebar)
      <div class="pointer-events-none absolute inset-0 overflow-hidden">
        <div class="absolute -left-16 -top-16 size-40 rounded-full bg-primary/20 blur-3xl" />
        <div class="absolute -bottom-10 left-10 size-32 rounded-full bg-chart-2/20 blur-3xl" />
      </div>

      // Brand — same artwork as the app icon (public/logo.png), larger/fuller mark
      <div class="relative z-10 flex items-center gap-3.5 px-5 pb-4 pt-12">
        <div class="relative flex size-12 shrink-0 items-center justify-center">
          // Transparent macOS-style app mark (same asset as Dock icon)
          <img
            src="public/logo.png"
            alt="CDXTheme"
            class="size-12 object-contain drop-shadow-md"
            draggable="false"
          />
        </div>
        <div class="min-w-0">
          <div class="truncate text-[15px] font-semibold tracking-tight text-foreground">
            {move || {
              let i18n = I18n { locale: ctx.locale.get() };
              i18n.t("app.name")
            }}
          </div>
          <div class="truncate text-[11px] text-muted-foreground">
            {move || {
              let i18n = I18n { locale: ctx.locale.get() };
              i18n.t("app.tagline")
            }}
          </div>
        </div>
      </div>

      <div class="relative z-10 mx-4 mb-3 h-px bg-gradient-to-r from-transparent via-border to-transparent" />

      // Navigation only (theme switch is in Settings; status bar is on main panel)
      <nav class="relative z-10 flex flex-1 flex-col gap-1 px-3 pb-4">
        <NavItem page=Page::Recommend icon=NavIcon::Recommend label_key="nav.recommend" />
        <NavItem page=Page::Library icon=NavIcon::Library label_key="nav.library" />
        <NavItem page=Page::Install icon=NavIcon::Install label_key="nav.install" />
        <NavItem page=Page::Restore icon=NavIcon::Restore label_key="nav.restore" />
        <NavItem page=Page::Settings icon=NavIcon::Settings label_key="nav.settings" />
      </nav>
    </aside>
  }
}

#[derive(Clone, Copy)]
enum NavIcon {
  Recommend,
  Library,
  Install,
  Restore,
  Settings,
}

#[component]
fn NavItem(page: Page, icon: NavIcon, label_key: &'static str) -> impl IntoView {
  let ctx = AppCtx::use_ctx();

  let on_click = move |_| ctx.page.set(page);

  view! {
    <button
      type="button"
      class=move || {
        let active = ctx.page.get() == page;
        if active {
          "group flex w-full items-center gap-3 rounded-xl bg-primary/12 px-3 py-2.5 text-left text-sm font-medium text-foreground shadow-sm ring-1 ring-primary/20 transition-all"
        } else {
          "group flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-left text-sm font-medium text-muted-foreground transition-all hover:bg-accent/70 hover:text-foreground"
        }
      }
      on:click=on_click
    >
      <span class=move || {
        let active = ctx.page.get() == page;
        if active {
          "flex size-8 items-center justify-center rounded-lg bg-primary text-primary-foreground shadow-md shadow-primary/25"
        } else {
          "flex size-8 items-center justify-center rounded-lg bg-muted/80 text-muted-foreground transition-colors group-hover:bg-accent group-hover:text-foreground"
        }
      }>
        {match icon {
          NavIcon::Recommend => view! { <Sparkles class="size-4" /> }.into_any(),
          NavIcon::Library => view! { <Library class="size-4" /> }.into_any(),
          NavIcon::Install => view! { <PackagePlus class="size-4" /> }.into_any(),
          NavIcon::Restore => view! { <RotateCcw class="size-4" /> }.into_any(),
          NavIcon::Settings => view! { <Settings class="size-4" /> }.into_any(),
        }}
      </span>
      <span class="truncate">
        {move || {
          let i18n = I18n { locale: ctx.locale.get() };
          i18n.t(label_key)
        }}
      </span>
      <Show when=move || ctx.page.get() == page>
        <span class="ml-auto size-1.5 rounded-full bg-primary shadow-[0_0_8px] shadow-primary" />
      </Show>
    </button>
  }
}
