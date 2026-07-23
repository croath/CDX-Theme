use leptos::prelude::*;

use crate::components::layout::{Sidebar, StatusBar};
use crate::components::ui::sonner::{SonnerPosition, SonnerToaster};
use crate::pages::{InstallPage, LibraryPage, RecommendPage, RestorePage, SettingsPage};
use crate::state::AppCtx;
use crate::types::Page;
use crate::window_chrome;

#[component]
pub fn App() -> impl IntoView {
  let ctx = AppCtx::provide();

  view! {
    // Seamless shell: content paints under overlay titlebar (no separate titlebar chrome).
    // Drag via startDragging on non-interactive mousedown (data-tauri-drag-region does not bubble).
    <div
      class="app-shell relative flex h-screen w-screen overflow-hidden bg-background text-foreground"
      on:mousedown=window_chrome::on_window_pointer_down
    >
      // Ambient mesh — full bleed under traffic lights / titlebar
      <div class="pointer-events-none absolute inset-0 overflow-hidden">
        <div class="absolute -left-32 top-0 size-[420px] rounded-full bg-primary/10 blur-3xl dark:bg-primary/15" />
        <div class="absolute bottom-0 right-0 size-[380px] rounded-full bg-chart-2/10 blur-3xl dark:bg-chart-2/15" />
        <div class="absolute left-1/3 top-1/3 size-[280px] rounded-full bg-chart-4/5 blur-3xl" />
      </div>

      <div class="relative z-10 flex min-h-0 min-w-0 flex-1">
        <Sidebar />

        // Right panel: page content + bottom connection status bar
        <main class="relative flex min-h-0 min-w-0 flex-1 flex-col">
          <div class="content-scroll min-h-0 flex-1 overflow-hidden px-5 pb-4 pt-10 sm:px-7 sm:pb-5 sm:pt-11 lg:px-8">
            {move || match ctx.page.get() {
              Page::Recommend => view! { <RecommendPage /> }.into_any(),
              Page::Install => view! { <InstallPage /> }.into_any(),
              Page::Library => view! { <LibraryPage /> }.into_any(),
              Page::Restore => view! { <RestorePage /> }.into_any(),
              Page::Settings => view! { <SettingsPage /> }.into_any(),
            }}
          </div>
          <StatusBar />
        </main>
      </div>

      <SonnerToaster position=SonnerPosition::BottomRight />
    </div>
  }
}
