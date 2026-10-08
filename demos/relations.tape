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

# 見本 examples/relations の3つのフォルダを登録した表にする(録画には映さない)
Hide
Type `cd ../relations && R="$PWD" && printf '[[place]]\nname = "Tasks"\npath = "%s/tasks"\n\n[[place]]\nname = "Projects"\npath = "%s/projects"\n\n[[place]]\nname = "Members"\npath = "%s/members"\n' "$R" "$R" "$R" > "$XDG_CONFIG_HOME/mdgrid/places.toml" && clear`
Enter
Sleep 300ms
Show

# リンクは行き先のノートの名前で出る(ファイルには [[mdgrid]] と書いてある)
Type "mdgrid tasks" Sleep 500ms Enter
Sleep 2.5s

# project のセルで Enter: projects/ のノートが名前で並ぶ(選ぶと [[名前]] で書く)
Right Sleep 500ms
Enter Sleep 2s
Down Sleep 800ms
Escape Sleep 800ms

# 行き先を開く: projects の表に移り、mdgrid の行が選ばれる
Type ":open_link" Sleep 800ms
Enter Sleep 3s

# つながった行: mdgrid を指すノートを、どの表のどの列からかと一緒に
Type ":linked_rows" Sleep 800ms
Enter Sleep 3.5s
Escape Sleep 500ms
Type "q" Sleep 800ms
