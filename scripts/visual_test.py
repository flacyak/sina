"""Visual test: show the QR code in a real terminal emulator, screenshot it and decode the pixels.

Runs xterm in a dark theme and a light theme to prove the symbol scans regardless of the
terminal's colors. Requires xterm, ImageMagick's `import`, and the zxing-cpp and Pillow packages.
Usage: visual_test.py <path-to-sina-binary> [output-dir]
Screenshots are written to output-dir (default: visual-screenshots) for inspection.
"""
import subprocess
import sys
import time
from pathlib import Path

from PIL import Image
import zxingcpp

TEXT = "https://github.com/flacyak/sina"
THEMES = {
    "dark": ["-bg", "black", "-fg", "white"],
    "light": ["-bg", "white", "-fg", "black"],
}
WINDOW_TITLE_PREFIX = "sina-visual-test"
SETTLE_SECONDS = 1.5


def screenshot_terminal(binary: str, theme: str, shot: Path) -> None:
    title = f"{WINDOW_TITLE_PREFIX}-{theme}"
    xterm = subprocess.Popen(
        ["xterm", "-T", title, "-fa", "DejaVu Sans Mono", "-fs", "14", "-geometry", "60x30",
         *THEMES[theme], "-hold", "-e", binary, TEXT],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    try:
        time.sleep(SETTLE_SECONDS)
        subprocess.run(["import", "-window", title, str(shot)], check=True)
    finally:
        xterm.terminate()
        xterm.wait()


def main() -> int:
    binary = sys.argv[1]
    out_dir = Path(sys.argv[2] if len(sys.argv) > 2 else "visual-screenshots")
    out_dir.mkdir(parents=True, exist_ok=True)
    failures = 0
    for theme in THEMES:
        shot = out_dir / f"xterm-{theme}.png"
        screenshot_terminal(binary, theme, shot)
        results = zxingcpp.read_barcodes(Image.open(shot))
        decoded = results[0].text if results else None
        status = "ok" if decoded == TEXT else f"FAIL (decoded {decoded!r})"
        print(f"{theme:5} theme: {status} ({shot})")
        failures += decoded != TEXT
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
