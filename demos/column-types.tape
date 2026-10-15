# 列の型を選び直す(CE-35): 列の見出しの型の印をクリックするか、パレットの column_type で型の窓を開き、型を選ぶと
# views.toml の表に書いてすぐ効く(ノートは書き換えない)。「自動」で推定に戻る。
# 撮る: sh demos/record.sh column-types

Output demos/out/demo-column-types.gif
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
Type `printf '[look.style]\nicons = true\n' > "$XDG_CONFIG_HOME/mdgrid/config.toml" && mkdir -p $ROOT/demos/out/column-types && clear`
Enter
Show

Type "mdgrid Tasks" Sleep 300ms Enter
Sleep 2s
# billable の列へ(チェックの列)。パレットから型の窓を開く。
Type "l" Sleep 300ms
Type "l" Sleep 300ms
Type "l" Sleep 300ms
Type "l" Sleep 300ms
Type "l" Sleep 300ms
Type "l" Sleep 600ms
Type ":" Sleep 400ms
Type "column_type" Sleep 600ms
Enter Sleep 1.2s
Screenshot demos/out/column-types/1-menu.png
# テキストにする → 見出しの型の印が外れる。
Up Sleep 300ms
Up Sleep 300ms
Up Sleep 300ms
Up Sleep 300ms
Down Sleep 400ms
Enter Sleep 1.5s
Screenshot demos/out/column-types/2-text.png
# 自動に戻す。
Type ":" Sleep 400ms
Type "column_type" Sleep 600ms
Enter Sleep 600ms
Up Sleep 300ms
Up Sleep 300ms
Up Sleep 300ms
Up Sleep 300ms
Up Sleep 300ms
Up Sleep 300ms
Enter Sleep 1.5s
Type "q" Sleep 600ms
