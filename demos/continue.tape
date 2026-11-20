# 最後の表とビューへ戻る(resume.continue): ビューを移って終え、同じフォルダで mdgrid --continue と打つと、そのビューで開く。
# tmux の復元のプラグインには、この --continue を走らせてもらう(docs/manual/ja/tasks.md「窓を立て直したときに同じ表に戻る」)。
# 撮る: vhs demos/continue.tape → demos/out/demo-continue.gif

Output demos/out/demo-continue.gif
Set Shell bash
Set FontFamily "BIZ UDGothic"
Set FontSize 15
Set LineHeight 1.2
Set Width 1500
Set Height 860
Set TypingSpeed 60ms
Env LANG "en_US.UTF-8"
Env COLORTERM "truecolor"

Source demos/setup.tape

Type "mdgrid Tasks.base" Sleep 400ms Enter
Sleep 2s

# ビューを By status へ移って終える
Type "]" Sleep 800ms
Type "]" Sleep 1.5s
Type "q" Sleep 1s

# 同じフォルダで --continue → By status で開く
Type "mdgrid --continue" Sleep 600ms Enter
Sleep 2.5s
Type "q" Sleep 800ms
