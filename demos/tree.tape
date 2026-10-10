# 親子のノートを字下げして並べ、Z で畳む(NV-27)。
# 撮る: vhs demos/tree.tape → demos/out/demo-tree.gif

Output demos/out/demo-tree.gif
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

# 親子の区画(Tab 3回)でオンにして反映
Type "o" Sleep 800ms
Tab 3 Sleep 1.2s
Enter Sleep 1s
Shift+Tab Shift+Tab Shift+Tab Shift+Tab Shift+Tab Sleep 600ms
Enter Sleep 2s

# 先頭の Launch で Z → 畳む、もう一度 → 開く
Type "gg" Sleep 800ms
Type "Z" Sleep 1.5s
Type "Z" Sleep 1.5s
Type "q" Sleep 800ms
