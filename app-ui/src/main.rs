mod api;
mod app;
mod components;
mod i18n;
mod pages;
mod posthog;
mod state;
mod types;
mod window_chrome;

use app::App;
use leptos::prelude::*;

fn main() {
  console_error_panic_hook::set_once();
  mount_to_body(|| {
    view! {
      <App />
    }
  });
}
