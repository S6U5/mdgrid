# 縦に並べる(stack.*): Tasks を開き、パレットの「下に表を並べる」で Projects を下に並べる。Ctrl+w j・k で表を移り、
# 下の表だけを検索で絞る。Ctrl+w + で高さを変え、Ctrl+w c で閉じる。印のページ Morning を --page で開く。
# 撮る: vhs demos/stack.tape → demos/out/demo-stack.gif

Output demos/out/demo-stack.gif
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

Type "mdgrid Tasks.base" Sleep 400ms Enter
Sleep 2.5s

# 下に Projects を並べる
Type ":" Sleep 600ms
Type "stack_add" Sleep 800ms Enter
Sleep 1s
Type "Projects" Sleep 800ms Enter
Sleep 2.5s

# 上の表へ移って戻り、下の表だけを検索で絞る
Ctrl+W Type "k" Sleep 1.2s
Ctrl+W Type "j" Sleep 1.2s
Type "\" Sleep 400ms
Type "Bakery" Sleep 800ms Enter
Sleep 1.5s

# 下の表を高くして、閉じる
Ctrl+W Type "+" Sleep 600ms
Ctrl+W Type "+" Sleep 1.2s
Ctrl+W Type "c" Sleep 1.5s
Type "q" Sleep 800ms

# 印のページを開く
Type "mdgrid . --page Morning" Sleep 600ms Enter
Sleep 2.5s
Type "q" Sleep 800ms
