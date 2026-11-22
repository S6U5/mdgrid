# 名前を付けたセッション(cli.session-*): パン屋の仕事だけに絞り、期日の順に並べた表示を、パレットの「セッションを保存」で
# 名前 bakery にする。終えてから mdgrid --sessions で一覧を見て、mdgrid -s bakery で同じ表示のまま開く。
# 撮る: vhs demos/sessions.tape → demos/out/demo-sessions.gif

Output demos/out/demo-sessions.gif
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

Type "mdgrid Tasks" Sleep 400ms Enter
Sleep 2.5s

# パン屋の仕事だけに絞る
Type "\" Sleep 800ms
Type "bakery" Sleep 600ms Enter
Sleep 1.5s

# パレットから今の表示をセッションとして保存する
Type ":" Sleep 800ms
Type "save_session" Sleep 800ms Enter
Sleep 1s
Type "bakery" Sleep 800ms Enter
Sleep 2s
Type "q" Sleep 800ms

# 保存したセッションの一覧を見て、名前で開く
Type "mdgrid --sessions" Sleep 600ms Enter
Sleep 2s
Type "mdgrid -s bakery" Sleep 600ms Enter
Sleep 3s
Type "q" Sleep 800ms
