#!/usr/bin/env python3
"""生成应用图标资源。仅依赖标准库，输出 PNG / ICO，ICNS 交给 macOS 的 iconutil。

用法: python3 scripts/generate-icons.py
"""

import os
import struct
import shutil
import subprocess
import sys
import zlib

RESOURCES = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "resources", "icons")


def lerp(a, b, t):
    return a + (b - a) * t


def rounded_rect(x, y, left, top, right, bottom, radius):
    if x < left or x > right or y < top or y > bottom:
        return False
    cx = min(max(x, left + radius), right - radius)
    cy = min(max(y, top + radius), bottom - radius)
    dx = x - cx
    dy = y - cy
    return dx * dx + dy * dy <= radius * radius


def triangle(px, py, a, b, c):
    def sign(p1, p2, p3):
        return (p1[0] - p3[0]) * (p2[1] - p3[1]) - (p2[0] - p3[0]) * (p1[1] - p3[1])

    p = (px, py)
    d1 = sign(p, a, b)
    d2 = sign(p, b, c)
    d3 = sign(p, c, a)
    has_neg = d1 < 0 or d2 < 0 or d3 < 0
    has_pos = d1 > 0 or d2 > 0 or d3 > 0
    return not (has_neg and has_pos)


def sample(u, v):
    """返回该采样点的 rgba，越界或透明处返回 None。"""
    if not rounded_rect(u, v, 0.0, 0.0, 1.0, 1.0, 0.22):
        return None
    top = (0x35, 0x35, 0x3D)
    bottom = (0x1B, 0x1B, 0x1F)
    color = [int(lerp(top[i], bottom[i], v)) for i in range(3)] + [255]

    if rounded_rect(u, v, 0.16, 0.20, 0.84, 0.80, 0.09):
        color = [0x10, 0x10, 0x13, 255]

    dx = u - 0.62
    dy = v - 0.34
    if dx * dx + dy * dy <= 0.075 * 0.075:
        color = [0x4C, 0x9A, 0xFF, 255]

    if triangle(u, v, (0.16, 0.80), (0.45, 0.44), (0.66, 0.80)):
        color = [0x45, 0x45, 0x50, 255]
    if triangle(u, v, (0.40, 0.80), (0.62, 0.52), (0.84, 0.80)):
        color = [0x98, 0x98, 0xA2, 255]

    return color


def render(size):
    samples = 3
    rows = []
    for y in range(size):
        row = bytearray()
        for x in range(size):
            accum = [0, 0, 0, 0]
            hits = 0
            for sy in range(samples):
                for sx in range(samples):
                    u = (x + (sx + 0.5) / samples) / size
                    v = (y + (sy + 0.5) / samples) / size
                    color = sample(u, v)
                    if color is None:
                        continue
                    hits += 1
                    for i in range(4):
                        accum[i] += color[i]
            if hits:
                row.extend(bytes(accum[i] // hits for i in range(4)))
            else:
                row.extend(b"\x00\x00\x00\x00")
        rows.append(bytes(row))
    return rows


def write_png(path, size):
    rows = render(size)
    raw = b"".join(b"\x00" + row for row in rows)

    def chunk(tag, data):
        return (
            struct.pack(">I", len(data))
            + tag
            + data
            + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
        )

    ihdr = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)
    data = (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", ihdr)
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )
    with open(path, "wb") as handle:
        handle.write(data)
    return data


def write_ico(path, png_by_size):
    sizes = sorted(png_by_size)
    header = struct.pack("<HHH", 0, 1, len(sizes))
    offset = 6 + 16 * len(sizes)
    entries = b""
    payload = b""
    for size in sizes:
        data = png_by_size[size]
        entries += struct.pack(
            "<BBBBHHII", 0 if size >= 256 else size, 0 if size >= 256 else size, 0, 0, 1, 32, len(data), offset
        )
        offset += len(data)
        payload += data
    with open(path, "wb") as handle:
        handle.write(header + entries + payload)


def main():
    os.makedirs(RESOURCES, exist_ok=True)
    png_by_size = {}
    for size in (16, 32, 48, 64, 128, 256, 512, 1024):
        png_by_size[size] = write_png(os.path.join(RESOURCES, f"icon-{size}.png"), size)

    with open(os.path.join(RESOURCES, "icon.png"), "wb") as handle:
        handle.write(png_by_size[256])

    write_ico(
        os.path.join(RESOURCES, "icon.ico"),
        {size: data for size, data in png_by_size.items() if size <= 256},
    )

    iconset = os.path.join(RESOURCES, "icon.iconset")
    os.makedirs(iconset, exist_ok=True)
    mapping = {
        "icon_16x16.png": 16,
        "icon_16x16@2x.png": 32,
        "icon_32x32.png": 32,
        "icon_32x32@2x.png": 64,
        "icon_128x128.png": 128,
        "icon_128x128@2x.png": 256,
        "icon_256x256.png": 256,
        "icon_256x256@2x.png": 512,
        "icon_512x512.png": 512,
        "icon_512x512@2x.png": 1024,
    }
    for name, size in mapping.items():
        with open(os.path.join(iconset, name), "wb") as handle:
            handle.write(png_by_size[size])

    if sys.platform == "darwin" and shutil_which("iconutil"):
        subprocess.run(
            ["iconutil", "-c", "icns", iconset, "-o", os.path.join(RESOURCES, "icon.icns")],
            check=True,
        )
        print("已生成 icon.icns")

    for size in png_by_size:
        os.remove(os.path.join(RESOURCES, f"icon-{size}.png"))
    shutil.rmtree(iconset, ignore_errors=True)
    print(f"图标已写入 {RESOURCES}")


def shutil_which(name):
    from shutil import which

    return which(name)


if __name__ == "__main__":
    main()
