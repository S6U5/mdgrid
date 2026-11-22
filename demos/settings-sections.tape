# 設定の画面の区画をキーで選ぶ: 番号のキーで1打で移り、h で左の一覧に入って j・k で区画を選び、l で項目へ戻る
# (navigation.settings-sections-keys・-digits・-focus)。
# 撮る: vhs demos/settings-sections.tape → demos/out/demo-settings-sections.gif

Output demos/out/demo-settings-sections.gif
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

Type "mdgrid Tasks" Sleep 400ms Enter
Sleep 2s

# 開くとフィルターの区画。左の一覧の番号で、並べ替え(3)・表示(6)へ1打で
Type "o" Sleep 1.5s
Type "3" Sleep 1.5s
Type "6" Sleep 1.5s
# h で左の一覧へ(印 > が左に移る)。k で区画を上へ、右もそれに合わせて替わる
Type "h" Sleep 1.5s
Type "k" Sleep 900ms
Type "k" Sleep 900ms
Type "k" Sleep 1.5s
# l で右の項目へ戻り(印 > が右に移る)、並べ替えを足す(列の一覧の先頭で並べる)
Type "l" Sleep 1.5s
Enter Sleep 1s
Enter Sleep 1.5s
# 反映せずに閉じる
Escape Sleep 1.2s
Type "q" Sleep 800ms
