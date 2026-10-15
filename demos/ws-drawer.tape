# 表の一覧の引き出し: `w`(またはヘッダーのワークスペース)で左から重ねて出し、同じワークスペースの表や、
# ほかのワークスペースの表へ移る。普段は出さない(WS-8・SR-42)。
# 撮る: vhs demos/ws-drawer.tape → demos/out/demo-ws-drawer.gif

Output demos/out/demo-ws-drawer.gif
Set Shell bash
Set FontFamily "BIZ UDGothic"
Set FontSize 15
Set LineHeight 1.25
Set Width 1500
Set Height 860
Set TypingSpeed 60ms
Env LANG "en_US.UTF-8"
Env COLORTERM "truecolor"

Source demos/setup.tape

# studio のフォルダは印で「Northwind Studio」(Tasks・Projects・Clients・People・Meetings)。別に Office のワークスペースを作っておく(映さない)
Hide
Type "mdgrid ../quirks/Tasks --add-to Office >/dev/null && clear"
Enter
Sleep 300ms
Show

# Tasks を開く(範囲は Northwind Studio)。普段は引き出しが無い
Type "mdgrid Tasks" Sleep 400ms Enter
Sleep 2.5s

# w で引き出し: 範囲の表(今の表 Tasks に印)と、畳んだ Office
Type "w" Sleep 2s
# Projects へ移る
Down Sleep 800ms
Enter Sleep 2.5s

# もう一度開き、Office を開いてその表へ(範囲は Office になる)
Type "w" Sleep 1.5s
Type "G" Sleep 800ms
Enter Sleep 1.5s
Down Sleep 800ms
Enter Sleep 3s

# 開いて Esc なら何も変えずに閉じる
Type "w" Sleep 1.5s
Escape Sleep 1.5s
Type "q" Sleep 800ms
