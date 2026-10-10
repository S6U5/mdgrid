# WBS: 親子で並べた表に番号と進み具合を出し、値の対応表を設定の画面で作る(NV-28)。
# 撮る: vhs demos/wbs.tape → demos/out/demo-wbs.gif

Output demos/out/demo-wbs.gif
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
Type `mkdir -p "$DEMO/wbs" && cd "$DEMO/wbs" && printf -- '---\nstatus: doing\n---\n' > Launch.md && printf -- '---\nparent: "[[Launch]]"\nstatus: doing\n---\n' > Website.md && printf -- '---\nparent: "[[Website]]"\nstatus: done\n---\n' > Design.md && printf -- '---\nparent: "[[Website]]"\nstatus: todo\n---\n' > Copy.md && printf -- '---\nparent: "[[Launch]]"\nstatus: todo\n---\n' > Press.md && clear`
Enter
Sleep 300ms
Show

Type "mdgrid ." Sleep 400ms Enter
Sleep 2s

# 親子の区画(Tab 3回): 親子と WBS をオン
Type "o" Sleep 800ms
Tab 3 Sleep 1s
Enter Sleep 600ms
Down 2 Sleep 400ms
Enter Sleep 1.2s
# 値の対応表(doing・todo・done の順)
Down 2 Sleep 400ms
Enter Sleep 500ms
Type "50 Doing" Sleep 300ms Enter
Sleep 600ms
Down Sleep 300ms
Enter Sleep 500ms
Type "0 To do" Sleep 300ms Enter
Sleep 600ms
Down Sleep 300ms
Enter Sleep 500ms
Type "100 Done" Sleep 300ms Enter
Sleep 1.5s
# 反映(Shift+Tab 5回で上の Apply)
Shift+Tab Shift+Tab Shift+Tab Shift+Tab Shift+Tab Sleep 600ms
Enter Sleep 3s
Type "q" Sleep 800ms
