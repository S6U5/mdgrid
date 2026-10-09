# ワークスペース: 表をまとめ(--add-to)、一覧を見て(--workspaces)、-w で開く。リンクと関係マップはその表だけ(WS-2〜WS-6)。
# 撮る: vhs demos/workspace.tape → demos/out/demo-workspace.gif

Output demos/out/demo-workspace.gif
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

Hide
Type "cd ../relations && clear"
Enter
Sleep 300ms
Show

# tasks・projects・members を Work にまとめる
Type "mdgrid tasks projects members --add-to Work" Sleep 400ms Enter
Sleep 1.2s
Type "mdgrid --workspaces" Sleep 400ms Enter
Sleep 2s

# Work で開く(最初の表。ヘッダーに · workspace Work)
Type "mdgrid -w Work" Sleep 400ms Enter
Sleep 2.5s

# 関係マップは Work の表(登録した表とは別のまとまり)
Type "R" Sleep 3s
Right Sleep 1.5s
Type "R" Sleep 1.2s
Type "q" Sleep 800ms
