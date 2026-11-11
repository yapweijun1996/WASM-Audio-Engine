"""Generate PNG compatibility icons matching the canonical code-owned SVG design."""
import struct
import zlib
import xml.etree.ElementTree as ET
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent / "icons"


def png(size):
    rows = bytearray()
    svg = ET.parse(ROOT / "icon.svg").getroot()
    ns = "{http://www.w3.org/2000/svg}"
    group = svg.find(ns + "g")
    bars = [tuple(float(rect.attrib[name]) for name in ("x", "y", "width", "height", "rx")) for rect in group.findall(ns + "rect")]
    def color(value):
        return tuple(int(value[index:index + 2], 16) for index in (1, 3, 5))
    foreground = color(group.attrib["fill"])
    background = color(svg.find(ns + "rect").attrib["fill"])
    for y in range(size):
        rows.append(0)
        for x in range(size):
            px, py = (x + 0.5) * 100 / size, (y + 0.5) * 100 / size
            inside = False
            for bx, by, width, height, radius in bars:
                cx = min(max(px, bx + radius), bx + width - radius)
                cy = min(max(py, by + radius), by + height - radius)
                if (px - cx) ** 2 + (py - cy) ** 2 <= radius ** 2:
                    inside = True
            rows.extend(foreground if inside else background)

    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))

    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", size, size, 8, 2, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(rows, 9)) + chunk(b"IEND", b"")


if __name__ == "__main__":
    for name, size in [("icon-192.png", 192), ("icon-512.png", 512), ("maskable-512.png", 512), ("apple-touch-icon.png", 180)]:
        (ROOT / name).write_bytes(png(size))
    print("Generated PNG icon compatibility assets")
