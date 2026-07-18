//! `cdxtheme` — theme package tools (thin CLI over `cdx-theme-core`).
//!
//! Examples:
//!   cdxtheme theme pack themes/redbull-racing
//!   cdxtheme theme pack themes/ferrari --format codedrobe-theme
//!   cdxtheme theme unpack ferrari-1.0.0.cdxtheme themes/ferrari
//!   cdxtheme theme convert ferrari-1.0.0.codedrobe-theme
//!   cdxtheme apply --app codex --theme ferrari-1.0.0.cdxtheme

use cdx_theme_core::{
  DEFAULT_CDP_PORT, PackageFormat, apply_theme, convert_package, pack_theme_dir, unpack_package,
};
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
  name = "cdxtheme",
  version,
  about = "CDXTheme CLI",
  long_about = "Pack, unpack, convert, and apply multi-app theme packages (.cdxtheme / .codedrobe-theme)."
)]
struct Cli {
  #[command(subcommand)]
  command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
  /// Theme package commands (pack / unpack / convert).
  Theme {
    #[command(subcommand)]
    command: ThemeCommands,
  },

  /// Apply a theme package to a host app (ensure CDP, then inject).
  Apply {
    /// Host app id (currently only `codex`).
    #[arg(long, default_value = "codex")]
    app: String,

    /// Path to `.cdxtheme` / `.codedrobe-theme` package.
    #[arg(long, short = 't')]
    theme: PathBuf,

    /// CDP remote-debugging port (default 9335).
    #[arg(long, default_value_t = DEFAULT_CDP_PORT)]
    port: u16,

    /// Timeout for CDP wait / inject (milliseconds).
    #[arg(long, default_value_t = 120_000)]
    timeout_ms: u64,
  },
}

/// Clap-facing package format (maps to `cdx_theme_core::PackageFormat`).
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
enum FormatArg {
  /// CDXTheme brand (default): `.cdxtheme`
  #[default]
  Cdxtheme,
  /// CodeDrobe brand: `.codedrobe-theme`
  #[value(name = "codedrobe-theme")]
  CodedrobeTheme,
}

impl From<FormatArg> for PackageFormat {
  fn from(value: FormatArg) -> Self {
    match value {
      FormatArg::Cdxtheme => PackageFormat::Cdxtheme,
      FormatArg::CodedrobeTheme => PackageFormat::CodedrobeTheme,
    }
  }
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

    /// Package brand: `cdxtheme` (default) or `codedrobe-theme` (same schema).
    #[arg(long, value_enum, default_value_t = FormatArg::Cdxtheme)]
    format: FormatArg,

    /// Pretty-print JSON (default: compact).
    #[arg(long)]
    pretty: bool,

    /// Overwrite existing output file.
    #[arg(long)]
    force: bool,
  },

  /// Unpack a portable package into a source theme directory.
  Unpack {
    /// Package file (`.cdxtheme` or `.codedrobe-theme`).
    input: PathBuf,

    /// Destination theme directory (theme.json + per-target css + images).
    output: PathBuf,
  },

  /// Convert a `codedrobe-theme` package to `.cdxtheme` (rewrites CSS brand tokens).
  Convert {
    /// Package file (`.codedrobe-theme` or `.cdxtheme`).
    input: PathBuf,

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
}

fn main() {
  init_tracing();
  if let Err(e) = run() {
    eprintln!("error: {e}");
    std::process::exit(1);
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

fn run() -> Result<(), Box<dyn std::error::Error>> {
  let cli = Cli::parse();
  match cli.command {
    Commands::Theme { command } => match command {
      ThemeCommands::Pack {
        source,
        output,
        format,
        pretty,
        force,
      } => {
        let (path, bytes) =
          pack_theme_dir(&source, output.as_deref(), format.into(), pretty, force)?;
        println!(
          "packed {} → {} ({} bytes)",
          source.display(),
          path.display(),
          bytes
        );
      }

      ThemeCommands::Unpack { input, output } => {
        let dir = unpack_package(&input, &output)?;
        println!("unpacked {} → {}", input.display(), dir.display());
      }

      ThemeCommands::Convert {
        input,
        output,
        pretty,
        force,
      } => {
        let (path, bytes) = convert_package(&input, output.as_deref(), pretty, force)?;
        println!(
          "converted {} → {} ({} bytes)",
          input.display(),
          path.display(),
          bytes
        );
      }
    },

    Commands::Apply {
      app,
      theme,
      port,
      timeout_ms,
    } => {
      let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
      let result = rt.block_on(apply_theme(&app, &theme, Some(port), timeout_ms))?;
      println!(
        "applied theme `{}` to {app} on port {} ({} target(s))",
        result.theme_id,
        result.port,
        result.targets.len()
      );
    }
  }
  Ok(())
}
