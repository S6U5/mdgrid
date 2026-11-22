# お気に入りのワークスペースを行き来する(workspace.switch-*)と、最近開いた表へ戻る(cli.recent-*)。
# quirks で起動し、W と数字でお気に入りのワークスペースへ移る(カンバンからでも)。W の先頭の「最初に開いた所」で戻り、
# パレットの open_recent でさっきの表へ。終えてから mdgrid --recent で同じ並びをシェルに出す。
# お気に入りは見本の examples/places/mdgrid/workspaces.toml(Studio・Reading・People)。
# 撮る: vhs demos/switch.tape → demos/out/demo-switch.gif

Output demos/out/demo-switch.gif
Set Shell bash
Set FontFamily "BIZ UDGothic"
Set FontSize 15
Set LineHeight 1.2
Set Width 1500
Set Height 860
Set TypingSpeed 60ms
Env LANG "en_US.UTF-8"
Env COLORTERM "truecolor"

Source demos/setup.tape

# 設定のフォルダを、お気に入りを書いた見本にする(録画には映さない)。相対のパスはそのフォルダから読む
Hide
Type `export XDG_CONFIG_HOME="$DEMO/examples/places" && cd ../quirks && clear`
Enter
Sleep 300ms
Show

Type "mdgrid Tasks" Sleep 400ms Enter
Sleep 2.5s

# W でお気に入りの一覧。1 で Studio へ(この起動でまだ見ていないので、最初の表)
Type "W" Sleep 1.5s
Type "1" Sleep 2.5s

# カンバンの画面からでも W。2 で Reading へ
Type "B" Sleep 1.5s
Type "W" Sleep 1.5s
Type "2" Sleep 2.5s

# 先頭の「最初に開いた所」で quirks に戻る
Type "W" Sleep 1.5s
Enter Sleep 2.5s

# パレットの open_recent で、さっきまで見ていた表へ
Type ":open_recent" Sleep 800ms
Enter Sleep 2s
Enter Sleep 2.5s
Type "q" Sleep 800ms

# 同じ並びをシェルへ(パス・ビュー・日時をタブで区切る)
Type "mdgrid --recent" Sleep 600ms Enter
Sleep 2.5s
