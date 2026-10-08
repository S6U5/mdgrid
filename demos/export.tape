# 見えている表をファイルに書き出す(絞った表 → CSV)と、画面なしの --print。
# 撮る: vhs demos/export.tape → demos/out/export.gif

Output demos/out/export.gif
Set Shell bash
Set FontFamily "BIZ UDGothic"
Set FontSize 16
Set LineHeight 1.25
Set Width 1280
Set Height 680
Set TypingSpeed 90ms
Env LANG "ja_JP.UTF-8"

Source demos/setup.tape

Type "mdgrid ." Sleep 500ms Enter
Sleep 2s

# \ で doing の行だけに絞る
Type "\doing" Sleep 600ms
Enter Sleep 1.5s

# パレットの「表をファイルに書き出す」。形は拡張子で決まる
Type ":表をファイルに書き出す" Sleep 800ms
Enter Sleep 1s
Type "doing.csv" Sleep 600ms
Enter Sleep 2s
Type "q" Sleep 800ms

# 先頭の列は各ノートのパス(--apply で戻せる)
Type "cat doing.csv" Sleep 500ms Enter
Sleep 3s

# 画面なしで出す
Type `clear && mdgrid --print --format md --filter 'status == "todo"' --sort due` Sleep 500ms Enter
Sleep 4s
