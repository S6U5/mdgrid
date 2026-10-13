# 表の行を伸ばす設定(CV-8): `[display] row_extra_lines = 2` で、長い値と改行のある値を列の幅で折り返し、
# 行を最大3行まで伸ばす。短い値だけの行は1行のまま。
# 撮る: sh demos/record.sh row-lines

Output demos/out/demo-row-lines.gif
Set Shell bash
Set FontFamily "BIZ UDGothic"
Set FontSize 15
Set LineHeight 1.25
Set Width 1100
Set Height 620
Set TypingSpeed 80ms
Env LANG "en_US.UTF-8"
Env COLORTERM "truecolor"

Source demos/setup.tape

Hide
Type `printf -- '---\nstatus: todo\nsummary: "Collect feedback from the beta testers,\\nsort it by theme,\\nand turn the top themes into issues."\n---\n' > "Tasks/Sort beta feedback.md"`
Enter
Type `printf -- '---\nstatus: doing\nsummary: Draft the post about what is new, why it matters and how to try it in five minutes.\n---\n' > "Tasks/Draft the launch post.md"`
Enter
Type `printf '[display]\nrow_extra_lines = 2\n' > "$XDG_CONFIG_HOME/mdgrid/config.toml" && mkdir -p $ROOT/demos/out/row-lines && clear`
Enter
Show

Type "mdgrid Tasks" Sleep 300ms Enter
Sleep 2s
Screenshot demos/out/row-lines/1-table.png
Sleep 1s
Type "j" Sleep 600ms
Type "j" Sleep 600ms
Type "j" Sleep 1.5s
Type "q" Sleep 600ms
