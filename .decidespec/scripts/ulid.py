#!/usr/bin/env python3
"""ULID を1つ(または N 個)生成して標準出力に出す。依存なし。

使い方: python3 scripts/ulid.py [N]
仕様: https://github.com/ulid/spec(48bit ミリ秒時刻 + 80bit 乱数、Crockford Base32、26文字)
"""
from __future__ import annotations

import os
import sys
import time

ALPHABET = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"


def ulid(ts_ms: int | None = None) -> str:
    if ts_ms is None:
        ts_ms = int(time.time() * 1000)
    value = (ts_ms << 80) | int.from_bytes(os.urandom(10), "big")
    out = []
    for _ in range(26):
        out.append(ALPHABET[value & 31])
        value >>= 5
    return "".join(reversed(out))


if __name__ == "__main__":
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 1
    for _ in range(n):
        print(ulid())
