#!/usr/bin/env python3
"""機能紹介: 画面の表をファイルに書き出す(OUT-2・OUT-5。2026-10-07)と、--print の形。"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import tour  # noqa: E402

TITLE = "表をファイルに書き出す"


def steps(t):
    t.keys("C-l")
    t.say("見えている表を、そのまま CSV・TSV・JSON・Markdown に書き出せる")
    t.run("mdgrid .")
    t.say("まず \\ で「doing」の行だけに絞る")
    t.type("\\")
    t.pause(0.6)
    t.type("doing")
    t.pause(0.8)
    t.keys("Enter")
    t.say("パレット(:)の「表をファイルに書き出す」")
    t.type(":")
    t.pause(0.6)
    t.type("表をファイルに書き出す")
    t.pause(1)
    t.keys("Enter")
    t.say("形は拡張子で決まる。doing.csv と打つ")
    t.type("doing.csv")
    t.pause(0.8)
    t.keys("Enter")
    t.say("書けた。終わって中身を見る")
    t.type("q")
    t.pause(1)
    t.keys("C-l")
    t.run("cat doing.csv")
    t.say("先頭の列は各ノートのパス。表計算で直して --apply で戻せる", 4)
    t.say("画面を出さずに出すなら --print(csv・tsv・json・md)")
    t.keys("C-l")
    t.run("mdgrid --print --format md --filter 'status == \"todo\"' --sort due")
    t.pause(3)


if __name__ == "__main__":
    tour.main(TITLE, steps)
