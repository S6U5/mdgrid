# 見た目の区画: テーマと組を選んで反映し、テンプレートとして保存する(SR-43)。
# 撮る: vhs demos/look.tape → demos/out/demo-look.gif

Output demos/out/demo-look.gif
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

Type "mdgrid ." Sleep 400ms Enter
Sleep 2s

# 見た目の区画(Tab 5回)
Type "o" Sleep 800ms
Tab 5 Sleep 1.2s
# テーマ: dracula
Enter Sleep 800ms
Down 4 Sleep 800ms
Enter Sleep 800ms
# 組: dozy-pink
Down Sleep 400ms
Enter Sleep 800ms
Down 6 Sleep 800ms
Enter Sleep 1.2s
# テンプレートとして保存
Down 2 Sleep 400ms
Enter Sleep 600ms
Type "Night" Sleep 400ms Enter
Sleep 1.2s
# 反映(Tab で上の Apply へ)
Tab Sleep 600ms
Enter Sleep 2.5s
Type "q" Sleep 800ms
