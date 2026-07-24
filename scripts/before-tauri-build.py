#!/usr/bin/env python3
"""Tauri beforeBuildCommand / beforeDevCommand helper (cross-platform).

Stages the cdxtheme CLI sidecar, then runs Trunk (build or serve).

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


def prepare_cli() -> None:
  if os.name == "nt":
    ps1 = ROOT / "scripts" / "prepare-cli-sidecar.ps1"
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
    return

  sh = ROOT / "scripts" / "prepare-cli-sidecar.sh"
  run(["bash", str(sh)])


def trunk_bin() -> str:
  found = shutil.which("trunk")
  if found:
    return found
  raise SystemExit("error: trunk not found on PATH — install with: cargo install trunk")


def main() -> int:
  dev = "--dev" in sys.argv[1:]
  prepare_cli()
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
