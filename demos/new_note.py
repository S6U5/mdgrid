#!/usr/bin/env python3
"""機能紹介: 新しいノートのフォーム(CE-25〜CE-27・CE-32・CE-33。2026-10-07)。"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import tour  # noqa: E402

TITLE = "新しいノートをフォームで作る"


def setup(t):
    t.file("config/mdgrid/config.toml", """[new_note]
folder = "タスク"
ask = ["status", "due", "priority", "owner"]
required = ["due"]
hidden = ["created"]

[new_note.set]
status = "todo"
due = "{date+7}"
created = "{now}"
""")


def steps(t):
    t.keys("C-l")
    t.say("新しいノートの決まりは config.toml の [new_note] に書く")
    t.run("cat $XDG_CONFIG_HOME/mdgrid/config.toml")
    t.say("聞く欄・必須の欄・隠して入れる欄(作成日)・前もって入れる値(雛形の変数も使える)", 4.5)
    t.keys("C-l")
    t.run("mdgrid .")
    t.say("表で a(か右上の「+ 新規」)を押すと、フォームの窓が開く")
    t.type("a")
    t.pause(1)
    t.say("欄ごとに型の手がかりが出る。status と due には雛形の値が入っている")
    t.say("まず名前を打つ")
    t.type("牛乳を買う")
    t.pause(0.8)
    t.keys("Enter")
    t.say("Enter で次の欄へ。status は todo のまま")
    t.keys("Enter")
    t.say("due は必須。空にして作ろうとすると…")
    t.back(12)
    t.keys("C-s")
    t.say("空のあいだは作らず、その欄に理由を出す")
    t.type("2026-10-15")
    t.pause(0.8)
    t.keys("Enter")
    t.say("priority を入れて、Ctrl+S で作る(どの欄からでも作れる)")
    t.type("2")
    t.pause(0.8)
    t.keys("C-s")
    t.say("作ったノートが表に出た。中身を見てみる")
    t.type("q")
    t.pause(1)
    t.keys("C-l")
    t.run("cat タスク/牛乳を買う.md")
    t.say("隠した欄の created には、作った日時が入っている", 4)
    t.say('窓で Ctrl+E を押すと、作ってすぐエディタで開ける(mode = "editor" ならいつもそう)', 4.5)


if __name__ == "__main__":
    tour.main(TITLE, steps, setup)
