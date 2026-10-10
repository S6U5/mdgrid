# CSV を台帳として開いて直す(直した値だけを書き換える・保存の前の差分・末尾に行を足す)。
# 撮る: vhs demos/csv.tape → demos/out/demo-csv.gif

Output demos/out/demo-csv.gif
Set Shell bash
Set FontFamily "BIZ UDGothic"
Set FontSize 16
Set LineHeight 1.25
Set Width 1280
Set Height 680
Set TypingSpeed 90ms
Env LANG "en_US.UTF-8"

Source demos/setup.tape

Type "cat equipment.csv" Sleep 500ms Enter
Sleep 2.5s
Type "clear && mdgrid equipment.csv" Sleep 500ms Enter
Sleep 2s

# qty を直す(数の列)
Type "jlll" Sleep 800ms
Enter Sleep 600ms
Type "4" Sleep 600ms
Enter Sleep 1s

# status を選び直す(候補のリスト)
Type "ll" Sleep 600ms
Enter Sleep 1s
Down Sleep 500ms
Enter Sleep 1s

# a で末尾に行を足し、id を打つ
Type "a" Sleep 1.2s
Type "A-006" Sleep 600ms
Enter Sleep 1s

# 保存の前の差分: 1つのファイルにまとめて、直した値だけ
Ctrl+S Sleep 3s
Enter Sleep 1.5s
Type "q" Sleep 800ms

Type "clear && cat equipment.csv" Sleep 500ms Enter
Sleep 3s
