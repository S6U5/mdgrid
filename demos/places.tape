# 登録した表を一覧から切り替えて開く・この表を登録する。
# 撮る: vhs demos/places.tape → demos/out/places.gif

Output demos/out/places.gif
Set Shell bash
Set FontFamily "BIZ UDGothic"
Set FontSize 16
Set LineHeight 1.25
Set Width 1280
Set Height 680
Set TypingSpeed 90ms
Env LANG "ja_JP.UTF-8"

Source demos/setup.tape

# 登録した表(録画には映さない)。パスは一時フォルダの見本
Hide
Type `E="$DEMO/examples"; printf '[[place]]\nname = "タスク"\ngroup = "仕事"\npath = "%s/vault/タスク.base"\nview = "進行中"\n\n[[place]]\nname = "予定"\ngroup = "仕事"\npath = "%s/schedule"\n\n[[place]]\nname = "人と AI"\ngroup = "見本"\npath = "%s/ai-human/vault"\n' "$E" "$E" "$E" > "$XDG_CONFIG_HOME/mdgrid/places.toml" && clear`
Enter
Sleep 300ms
Show

# 引数なしで起動すると、今のフォルダの表の上に、登録した表が分類ごとに出る
Type "mdgrid" Sleep 500ms Enter
Sleep 2.5s

# 打って絞り、Enter で開く(登録したビュー「進行中」で)
Type "タスク" Sleep 1s
Enter Sleep 2.5s

# 開いたままでも、パレットの「登録した表を開く」で切り替えられる
Type ":登録した表を開く" Sleep 800ms
Enter Sleep 1.2s
Type "予定" Sleep 1s
Enter Sleep 2.5s

# 今の表を登録する: 名前 → 分類(今ある分類から選ぶか、新しく打つ)
Type ":この表を登録" Sleep 800ms
Enter Sleep 1.5s
Backspace 12 Type "会議" Sleep 800ms
Enter Sleep 1.5s
Enter Sleep 2s

# 一覧に「仕事 › 会議」が増えた
Type ":登録した表を開く" Sleep 800ms
Enter Sleep 3s
Escape Sleep 500ms
Type "q" Sleep 1s
