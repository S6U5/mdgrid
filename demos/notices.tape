# 通知の欄(notice.*): 見本の設定の [[notices]] が遅れたタスクを数えて、ヘッダーの上に出す。m で欄へ移り、Enter でその行に絞る。
# 同じ知らせを mdgrid --notices で外に出す。
# 撮る: vhs demos/notices.tape → demos/out/demo-notices.gif

Output demos/out/demo-notices.gif
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

Type "mdgrid --config ../configs/studio.toml Tasks.base" Sleep 400ms Enter
Sleep 2.5s

# 欄へ移って、遅れたタスクに絞る
Type "m" Sleep 1.2s
Enter Sleep 2s
Type "q" Sleep 800ms

# 同じ知らせを外に出す
Type "mdgrid --config ../configs/studio.toml Tasks.base --notices" Sleep 600ms Enter
Sleep 2s
