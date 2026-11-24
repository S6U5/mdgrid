# ルーチンの回(routine.*): 見本 examples/routines を開くと、下の帯に来ている回の数が出る。パレットの routine_runs で
# 作るノートの名前を見て、y と Enter で Runs/ に作る。続けて mdgrid --runs が、もう作る回の無いことを確かめる。
# 撮る: vhs demos/routine.tape → demos/out/demo-routine.gif

Output demos/out/demo-routine.gif
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

Type "cd ../routines && mdgrid" Sleep 400ms Enter
Sleep 2.5s

# 来ている回の名前を見て、作る
Type ":" Sleep 600ms
Type "routine_runs" Sleep 800ms
Enter Sleep 2.5s
Type "y" Sleep 600ms
Enter Sleep 2s
Type "q" Sleep 800ms

# できた回のノートと、もう作る回の無い --runs
Type "ls Runs" Sleep 400ms Enter
Sleep 1.5s
Type "mdgrid --runs; echo $?" Sleep 400ms Enter
Sleep 2s
