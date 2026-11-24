# まとまりを値の候補の順に並べ、字と名前で見せる(navigation.group-values-order・navigation.group-values-icon):
# タスクの台帳(examples/tasks)の Open のビューは status でまとまり、todo・doing・waiting の順に並ぶ。設定の
# [rules.status] の icons と labels で、見出しとカンバンのレーンの題が「[ ] To do」のように出る。
# 撮る: vhs demos/group-values.tape → demos/out/demo-group-values.gif

Output demos/out/demo-group-values.gif
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

# 値の候補は台帳の印にある。字と名前だけを全体の設定に書く。録画の字体は ○ のような幅の曖昧な字を全角で描いて
# 隣の字に重なるので、ここでは ASCII の字にする(手元では Nerd Font の字でもよい)。
Hide
Type `printf '[rules.status]\nicons = { todo = "[ ]", doing = "[>]", waiting = "[~]", done = "[x]" }\nlabels = { todo = "To do", doing = "In progress", waiting = "Waiting" }\n' > "$XDG_CONFIG_HOME/mdgrid/config.toml"`
Enter
Type `cd "$DEMO/examples/tasks" && clear`
Enter
Sleep 300ms
Show

Type "mdgrid ." Sleep 400ms Enter
Sleep 2.5s

# 見出しは「Status: [ ] To do」から流れの順。B でカンバンにすると、レーンの題も同じ字と名前。
Type "j" Sleep 800ms
Type "j" Sleep 800ms
Type "B" Sleep 2.5s
Type "l" Sleep 800ms
Escape Sleep 1.5s
Type "q" Sleep 800ms
