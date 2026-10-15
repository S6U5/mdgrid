# 関係マップ(ER 図): 上の端のタブかキー R で開き、表とつながりを選ぶ(REL-7〜REL-9)。
# 撮る: vhs demos/relmap.tape → demos/out/demo-relmap.gif

Output demos/out/demo-relmap.gif
Set Shell bash
Set FontFamily "BIZ UDGothic"
Set FontSize 15
Set LineHeight 1.2
Set Width 1500
Set Height 860
Set TypingSpeed 80ms
Env LANG "en_US.UTF-8"
Env COLORTERM "truecolor"

Source demos/setup.tape


Type "mdgrid Tasks" Sleep 500ms Enter
Sleep 2s

# 右上のタブ(Table / Relations)。R で関係マップへ
Type "R" Sleep 3s

# ←→ でつながりを選ぶ(矢印・詳細・つながった行が変わる)
Right Sleep 1.5s
Right Sleep 1.5s

# ↑↓ で表を選ぶ
Down Sleep 1.5s
Right Sleep 1.5s
Down Sleep 1.5s

# R で表に戻る
Type "R" Sleep 1.5s
Type "q" Sleep 800ms
