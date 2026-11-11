"""Generate or verify license-text inventory for locked Rust dependencies."""

import argparse
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / "THIRD_PARTY_NOTICES.txt"
LICENSE_NAME = re.compile(r"^(license|copying|notice)", re.IGNORECASE)


def generate():
    result = subprocess.run(
        ["cargo", "metadata", "--locked", "--format-version", "1"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    metadata = json.loads(result.stdout)
    sections = [
        "Third-party Rust dependency license inventory\n"
        "Generated from Cargo.lock. Inclusion does not determine which "
        "license option applies or resolve patent questions.\n"
    ]
    packages = sorted(metadata["packages"], key=lambda item: (item["name"], item["version"]))
    for package in packages:
        if package["source"] is None:
            continue
        root = Path(package["manifest_path"]).resolve().parent
        files = sorted(
            (path for path in root.iterdir() if path.is_file() and LICENSE_NAME.match(path.name)),
            key=lambda path: path.name.lower(),
        )
        if not package.get("license") or not files:
            raise RuntimeError(f"Missing declared license or license text: {package['name']} {package['version']}")
        sections.append(f"\n{'=' * 72}\n{package['name']} {package['version']}\nDeclared license: {package['license']}\n")
        for path in files:
            resolved = path.resolve()
            if not resolved.is_relative_to(root):
                raise RuntimeError(f"License path escapes package: {path}")
            sections.append(f"\n--- {path.name} ---\n{resolved.read_text(encoding='utf-8').strip()}\n")
    return "".join(sections)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true", help="Update the generated inventory")
    args = parser.parse_args()
    expected = generate()
    if args.write:
        OUTPUT.write_text(expected, encoding="utf-8", newline="\n")
        print(f"Wrote {OUTPUT.name}")
    elif not OUTPUT.exists() or OUTPUT.read_text(encoding="utf-8") != expected:
        raise SystemExit("License inventory is missing or stale; run with --write and review it.")
    else:
        print("License inventory matches locked Rust dependencies")


if __name__ == "__main__":
    main()
