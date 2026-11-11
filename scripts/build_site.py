"""Build a whitelisted static Pages artifact without copying repository contents."""

import argparse
import hashlib
import json
import re
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ASSETS = (
    "examples/index.html",
    "examples/tools.js",
    "dist/index.js",
    "dist/errors.js",
    "dist/wav.js",
    "dist/wasm/wasm_audio_engine.js",
    "dist/wasm/wasm_audio_engine_bg.wasm",
    "LICENSE",
    "THIRD_PARTY_NOTICES.txt",
)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "_site")
    parser.add_argument("--revision", required=True)
    args = parser.parse_args()
    if not re.fullmatch(r"[0-9a-f]{40}", args.revision):
        parser.error("revision must be a full Git commit SHA")
    for name in ASSETS:
        source = ROOT / name
        if source.is_symlink() or not source.is_file():
            raise RuntimeError("Missing or linked site asset: " + name)
    # Exclusive creation avoids merging stale files or replacing user data.
    args.output.mkdir(parents=True, exist_ok=False)
    for name in ASSETS:
        destination = args.output / name
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(ROOT / name, destination)
    (args.output / "index.html").write_text(
        '<!doctype html>\n<html lang="en"><meta charset="utf-8">'
        '<meta name="viewport" content="width=device-width, initial-scale=1">'
        '<meta http-equiv="refresh" content="0;url=./examples/">'
        '<title>Audio Engine Lab</title><a href="./examples/">Open Audio Engine Lab</a></html>\n',
        encoding="utf-8",
    )
    (args.output / ".nojekyll").touch()
    hashes = {
        path.relative_to(args.output).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
        for path in sorted(args.output.rglob("*"))
        if path.is_file()
    }
    (args.output / "build.json").write_text(
        json.dumps({"revision": args.revision, "assets": hashes}, indent=2) + "\n",
        encoding="utf-8",
    )
    print("Built static site:", args.output)


if __name__ == "__main__":
    main()
