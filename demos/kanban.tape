# カンバンの画面(KB-2〜KB-4): タスクの台帳(examples/tasks)を B でカンバンにし、札を動かして状態を書き換え、取り消す。
# 撮る: vhs demos/kanban.tape → demos/out/demo-kanban.gif

Output demos/out/demo-kanban.gif
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
Type `cd "$DEMO/examples/tasks" && clear`
Enter
Sleep 300ms
Show

Type "mdgrid ." Sleep 400ms Enter
Sleep 2s

# B でカンバン。j で札を選び、L で doing へ動かし、l でレーンを移り、u で取り消し、Enter で表のその行へ。
Type "B" Sleep 2s
Type "j" Sleep 800ms
Type "L" Sleep 1.5s
Type "l" Sleep 800ms
Type "l" Sleep 800ms
Type "u" Sleep 1.5s
Enter Sleep 1.5s
Type "q" Sleep 800ms
