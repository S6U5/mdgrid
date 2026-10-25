# スターターから台帳を作る(ST-3・ST-4・ST-5): --starters の一覧、--init-workspace --starter で作って開く、
# パレットの「スターターから台帳を作る」。
# 撮る: vhs demos/starter.tape → demos/out/demo-starter.gif

Output demos/out/demo-starter.gif
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

# 一覧
Type "mdgrid --starters" Sleep 400ms Enter
Sleep 2.5s

# 読書の記録を見本つきで作って開く
Type `mdgrid "$DEMO/books" --init-workspace --starter reading --with-samples` Sleep 400ms Enter
Sleep 2.5s
Type `mdgrid "$DEMO/books"` Sleep 400ms Enter
Sleep 2.5s
Type "]" Sleep 1.2s
Type "]" Sleep 1.2s

# パレットからも(スターターの一覧まで見せて、やめる)
Type ":" Sleep 800ms
Type "starter" Sleep 1.2s
Enter Sleep 2s
Escape Sleep 800ms
Type "q" Sleep 800ms
