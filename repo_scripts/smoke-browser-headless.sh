#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARTIFACT_DIR="${CITYBOUND_SMOKE_ARTIFACT_DIR:-$ROOT_DIR/output/playwright}"
SERVER_BIN="${CITYBOUND_SERVER_BIN:-$ROOT_DIR/target/release/citybound}"
SMOKE_URL="${CITYBOUND_SMOKE_URL:-http://localhost:1234}"

find_headless_shell() {
    local candidates=(
        "${CHROME_HEADLESS_SHELL:-}"
        "$HOME/Library/Caches/ms-playwright"
        "$HOME/.cache/ms-playwright"
        "$HOME/AppData/Local/ms-playwright"
    )

    local candidate
    for candidate in "${candidates[@]}"; do
        if [[ -n "$candidate" && -f "$candidate" && -x "$candidate" ]]; then
            printf '%s\n' "$candidate"
            return 0
        fi

        if [[ -d "$candidate" ]]; then
            local found
            found="$(find "$candidate" -path '*chrome-headless-shell*' -type f -name 'chrome-headless-shell' | sort | tail -n 1)"
            if [[ -n "$found" ]]; then
                printf '%s\n' "$found"
                return 0
            fi
        fi
    done

    return 1
}

fail() {
    printf '%s\n' "$1" >&2
    exit 1
}

HEADLESS_BIN="$(find_headless_shell || true)"

[[ -x "$SERVER_BIN" ]] || fail "Missing server binary at $SERVER_BIN. Run npm run build-server first."
[[ -n "$HEADLESS_BIN" && -x "$HEADLESS_BIN" ]] || fail "Missing Playwright Chromium headless shell. Run npx playwright install chromium or set CHROME_HEADLESS_SHELL."
command -v python3 >/dev/null 2>&1 || fail "python3 is required for the browser smoke test."

mkdir -p "$ARTIFACT_DIR"
find "$ARTIFACT_DIR" -type f -delete 2>/dev/null || true

python3 - "$SERVER_BIN" "$HEADLESS_BIN" "$SMOKE_URL" "$ARTIFACT_DIR" <<'PY'
import errno
import os
import pty
import signal
import subprocess
import sys
import threading
import time
import urllib.request

server_bin, headless_bin, smoke_url, artifact_dir = sys.argv[1:5]
server_log_path = os.path.join(artifact_dir, "server.log")
browser_dom_path = os.path.join(artifact_dir, "browser-smoke-dom.html")
browser_log_path = os.path.join(artifact_dir, "browser-smoke-headless.log")
browser_png_path = os.path.join(artifact_dir, "browser-smoke-headless.png")

master_fd, slave_fd = pty.openpty()
server_log = open(server_log_path, "ab", buffering=0)

server = subprocess.Popen(
    [server_bin],
    stdin=slave_fd,
    stdout=slave_fd,
    stderr=slave_fd,
    start_new_session=True,
)
os.close(slave_fd)

def cleanup():
    if server.poll() is None:
        try:
            os.killpg(server.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        try:
            server.wait(timeout=5)
        except subprocess.TimeoutExpired:
            try:
                os.killpg(server.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            server.wait(timeout=5)
    try:
        os.close(master_fd)
    except OSError:
        pass
    server_log.close()

def fail(message):
    cleanup()
    print(message, file=sys.stderr)
    raise SystemExit(1)

def drain_server_output():
    while True:
        try:
            chunk = os.read(master_fd, 4096)
            if not chunk:
                return
            server_log.write(chunk)
        except OSError as exc:
            if exc.errno == errno.EIO:
                return
            raise

drain_thread = threading.Thread(target=drain_server_output, daemon=True)
drain_thread.start()

deadline = time.time() + 20
last_error = None
while time.time() < deadline:
    if server.poll() is not None:
        fail(
            f"Citybound server exited before it became ready (exit code {server.returncode}). "
            f"See {server_log_path}."
        )

    try:
        with urllib.request.urlopen(smoke_url, timeout=1):
            break
    except Exception as exc:
        last_error = exc
        time.sleep(0.2)
else:
    fail(
        f"Citybound server did not become ready at {smoke_url}. "
        f"Last probe error: {last_error}. See {server_log_path}."
    )

with open(browser_dom_path, "wb") as browser_dom, open(browser_log_path, "wb") as browser_log:
    browser = subprocess.run(
        [
            headless_bin,
            "--headless",
            "--no-sandbox",
            "--disable-gpu",
            "--hide-scrollbars",
            "--enable-logging=stderr",
            "--v=1",
            "--virtual-time-budget=15000",
            "--window-size=1440,900",
            f"--screenshot={browser_png_path}",
            "--dump-dom",
            smoke_url,
        ],
        stdout=browser_dom,
        stderr=browser_log,
        check=False,
    )

cleanup()

if browser.returncode != 0:
    print(
        f"Headless browser exited with code {browser.returncode}. "
        f"See {browser_log_path}.",
        file=sys.stderr,
    )
    raise SystemExit(1)

with open(browser_log_path, "r", encoding="utf-8", errors="replace") as browser_log:
    browser_log_text = browser_log.read()

if '"After setup"' not in browser_log_text:
    print(
        f"Browser smoke failed before wasm setup completed. See {browser_log_path}.",
        file=sys.stderr,
    )
    raise SystemExit(1)

for needle in ("Uncaught", "Unhandled Rejection", "Rust WASM error", "ReferenceError:", "TypeError:"):
    if needle in browser_log_text:
        print(
            f"Browser smoke hit an uncaught runtime error ({needle}). "
            f"See {browser_log_path} and {browser_dom_path}.",
            file=sys.stderr,
        )
        raise SystemExit(1)

with open(browser_dom_path, "r", encoding="utf-8", errors="replace") as browser_dom:
    browser_dom_text = browser_dom.read()

if 'id="errors" class="errorsHappened"' in browser_dom_text:
    print(
        f"Browser smoke activated the in-app error overlay. See {browser_dom_path}.",
        file=sys.stderr,
    )
    raise SystemExit(1)

print("Headless browser smoke passed.")
print(f"Artifacts: {artifact_dir}")
PY
