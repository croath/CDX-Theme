//! `cdxtheme` — theme package tools (thin CLI over `cdx-theme-core`).
//!
//! Examples:
//!   cdxtheme theme pack themes/redbull-racing
//!   cdxtheme theme unpack ferrari-1.0.0.cdxtheme themes/ferrari
//!   cdxtheme theme merge-css themes/my-theme
//!   cdxtheme apply --app codex --theme ferrari-1.0.0.cdxtheme
//!   cdxtheme apply --app workbuddy --theme ferrari-1.0.0.cdxtheme
//!   cdxtheme restore
//!   cdxtheme restore --app workbuddy
//!   cdxtheme detect
//!   cdxtheme appearance dark
//!   cdxtheme verify layout
//!   cdxtheme probe --tab Work
//!   cdxtheme screenshot -o /tmp/work.jpg --tab Work

use cdx_theme_core::{
  AppearanceTheme, DEFAULT_CDP_PORT, InjectOptions, apply_theme, default_cdp_port_for_app,
  detect_hosts, layout_default_options, load_theme_package, merge_theme_css,
  pack_theme_dir_with_options, probe, restart_codex_debugging, restore_theme, screenshot,
  set_appearance_theme, unpack_package, verify_layout, verify_theme,
};
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(
  name = "cdxtheme",
  version,
  about = "CDXTheme CLI",
  long_about = "Pack, unpack, apply, restore, and verify multi-app theme packages (.cdxtheme)."
)]
struct Cli {
  #[command(subcommand)]
  command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
  /// Theme package commands (pack / unpack).
  Theme {
    #[command(subcommand)]
    command: ThemeCommands,
  },

  /// Apply a theme package to a host app (ensure CDP, then inject).
  Apply {
    /// Host app id: `codex` (default CDP 9335) or `workbuddy` (default CDP 9336).
    #[arg(long, default_value = "codex")]
    app: String,

    /// Path to `.cdxtheme` package.
    #[arg(long, short = 't')]
    theme: PathBuf,

    /// CDP remote-debugging port (default: 9335 codex / 9336 workbuddy).
    #[arg(long)]
    port: Option<u16>,

    /// Timeout for CDP wait / inject (milliseconds).
    #[arg(long, default_value_t = 120_000)]
    timeout_ms: u64,
  },

  /// Restore default skin (ensure CDP, then remove injected theme DOM/CSS).
  Restore {
    /// Host app id: `codex` (default) or `workbuddy`.
    #[arg(long, default_value = "codex")]
    app: String,

    /// CDP remote-debugging port (default: 9335 codex / 9336 workbuddy).
    #[arg(long)]
    port: Option<u16>,

    /// Timeout for CDP wait / restore (milliseconds).
    #[arg(long, default_value_t = 120_000)]
    timeout_ms: u64,
  },

  /// Detect Codex / WorkBuddy install, process, and default CDP status.
  Detect {
    /// Print full report as JSON.
    #[arg(long)]
    json: bool,
  },

  /// Adjust ChatGPT / Codex appearance mode (`dark` / `light` / `system`).
  ///
  /// Writes `[desktop].appearanceTheme` in `~/.codex/config.toml`.
  /// Restarts Codex when the value changes (unless `--no-restart`).
  Appearance {
    /// Appearance mode: dark, light, or system.
    #[arg(value_enum)]
    mode: AppearanceMode,

    /// Codex config path (default `~/.codex/config.toml`).
    #[arg(long)]
    config: Option<PathBuf>,

    /// CDP remote-debugging port used when restarting Codex (default 9335).
    #[arg(long, default_value_t = DEFAULT_CDP_PORT)]
    port: u16,

    /// Only write config; do not restart Codex after a mode change.
    #[arg(long)]
    no_restart: bool,
  },

  /// Verify injected theme and/or Chat/Work layout over CDP.
  Verify {
    #[command(subcommand)]
    command: VerifyCommands,
  },

  /// Quick DOM snapshot or custom JS evaluate over CDP.
  Probe {
    /// CDP remote-debugging port (default 9335).
    #[arg(long, default_value_t = DEFAULT_CDP_PORT)]
    port: u16,

    /// Timeout for CDP (milliseconds).
    #[arg(long, default_value_t = 30_000)]
    timeout_ms: u64,

    /// Switch Chat/Work tab before probing.
    #[arg(long, value_enum)]
    tab: Option<TabLabel>,

    /// Wait after tab switch (milliseconds).
    #[arg(long, default_value_t = 900)]
    wait_ms: u64,

    /// Raw JS expression (default: theme DOM snapshot).
    #[arg(long)]
    expr: Option<String>,
  },

  /// Capture a JPEG screenshot of live Codex via CDP.
  Screenshot {
    /// Output JPEG path.
    #[arg(short = 'o', long)]
    output: PathBuf,

    /// CDP remote-debugging port (default 9335).
    #[arg(long, default_value_t = DEFAULT_CDP_PORT)]
    port: u16,

    /// Timeout for CDP (milliseconds).
    #[arg(long, default_value_t = 30_000)]
    timeout_ms: u64,

    /// Switch Chat/Work tab before capture.
    #[arg(long, value_enum)]
    tab: Option<TabLabel>,

    /// Wait after tab switch (milliseconds).
    #[arg(long, default_value_t = 900)]
    wait_ms: u64,

    /// JPEG quality 1–100.
    #[arg(long, default_value_t = 70)]
    quality: u8,
  },
}

#[derive(Subcommand, Debug)]
enum ThemeCommands {
  /// Pack a source theme (directory or theme.json / manifest.json) into a portable package.
  ///
  /// By default merges `codex/*.css` / `workbuddy/*.css` **in memory** into the package.
  /// Does not write root `codex.css` / `workbuddy.css` (use `theme merge-css` for that).
  Pack {
    /// Theme directory (`theme.json` preferred, else `manifest.json`) or path to that file.
    source: PathBuf,

    /// Output file path. Defaults to `{id}-{version}.cdxtheme`.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Pretty-print JSON (default: compact).
    #[arg(long)]
    pretty: bool,

    /// Overwrite existing output file.
    #[arg(long)]
    force: bool,

    /// Skip in-memory merge of CSS partials into the package.
    /// Default merges `codex/` / `workbuddy/` partials when present (without writing root CSS files).
    #[arg(long)]
    no_merge_css: bool,
  },

  /// Unpack a portable package into a source theme directory.
  Unpack {
    /// Package file (`.cdxtheme`).
    input: PathBuf,

    /// Destination theme directory (theme.json + per-target css + images).
    output: PathBuf,
  },

  /// Write merged CSS partials to root files (`codex/*.css` → `codex.css`).
  ///
  /// Partials are concatenated in alphabetical order (use numeric prefixes like
  /// `00-tokens.css`, `01-shell.css`).
  ///
  /// `theme pack` embeds partials in memory by default and does **not** write these files.
  #[command(name = "merge-css")]
  MergeCss {
    /// Theme source directory (contains `codex/` and/or `workbuddy/` partial dirs).
    source: PathBuf,

    /// Only merge this target (`codex` or `workbuddy`). Default: every present partial dir.
    #[arg(long, short = 't')]
    target: Option<String>,
  },
}

#[derive(Subcommand, Debug)]
enum VerifyCommands {
  /// Verify inject state (theme id/version/style/host classes).
  Inject {
    /// Optional package path to load expected theme id/version.
    #[arg(long, short = 't')]
    theme: Option<PathBuf>,

    /// CDP remote-debugging port (default 9335).
    #[arg(long, default_value_t = DEFAULT_CDP_PORT)]
    port: u16,

    /// Timeout for CDP (milliseconds).
    #[arg(long, default_value_t = 30_000)]
    timeout_ms: u64,

    /// Print raw JSON results.
    #[arg(long)]
    json: bool,
  },

  /// Verify Chat/Work layout contracts (composer, util stack, ribbon, hero).
  Layout {
    /// Contexts to check: chat, work (comma-separated).
    #[arg(long, default_value = "chat,work")]
    contexts: String,

    /// CDP remote-debugging port (default 9335).
    #[arg(long, default_value_t = DEFAULT_CDP_PORT)]
    port: u16,

    /// Timeout for CDP (milliseconds).
    #[arg(long, default_value_t = 30_000)]
    timeout_ms: u64,

    /// Wait after each tab switch (milliseconds).
    #[arg(long, default_value_t = 1100)]
    wait_ms: u64,

    /// Write full JSON report to this path.
    #[arg(long)]
    json_out: Option<PathBuf>,

    /// Also print full JSON to stdout.
    #[arg(long)]
    json: bool,
  },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum TabLabel {
  Chat,
  Work,
}

impl TabLabel {
  fn as_str(self) -> &'static str {
    match self {
      TabLabel::Chat => "Chat",
      TabLabel::Work => "Work",
    }
  }
}

/// ChatGPT / Codex `[desktop].appearanceTheme` values.
#[derive(Clone, Copy, Debug, ValueEnum)]
enum AppearanceMode {
  Dark,
  Light,
  System,
}

impl AppearanceMode {
  fn to_core(self) -> AppearanceTheme {
    match self {
      AppearanceMode::Dark => AppearanceTheme::Dark,
      AppearanceMode::Light => AppearanceTheme::Light,
      AppearanceMode::System => AppearanceTheme::System,
    }
  }
}

fn main() -> ExitCode {
  init_tracing();
  match run() {
    Ok(code) => code,
    Err(e) => {
      eprintln!("error: {e}");
      ExitCode::from(2)
    }
  }
}

fn init_tracing() {
  use tracing_subscriber::EnvFilter;
  let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
  let _ = tracing_subscriber::fmt()
    .with_env_filter(filter)
    .with_writer(std::io::stderr)
    .with_target(false)
    .try_init();
}

fn runtime() -> Result<tokio::runtime::Runtime, Box<dyn std::error::Error>> {
  Ok(
    tokio::runtime::Builder::new_multi_thread()
      .enable_all()
      .build()?,
  )
}

fn run() -> Result<ExitCode, Box<dyn std::error::Error>> {
  let cli = Cli::parse();
  match cli.command {
    Commands::Theme { command } => match command {
      ThemeCommands::Pack {
        source,
        output,
        pretty,
        force,
        no_merge_css,
      } => {
        let result =
          pack_theme_dir_with_options(&source, output.as_deref(), pretty, force, !no_merge_css)?;
        for m in &result.merges {
          println!(
            "merged {} ({} part{}) → package:{} ({} bytes, in-memory)",
            m.partials_dir.display(),
            m.parts.len(),
            if m.parts.len() == 1 { "" } else { "s" },
            m.target,
            m.bytes
          );
        }
        println!(
          "packed {} → {} ({} bytes)",
          source.display(),
          result.path.display(),
          result.bytes
        );
        Ok(ExitCode::SUCCESS)
      }

      ThemeCommands::Unpack { input, output } => {
        let dir = unpack_package(&input, &output)?;
        println!("unpacked {} → {}", input.display(), dir.display());
        Ok(ExitCode::SUCCESS)
      }

      ThemeCommands::MergeCss { source, target } => {
        let results = merge_theme_css(&source, target.as_deref())?;
        for r in &results {
          println!(
            "merged {} ({} part{}) → {} ({} bytes)",
            r.partials_dir.display(),
            r.parts.len(),
            if r.parts.len() == 1 { "" } else { "s" },
            r.output.display(),
            r.bytes
          );
          for p in &r.parts {
            println!("  • {p}");
          }
        }
        Ok(ExitCode::SUCCESS)
      }
    },

    Commands::Apply {
      app,
      theme,
      port,
      timeout_ms,
    } => {
      let rt = runtime()?;
      let port = port.or_else(|| Some(default_cdp_port_for_app(&app)));
      let result = rt.block_on(apply_theme(&app, &theme, port, timeout_ms))?;
      println!(
        "applied theme `{}` to {app} on port {} ({} target(s))",
        result.theme_id,
        result.port,
        result.targets.len()
      );
      Ok(ExitCode::SUCCESS)
    }

    Commands::Restore {
      app,
      port,
      timeout_ms,
    } => {
      let rt = runtime()?;
      let port = port.or_else(|| Some(default_cdp_port_for_app(&app)));
      let result = rt.block_on(restore_theme(Some(&app), port, timeout_ms))?;
      println!(
        "restored default skin for {app} on port {} ({} target(s))",
        result.port,
        result.targets.len()
      );
      Ok(ExitCode::SUCCESS)
    }

    Commands::Detect { json } => {
      let rt = runtime()?;
      let report = rt.block_on(detect_hosts());
      if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
      } else {
        println!("Host app detection");
        println!();
        for host in &report.hosts {
          let status = if host.installed {
            "installed"
          } else {
            "not found"
          };
          println!("{} ({})", host.display_name, host.app_id);
          println!("  status:     {status}");
          if let Some(path) = &host.path {
            println!("  path:       {}", path.display());
          }
          if let Some(exe) = &host.executable {
            if host.path.as_ref().map(|p| p.as_path()) != Some(exe.as_path()) {
              println!("  executable: {}", exe.display());
            }
          }
          println!("  running:    {}", if host.running { "yes" } else { "no" });
          println!(
            "  cdp:        {} (port {}, {} target(s))",
            if host.cdp_reachable {
              "reachable"
            } else {
              "not reachable"
            },
            host.default_cdp_port,
            host.cdp_targets
          );
          println!();
        }
        let installed = report.hosts.iter().filter(|h| h.installed).count();
        println!(
          "summary: {installed}/{} host(s) installed",
          report.hosts.len()
        );
      }
      Ok(ExitCode::SUCCESS)
    }

    Commands::Appearance {
      mode,
      config,
      port,
      no_restart,
    } => {
      let result = set_appearance_theme(mode.to_core(), config.as_deref())?;
      let prev = result.previous.as_deref().unwrap_or("(unset)");
      if result.changed {
        println!(
          "appearanceTheme: {prev} → {} ({})",
          result.mode,
          result.config.display()
        );
        if no_restart {
          println!("config updated; restart Codex/ChatGPT for the mode to take effect");
        } else {
          let rt = runtime()?;
          let msg = rt.block_on(restart_codex_debugging(port))?;
          println!("{msg}");
        }
      } else {
        println!(
          "appearanceTheme already `{}` ({})",
          result.mode,
          result.config.display()
        );
      }
      Ok(ExitCode::SUCCESS)
    }

    Commands::Verify { command } => match command {
      VerifyCommands::Inject {
        theme,
        port,
        timeout_ms,
        json,
      } => {
        let rt = runtime()?;
        let expected = if let Some(path) = theme.as_ref() {
          let loaded = load_theme_package(path).map_err(|e| e.to_string())?;
          Some(loaded.public())
        } else {
          None
        };
        // PublicTheme is inside loaded - need to check API
        // load_theme_package returns LoadedTheme; use verify with theme id via expression path
        let opts = InjectOptions { port, timeout_ms };
        let expected_ref = expected.as_ref();
        let result = rt.block_on(verify_theme(expected_ref, opts))?;
        if json {
          println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
          println!(
            "verify inject on port {} ({} target(s))",
            result.port,
            result.targets.len()
          );
          let mut any_fail = false;
          for t in &result.targets {
            let pass = t
              .result
              .get("pass")
              .and_then(|v| v.as_bool())
              .unwrap_or(false);
            if !pass {
              any_fail = true;
            }
            let status = if pass { "PASS" } else { "FAIL" };
            println!("  [{status}] {} {} → {}", t.target_id, t.url, t.result);
          }
          if any_fail {
            return Ok(ExitCode::from(1));
          }
        }
        // if json mode still check fail
        let any_fail = result.targets.iter().any(|t| {
          !t.result
            .get("pass")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        });
        Ok(if any_fail {
          ExitCode::from(1)
        } else {
          ExitCode::SUCCESS
        })
      }

      VerifyCommands::Layout {
        contexts,
        port,
        timeout_ms,
        wait_ms,
        json_out,
        json,
      } => {
        let rt = runtime()?;
        let opts = layout_default_options(Some(port), Some(timeout_ms));
        let ctxs: Vec<&str> = contexts
          .split(',')
          .map(|s| s.trim())
          .filter(|s| !s.is_empty())
          .collect();
        let report = rt.block_on(verify_layout(opts, &ctxs, wait_ms))?;

        // human summary
        println!("=== theme layout verify ===");
        for (name, data) in &report.contexts {
          let issues = data
            .get("issues")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
          let status = if issues.is_empty() { "PASS" } else { "FAIL" };
          let context = data.get("context").and_then(|v| v.as_str()).unwrap_or("?");
          println!("\n[{status}] {name} ({context})");
          if let Some(brand) = data.get("brand") {
            println!("  brand={brand}");
          }
          if let Some(composer) = data.get("composer") {
            println!("  composer={composer}");
          }
          if name == "work" || data.get("isWork").and_then(|v| v.as_bool()) == Some(true) {
            println!(
              "  wrapPos={} util={} hero={} missions={} gapHeroMissions={} gapMissionsUtil={}",
              data
                .get("wrapPos")
                .map(|v| v.to_string())
                .unwrap_or_else(|| "null".into()),
              data
                .get("utilBox")
                .map(|v| v.to_string())
                .unwrap_or_else(|| "null".into()),
              data
                .get("hero")
                .map(|v| v.to_string())
                .unwrap_or_else(|| "null".into()),
              data
                .get("missions")
                .map(|v| v.to_string())
                .unwrap_or_else(|| "null".into()),
              data
                .get("gapHeroMissions")
                .map(|v| v.to_string())
                .unwrap_or_else(|| "null".into()),
              data
                .get("gapMissionsUtil")
                .map(|v| v.to_string())
                .unwrap_or_else(|| "null".into()),
            );
            if let Some(before) = data.get("composerBefore") {
              println!("  composerBefore={before}");
            }
          }
          for issue in &issues {
            if let Some(s) = issue.as_str() {
              println!("  - {s}");
            }
          }
        }
        println!();
        if report.ok {
          println!("RESULT: PASS");
        } else {
          println!("RESULT: FAIL ({} issue(s))", report.issue_count);
          for i in &report.issues {
            println!("  • {i}");
          }
        }

        if let Some(path) = json_out {
          if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
          }
          std::fs::write(&path, serde_json::to_string_pretty(&report)? + "\n")?;
          println!("wrote {}", path.display());
        }
        if json {
          println!("{}", serde_json::to_string_pretty(&report)?);
        }

        Ok(if report.ok {
          ExitCode::SUCCESS
        } else {
          ExitCode::from(1)
        })
      }
    },

    Commands::Probe {
      port,
      timeout_ms,
      tab,
      wait_ms,
      expr,
    } => {
      let rt = runtime()?;
      let opts = layout_default_options(Some(port), Some(timeout_ms));
      let tab_s = tab.map(|t| t.as_str());
      let value = rt.block_on(probe(opts, tab_s, expr.as_deref(), wait_ms))?;
      println!("{}", serde_json::to_string_pretty(&value)?);
      Ok(ExitCode::SUCCESS)
    }

    Commands::Screenshot {
      output,
      port,
      timeout_ms,
      tab,
      wait_ms,
      quality,
    } => {
      let rt = runtime()?;
      let opts = layout_default_options(Some(port), Some(timeout_ms));
      let tab_s = tab.map(|t| t.as_str());
      rt.block_on(screenshot(opts, &output, tab_s, quality, wait_ms))?;
      println!("wrote {}", output.display());
      Ok(ExitCode::SUCCESS)
    }
  }
}
