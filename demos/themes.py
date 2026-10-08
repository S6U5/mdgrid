#!/usr/bin/env python3
"""機能紹介: 色のテーマ(SR-26・SR-27)。テーマは設定の theme で選ぶ(画面からは変えられない)。"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import tour  # noqa: E402

THEMES = ["nord", "solarized-light", "dracula", "gruvbox", "pink-monster", "dozy-pink"]

TITLE = "色のテーマを切り替える"


def steps(t):
    t.keys("C-l")
    t.say("テーマは設定の theme の1行で選ぶ。既定は端末の色のまま")
    t.run("mdgrid .")
    t.say("これが既定(端末の色)", 3)
    t.type("q")
    t.pause(0.8)
    t.keys("C-l")
    t.run("cat ../themes/nord.toml")
    t.say("見本の設定は examples/themes/ にある。--config で読ませて開く", 3.5)
    for name in THEMES:
        t.keys("C-l")
        t.say(f"theme = \"{name}\"", 0.5)
        t.run(f"mdgrid --config ../themes/{name}.toml .")
        t.keys("j", "j", "l")
        t.pause(2.5)
        t.type("q")
        t.pause(0.8)
    t.keys("C-l")
    t.say("自分の設定(~/.config/mdgrid/config.toml)なら theme = \"nord\" の1行でよい。NO_COLOR にも従う", 4.5)


if __name__ == "__main__":
    tour.main(TITLE, steps)
