# vim モード(screen.vim-*。設定の [keys] の vim = true): 探す欄で Esc・b・cw と直す、y と p で値を写し . で繰り返す、
# dd でノートをごみ箱へ動かして u で戻す、パレットの q! でためた変更を捨てて終わる。
# 撮る: vhs demos/vim.tape → demos/out/demo-vim.gif

Output demos/out/demo-vim.gif
Set Shell bash
Set FontFamily "BIZ UDGothic"
Set FontSize 16
Set LineHeight 1.25
Set Width 1280
Set Height 680
Set TypingSpeed 90ms
Env LANG "en_US.UTF-8"

Source demos/setup.tape

# vim モードを入れる(録画には映さない)
Hide
Type `printf '[keys]\nvim = true\n' > "$XDG_CONFIG_HOME/mdgrid/config.toml" && clear`
Enter
Sleep 300ms
Show

Type "cat $XDG_CONFIG_HOME/mdgrid/config.toml" Sleep 500ms Enter
Sleep 2s
Type "clear && mdgrid Tasks" Sleep 500ms Enter
Sleep 2s

# 探す欄は挿入のモードで始まる。打ち間違えた語を、Esc でノーマルのモードにして b で戻り、cw で替える
Type "/" Sleep 600ms
Type "bakery screens" Sleep 1s
Escape Sleep 1.2s
Type "b" Sleep 700ms
Type "cw" Sleep 700ms
Type "site" Sleep 1s
Enter Sleep 1.5s

# status を y で写し、下の行で p。続けて j . で同じ値を繰り返す
Type "0" Sleep 500ms
Type "y" Sleep 800ms
Type "j" Sleep 500ms
Type "p" Sleep 1.2s
Type "j" Sleep 500ms
Type "." Sleep 1.5s

# dd でノートをごみ箱へ(名前を確かめて y)、u で戻す
Type "dd" Sleep 1.5s
Type "y" Sleep 500ms
Enter Sleep 1.5s
Type "u" Sleep 1.5s

# パレットの q! で、ためた変更を捨てて終わる(ノートは書かない)
Type ":q!" Sleep 1s
Enter Sleep 1s
