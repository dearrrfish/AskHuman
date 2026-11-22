"""Reject missing or unsmudged assets before compiling Tauri."""

import json
from pathlib import Path

root = Path("src-tauri")
config = json.loads((root / "tauri.conf.json").read_text())
required = {root / icon for icon in config["bundle"]["icon"]}
required.add(root / "icons/icon.png")
required.update((root / "icons").rglob("*.png"))
missing = []
for path in sorted(required):
    if not path.is_file():
        missing.append(str(path))
    else:
        with path.open("rb") as asset:
            if asset.read(128).startswith(b"version https://git-lfs.github.com/spec/v1"):
                missing.append(str(path))
if missing:
    raise SystemExit(
        "Missing Git LFS assets: "
        + ", ".join(missing)
        + "\nFetch with git+https://github.com/Naituw/AskHuman.git?ref=<ref>&lfs=1"
        + " or git+file:///absolute/worktree/path?lfs=1."
    )
