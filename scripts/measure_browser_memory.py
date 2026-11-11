"""Measure an isolated local Chrome process tree while processing one WAV file."""
import argparse
import json
import subprocess
import tempfile
import time
from pathlib import Path
import psutil

ROOT = Path(__file__).resolve().parent.parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--input', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
profile = Path(tempfile.mkdtemp(prefix='wasm-audio-memory-chrome-'))
chrome = Path('C:/Program Files/Google/Chrome/Application/chrome.exe')
process = subprocess.Popen([str(chrome), '--headless=new', '--no-first-run', '--disable-extensions', '--disable-background-networking', '--remote-debugging-port=0', '--user-data-dir='+str(profile), 'about:blank'], creationflags=subprocess.CREATE_NO_WINDOW, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
root = psutil.Process(process.pid)
private_peak = rss_peak = samples = 0
runner = None
try:
    for _ in range(200):
        if (profile / 'DevToolsActivePort').exists():
            break
        time.sleep(0.1)
    port = (profile / 'DevToolsActivePort').read_text().splitlines()[0]
    result_path = profile / 'page-result.json'
    runner = subprocess.Popen(['node', str(ROOT / 'scripts/run_browser_file.mjs'), port, str(args.input.resolve(strict=True)), str(result_path)], cwd=ROOT)
    while runner.poll() is None:
        try:
            processes = [root] + root.children(recursive=True)
            rss = private = 0
            for child in processes:
                try:
                    info = child.memory_info()
                    rss += info.rss
                    private += info.private
                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    pass
            rss_peak = max(rss_peak, rss)
            private_peak = max(private_peak, private)
            samples += 1
        except psutil.NoSuchProcess:
            pass
        time.sleep(0.25)
    if runner.returncode:
        raise RuntimeError('Browser measurement runner failed')
    result = json.loads(result_path.read_text())
    result.update(sampledProcessTreePrivateBytes=private_peak, sampledProcessTreeWorkingSetBytes=rss_peak, sampleIntervalSeconds=0.25, sampleCount=samples, profileDirectory=str(profile), measurement='Sum of isolated Chrome parent/descendant process metrics; sampled high-water marks, not continuous peaks. Working-set sum may count shared pages repeatedly. Private bytes measure committed private memory, not resident RAM.')
    args.output.write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(result, indent=2))
finally:
    if runner and runner.poll() is None:
        runner.terminate()
    if process.poll() is None:
        process.terminate()
    # Keep the isolated profile for audit; do not recursively delete generated browser data.
