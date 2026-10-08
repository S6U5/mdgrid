# 色のテーマ(設定の theme の1行。見本の設定は examples/themes/)。
# 撮る: vhs demos/themes.tape → demos/out/themes.gif

Output demos/out/themes.gif
Set Shell bash
Set FontFamily "BIZ UDGothic"
Set FontSize 16
Set LineHeight 1.25
Set Width 1280
Set Height 680
Set TypingSpeed 60ms
Env LANG "ja_JP.UTF-8"
Env COLORTERM "truecolor"

Source demos/setup.tape

Type "cat ../themes/nord.toml" Sleep 500ms Enter
Sleep 2.5s

Type "clear && mdgrid --config ../themes/nord.toml ." Sleep 300ms Enter
Sleep 2.5s
Type "q"
Type "clear && mdgrid --config ../themes/solarized-light.toml ." Sleep 300ms Enter
Sleep 2.5s
Type "q"
Type "clear && mdgrid --config ../themes/dracula.toml ." Sleep 300ms Enter
Sleep 2.5s
Type "q"
Type "clear && mdgrid --config ../themes/gruvbox.toml ." Sleep 300ms Enter
Sleep 2.5s
Type "q"
Type "clear && mdgrid --config ../themes/pink-monster.toml ." Sleep 300ms Enter
Sleep 2.5s
Type "q"
Type "clear && mdgrid --config ../themes/dozy-pink.toml ." Sleep 300ms Enter
Sleep 2.5s
Type "q"
Sleep 500ms
