# 登録した表を一覧から切り替えて開く・この表を登録する。
# 撮る: vhs demos/places.tape → demos/out/demo-places.gif

Output demos/out/demo-places.gif
Set Shell bash
Set FontFamily "BIZ UDGothic"
Set FontSize 16
Set LineHeight 1.25
Set Width 1280
Set Height 680
Set TypingSpeed 90ms
Env LANG "en_US.UTF-8"

Source demos/setup.tape

# 登録した表(録画には映さない)。パスは一時フォルダの見本
Hide
Type `E="$DEMO/examples"; printf '[[place]]\nname = "Tasks"\ngroup = "Work"\npath = "%s/demo/Tasks.base"\nview = "Open"\n\n[[place]]\nname = "Schedule"\ngroup = "Work"\npath = "%s/schedule"\n\n[[place]]\nname = "Notes"\ngroup = "Personal"\npath = "%s/vault"\n' "$E" "$E" "$E" > "$XDG_CONFIG_HOME/mdgrid/places.toml" && clear`
Enter
Sleep 300ms
Show

# 引数なしで起動すると、今のフォルダの表の上に、登録した表が分類ごとに出る
Type "mdgrid" Sleep 500ms Enter
Sleep 2.5s

# 打って絞り、Enter で開く(登録したビュー Open で)
Type "tasks" Sleep 1s
Enter Sleep 2.5s

# 開いたままでも、パレットの open_place で切り替えられる
Type ":open_place" Sleep 800ms
Enter Sleep 1.2s
Type "schedule" Sleep 1s
Enter Sleep 2.5s

# 今の表を登録する(register_place): 名前 → 分類(今ある分類から選ぶか、新しく打つ)
Type ":register_place" Sleep 800ms
Enter Sleep 1.5s
Backspace 12 Type "Meetings" Sleep 800ms
Enter Sleep 1.5s
Enter Sleep 2s

# 一覧に Work › Meetings が増えた
Type ":open_place" Sleep 800ms
Enter Sleep 3s
Escape Sleep 500ms
Type "q" Sleep 1s
