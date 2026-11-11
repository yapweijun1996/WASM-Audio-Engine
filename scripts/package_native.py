"""Package one explicitly built native library with headers and license notices."""
import argparse
import hashlib
import json
from pathlib import Path
from zipfile import ZipFile, ZIP_DEFLATED

ROOT = Path(__file__).resolve().parent.parent

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--library', type=Path, required=True)
    parser.add_argument('--target', required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    library = args.library.resolve(strict=True)
    if library.suffix not in {'.so', '.dll', '.dylib', '.a', '.lib'}:
        parser.error('Expected an explicitly built native library')
    files = {
        'include/wasm_audio_engine.h': ROOT / 'include/wasm_audio_engine.h',
        'LICENSE': ROOT / 'LICENSE',
        'THIRD_PARTY_NOTICES.txt': ROOT / 'THIRD_PARTY_NOTICES.txt',
        'lib/' + library.name: library,
    }
    manifest = {'schemaVersion': 1, 'apiVersion': '0.1.0', 'target': args.target,
                'files': {name: hashlib.sha256(path.read_bytes()).hexdigest() for name, path in files.items()}}
    # Exclusive creation protects previous build artifacts from accidental replacement.
    with ZipFile(args.output, mode='x', compression=ZIP_DEFLATED) as archive:
        for name, path in files.items():
            archive.write(path, name)
        archive.writestr('manifest.json', json.dumps(manifest, indent=2) + '\n')
    with ZipFile(args.output) as archive:
        for name, expected in manifest['files'].items():
            if hashlib.sha256(archive.read(name)).hexdigest() != expected:
                raise RuntimeError('Archive verification failed: ' + name)
    print('Verified native archive:', args.output)

if __name__ == '__main__':
    main()
