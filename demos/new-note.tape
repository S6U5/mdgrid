# 新しいノートをフォームで作る(雛形の値・必須の欄・隠して入れる作成日)。
# 撮る: vhs demos/new-note.tape → demos/out/new-note.gif

Output demos/out/new-note.gif
Set Shell bash
Set FontFamily "BIZ UDGothic"
Set FontSize 16
Set LineHeight 1.25
Set Width 1280
Set Height 680
Set TypingSpeed 90ms
Env LANG "ja_JP.UTF-8"

Source demos/setup.tape

# 新しいノートの決まり(録画には映さない)
Hide
Type `printf '[new_note]\nfolder = "タスク"\nask = ["status", "due", "priority"]\nrequired = ["due"]\nhidden = ["created"]\n\n[new_note.set]\nstatus = "todo"\ndue = "{date+7}"\ncreated = "{now}"\n' > "$XDG_CONFIG_HOME/mdgrid/config.toml" && clear`
Enter
Sleep 300ms
Show

Type "cat $XDG_CONFIG_HOME/mdgrid/config.toml" Sleep 500ms Enter
Sleep 3s
Type "clear && mdgrid ." Sleep 500ms Enter
Sleep 2s

# a でフォームの窓。status と due には雛形の値が入っている
Type "a" Sleep 1.5s
Type "牛乳を買う" Sleep 600ms
Enter Sleep 800ms
Enter Sleep 800ms

# due は必須: 空にして作ろうとすると、その欄に理由を出す
Backspace 12 Sleep 400ms
Ctrl+S Sleep 2s
Type "2026-10-15" Sleep 600ms
Enter Sleep 800ms
Type "2" Sleep 600ms

# Ctrl+S で作る(どの欄からでも)
Ctrl+S Sleep 2s
Type "q" Sleep 800ms

Type "cat タスク/牛乳を買う.md" Sleep 500ms Enter
Sleep 3s
