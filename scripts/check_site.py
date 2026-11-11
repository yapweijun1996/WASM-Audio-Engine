"""Check deployment identity, asset integrity, and the static artifact allowlist."""

import argparse
import hashlib
import json
from pathlib import Path

from build_site import ASSETS


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("site", type=Path)
    parser.add_argument("--revision", required=True)
    args = parser.parse_args()
    manifest = json.loads((args.site / "build.json").read_text(encoding="utf-8"))
    if manifest["revision"] != args.revision:
        raise RuntimeError("Deployment revision does not match")
    expected = set(ASSETS) | {"index.html", ".nojekyll"}
    actual = {
        path.relative_to(args.site).as_posix()
        for path in args.site.rglob("*") if path.is_file()
    }
    if actual != expected | {"build.json"} or set(manifest["assets"]) != expected:
        raise RuntimeError("Unexpected or missing static artifact files")
    for name, digest in manifest["assets"].items():
        path = args.site / name
        if path.is_symlink() or hashlib.sha256(path.read_bytes()).hexdigest() != digest:
            raise RuntimeError("Static asset integrity check failed: " + name)
    if (args.site / "dist/wasm/wasm_audio_engine_bg.wasm").read_bytes()[:4] != b"\0asm":
        raise RuntimeError("Missing WebAssembly binary header")
    html = (args.site / "examples/index.html").read_text(encoding="utf-8-sig")
    if '../dist/index.js' not in html or '../dist/wasm/wasm_audio_engine.js' not in html:
        raise RuntimeError("Demo imports do not preserve the project subpath")
    print("Static artifact identity, allowlist, and hashes verified")


if __name__ == "__main__":
    main()
