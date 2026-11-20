# グラフの画面(CH-2〜CH-8): タスクの台帳(examples/tasks)を P でグラフにし、種類と印のグラフを切り替え、棒から表を絞る。
# 撮る: vhs demos/chart.tape → demos/out/demo-chart.gif

Output demos/out/demo-chart.gif
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

# P で状態ごとの棒。Tab で積み上げ棒、] で印の次のグラフ(月ごとの期限の折れ線)、] でヒートマップ、[ [ で棒へ戻り、
# j で doing を選んで Enter で表をその状態に絞る。
Type "P" Sleep 2s
Type "j" Sleep 800ms
Tab Sleep 1.5s
Type "]" Sleep 2s
Type "]" Sleep 2s
Type "[" Sleep 800ms
Type "[" Sleep 1.5s
Type "j" Sleep 800ms
Enter Sleep 2s
Type "q" Sleep 800ms
