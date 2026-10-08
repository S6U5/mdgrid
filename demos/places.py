#!/usr/bin/env python3
"""機能紹介: 登録した表(CLI-18・CLI-19。2026-10-08)。"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import tour  # noqa: E402

TITLE = "登録した表を切り替えて開く"


def setup(t):
    ex = os.path.join(t.work, "examples")
    t.file("config/mdgrid/places.toml", f"""# よく使う表。手で書いても、画面の「この表を登録」で足してもよい。
[[place]]
name = "タスク"
group = "仕事"
path = "{ex}/vault/タスク.base"
view = "進行中"

[[place]]
name = "予定"
group = "仕事"
path = "{ex}/schedule"

[[place]]
name = "人と AI"
group = "見本"
path = "{ex}/ai-human/vault"
""")


def steps(t):
    t.keys("C-l")
    t.say("よく使う表は places.toml に、名前と分類を付けて登録できる")
    t.run("cat $XDG_CONFIG_HOME/mdgrid/places.toml")
    t.say("3つ登録してある。引数なしで mdgrid を起動してみる")
    t.keys("C-l")
    t.run("mdgrid")
    t.say("今のフォルダの表の上に、登録した表が分類ごとに出る")
    t.say("打つと絞れる。「タスク」と打って Enter")
    t.type("タスク")
    t.pause(1.2)
    t.keys("Enter")
    t.say("タスク.base が、登録したビュー「進行中」で開いた")
    t.say("開いたままでも切り替えられる。パレット(:)の「登録した表を開く」")
    t.type(":")
    t.pause(0.6)
    t.type("登録した表を開く")
    t.pause(1)
    t.keys("Enter")
    t.type("予定")
    t.pause(1.2)
    t.keys("Enter")
    t.say("予定の表に移った(変更があれば、移る前に保存するか聞く)")
    t.say("今の表を登録するには、パレットの「この表を登録」")
    t.type(":")
    t.pause(0.6)
    t.type("この表を登録")
    t.pause(1)
    t.keys("Enter")
    t.say("名前の欄には、今のビューかフォルダの名前が入っている。「会議」に変える")
    t.back(12)
    t.type("会議")
    t.pause(0.8)
    t.keys("Enter")
    t.say("分類は今あるものから選ぶか、新しく打つ。「仕事」を選ぶ")
    t.keys("Enter")
    t.say("登録できた。一覧を開いて確かめる")
    t.type(":")
    t.pause(0.6)
    t.type("登録した表を開く")
    t.pause(1)
    t.keys("Enter")
    t.say("「仕事 › 会議」が増えた", 3.5)
    t.keys("Escape")
    t.type("q")
    t.pause(1)


if __name__ == "__main__":
    tour.main(TITLE, steps, setup)
