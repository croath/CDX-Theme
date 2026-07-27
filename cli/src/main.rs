//! `cdxtheme` — theme package tools (thin CLI over `cdx-theme-core`).
//!
//! Examples:
//!   cdxtheme theme pack themes/redbull-racing
//!   cdxtheme theme unpack ferrari-1.0.0.cdxtheme themes/ferrari
//!   cdxtheme apply --app codex --theme ferrari-1.0.0.cdxtheme
//!   cdxtheme verify layout
//!   cdxtheme probe --tab Work
//!   cdxtheme screenshot -o /tmp/work.jpg --tab Work

use cdx_theme_core::{
  DEFAULT_CDP_PORT, InjectOptions, apply_theme, layout_default_options, load_theme_package,
  pack_theme_dir, probe, screenshot, unpack_package, verify_layout, verify_theme,
};
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(
  name = "cdxtheme",
  version,
  about = "CDXTheme CLI",
  long_about = "Pack, unpack, apply, and verify multi-app theme packages (.cdxtheme)."
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
    /// Host app id (currently only `codex`).
    #[arg(long, default_value = "codex")]
    app: String,

    /// Path to `.cdxtheme` package.
    #[arg(long, short = 't')]
    theme: PathBuf,

    /// CDP remote-debugging port (default 9335).
    #[arg(long, default_value_t = DEFAULT_CDP_PORT)]
    port: u16,

    /// Timeout for CDP wait / inject (milliseconds).
    #[arg(long, default_value_t = 120_000)]
    timeout_ms: u64,
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
  },

  /// Unpack a portable package into a source theme directory.
  Unpack {
    /// Package file (`.cdxtheme`).
    input: PathBuf,

    /// Destination theme directory (theme.json + per-target css + images).
    output: PathBuf,
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
      } => {
        let (path, bytes) = pack_theme_dir(&source, output.as_deref(), pretty, force)?;
        println!(
          "packed {} → {} ({} bytes)",
          source.display(),
          path.display(),
          bytes
        );
        Ok(ExitCode::SUCCESS)
      }

      ThemeCommands::Unpack { input, output } => {
        let dir = unpack_package(&input, &output)?;
        println!("unpacked {} → {}", input.display(), dir.display());
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
      let result = rt.block_on(apply_theme(&app, &theme, Some(port), timeout_ms))?;
      println!(
        "applied theme `{}` to {app} on port {} ({} target(s))",
        result.theme_id,
        result.port,
        result.targets.len()
      );
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
            println!(
              "  [{status}] {} {} → {}",
              t.target_id,
              t.url,
              t.result
            );
          }
          if any_fail {
            return Ok(ExitCode::from(1));
          }
        }
        // if json mode still check fail
        let any_fail = result.targets.iter().any(|t| {
          !t
            .result
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
          let context = data
            .get("context")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
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
              data.get("wrapPos").map(|v| v.to_string()).unwrap_or_else(|| "null".into()),
              data.get("utilBox").map(|v| v.to_string()).unwrap_or_else(|| "null".into()),
              data.get("hero").map(|v| v.to_string()).unwrap_or_else(|| "null".into()),
              data.get("missions").map(|v| v.to_string()).unwrap_or_else(|| "null".into()),
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
