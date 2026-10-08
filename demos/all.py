#!/usr/bin/env python3
"""mdgrid の機能紹介を、1つの流れでまとめて見せる。

    python3 all.py                  # 全部(下の並びの順)
    python3 all.py places themes    # 選んだものだけ(名前はファイル名)
    python3 all.py --list           # 名前の一覧

新しい紹介を足したら SECTIONS に名前を足す。
"""
import importlib
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import tour  # noqa: E402

# 見せる順。値の入れ方 → 作る → 出す → 切り替える → 見た目。
SECTIONS = ["values", "new_note", "export", "places", "themes"]


def main(argv):
    if argv[:1] == ["--list"]:
        for name in SECTIONS:
            print(f"{name:10} {importlib.import_module(name).TITLE}")
        return
    names = argv or SECTIONS
    unknown = [n for n in names if n not in SECTIONS]
    if unknown:
        sys.exit("知らない名前: %s(--list で一覧)" % ", ".join(unknown))
    mods = [importlib.import_module(n) for n in names]
    t = tour.Tour("mdgrid の機能をひととおり" if len(mods) > 1 else mods[0].TITLE)
    for m in mods:
        if hasattr(m, "setup"):
            m.setup(t)

    def steps(t):
        for i, m in enumerate(mods, 1):
            if i > 1:
                t.home()
            t.say(f"{i}/{len(mods)}  {m.TITLE}", 2.5)
            m.steps(t)

    t.play(steps)


if __name__ == "__main__":
    main(sys.argv[1:])
