#!/usr/bin/env python3
"""Tauri beforeBuildCommand / beforeDevCommand helper (cross-platform).

Stages the cdxtheme CLI + Bun sidecars, then runs Trunk (build or serve).

Usage:
  python3 scripts/before-tauri-build.py           # trunk build
  python3 scripts/before-tauri-build.py --dev     # trunk serve
"""

from __future__ import annotations

import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def run(cmd: list[str], *, cwd: Path | None = None) -> None:
  print(f"==> {' '.join(cmd)}", flush=True)
  subprocess.check_call(cmd, cwd=str(cwd or ROOT))


def run_ps1(name: str) -> None:
  ps1 = ROOT / "scripts" / name
  run(
    [
      "powershell",
      "-NoProfile",
      "-ExecutionPolicy",
      "Bypass",
      "-File",
      str(ps1),
    ]
  )


def run_sh(name: str) -> None:
  sh = ROOT / "scripts" / name
  run(["bash", str(sh)])


def prepare_cli() -> None:
  if os.name == "nt":
    run_ps1("prepare-cli-sidecar.ps1")
    return
  run_sh("prepare-cli-sidecar.sh")


def prepare_bun() -> None:
  """Download Bun into app-tauri/binaries if missing (for bundle.externalBin)."""
  if os.name == "nt":
    run_ps1("prepare-bun-sidecar.ps1")
    return
  run_sh("prepare-bun-sidecar.sh")


def trunk_bin() -> str:
  found = shutil.which("trunk")
  if found:
    return found
  raise SystemExit("error: trunk not found on PATH — install with: cargo install trunk")


def main() -> int:
  dev = "--dev" in sys.argv[1:]
  prepare_cli()
  prepare_bun()
  ui = ROOT / "app-ui"
  if dev:
    # Replace process so signals go to trunk serve.
    os.chdir(ui)
    os.execvp(trunk_bin(), [trunk_bin(), "serve"])
  run([trunk_bin(), "build"], cwd=ui)
  return 0


if __name__ == "__main__":
  try:
    raise SystemExit(main())
  except subprocess.CalledProcessError as e:
    raise SystemExit(e.returncode) from e
