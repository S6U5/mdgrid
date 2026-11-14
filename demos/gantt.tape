# ガントの画面(GA-1〜GA-4): タスクの台帳(examples/tasks)を C でガントにし、単位と位置を変える。
# 撮る: vhs demos/gantt.tape → demos/out/demo-gantt.gif

Output demos/out/demo-gantt.gif
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

# C でガント。j で行を選び、- で週、+ で日に戻し、l で後ろへ、t で今日へ。
Type "C" Sleep 2s
Type "j" Sleep 800ms
Type "j" Sleep 800ms
Type "-" Sleep 1.5s
Type "+" Sleep 1.5s
Type "l" Sleep 400ms
Type "l" Sleep 400ms
Type "l" Sleep 1s
Type "t" Sleep 1.5s
Enter Sleep 1.5s
Type "q" Sleep 800ms
