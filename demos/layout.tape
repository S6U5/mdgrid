# 画面の配置の見本(SR-1): 普段の表・絞り込みの窓・絞った後・表の一覧の引き出しを、決まった場面で撮る。
# 配置を変えるときに、前と後を同じ操作で撮って並べるために使う(GIF のほかに場面ごとの PNG を demos/out/layout/ に)。
# 撮る: vhs demos/layout.tape

Output demos/out/demo-layout.gif
Set Shell bash
Set FontFamily "BIZ UDGothic"
Set FontSize 15
Set LineHeight 1.25
Set Width 1500
Set Height 860
Set TypingSpeed 80ms
Env LANG "ja_JP.UTF-8"
Env COLORTERM "truecolor"

Source demos/setup.tape

Hide
Type "mkdir -p $ROOT/demos/out/layout && clear"
Enter
Show

# 1 普段の表
Type "mdgrid Tasks" Sleep 300ms Enter
Sleep 2s
Screenshot demos/out/layout/1-table.png
Sleep 1s

# 2 絞り込みの窓(status の done を隠す)
Type "|" Sleep 1s
Enter Sleep 800ms
Enter Sleep 800ms
Enter Sleep 1s
Down Sleep 400ms
Down Sleep 400ms
Space Sleep 600ms
Enter Sleep 1.5s
Screenshot demos/out/layout/2-filter-window.png
Sleep 1s

# 3 絞って並べ替えた後(並べ替えは due)
Escape Sleep 500ms
Type "S" Sleep 600ms
Enter Sleep 500ms
Down Sleep 200ms
Enter Sleep 800ms
Escape Sleep 1s
Screenshot demos/out/layout/3-filtered.png
Sleep 1s
Type "q" Sleep 600ms

# 4 表の一覧の引き出し(印のワークスペースのある relations の tasks)
Type "cd ../relations && clear" Enter
Type "mdgrid tasks" Sleep 300ms Enter
Sleep 2s
Type "w" Sleep 1.2s
Screenshot demos/out/layout/4-drawer.png
Sleep 1s
Escape Sleep 300ms
Type "q" Sleep 600ms
