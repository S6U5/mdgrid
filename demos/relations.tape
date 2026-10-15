# リレーション: リンクを名前で見せ、行き先のノートから選び、行き先を開き、つながった行を出す(REL-2〜REL-5)。
# 撮る: vhs demos/relations.tape → demos/out/demo-relations.gif

Output demos/out/demo-relations.gif
Set Shell bash
Set FontFamily "BIZ UDGothic"
Set FontSize 16
Set LineHeight 1.25
Set Width 1280
Set Height 680
Set TypingSpeed 80ms
Env LANG "en_US.UTF-8"

Source demos/setup.tape


# リンクは行き先のノートの名前で出る(ファイルには [[Clinic Booking App]] のように書いてある)。5つの表は印(.mdgrid/workspace.toml)で1つのワークスペース
Type "mdgrid Tasks" Sleep 500ms Enter
Sleep 2.5s

# project のセルで Enter: Projects/ のノートが名前で並ぶ(選ぶと [[名前]] で書く)
Right Sleep 500ms
Enter Sleep 2s
Down Sleep 800ms
Escape Sleep 800ms

# 行き先を開く: Projects の表に移り、その案件の行が選ばれる
Type ":open_link" Sleep 800ms
Enter Sleep 3s

# つながった行: その案件を指すタスクと会議を、どの表のどの列からかと一緒に
Type ":linked_rows" Sleep 800ms
Enter Sleep 3.5s
Escape Sleep 500ms
Type "q" Sleep 800ms
