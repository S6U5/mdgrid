# 型ごとに値を入れて保存する(リスト・日付・数・チェック・タグ → 保存の確認)。
# 撮る: vhs demos/values.tape → demos/out/demo-values.gif

Output demos/out/demo-values.gif
Set Shell bash
Set FontFamily "BIZ UDGothic"
Set FontSize 16
Set LineHeight 1.25
Set Width 1280
Set Height 680
Set TypingSpeed 90ms
Env LANG "en_US.UTF-8"

Source demos/setup.tape

Type "mdgrid Tasks" Sleep 500ms Enter
# status(リスト): ほかのノートが使っている値から選ぶ
Enter Sleep 1.2s
Down Sleep 600ms
Enter Sleep 1s
# project(リンク): Projects/ のノートから名前で選ぶ(ファイルには [[名前]] で書く)
Right Enter Sleep 1.5s
Down Sleep 600ms
Enter Sleep 1s
# due(日付): カレンダーで1週間後の次の日へ
Right Right Enter Sleep 1.2s
Down Sleep 400ms Right Sleep 800ms
Enter Sleep 1s
# priority(数)
Right Enter Sleep 600ms
Backspace 3 Type "3" Sleep 600ms
Enter Sleep 1s
# billable(チェック): Enter で切り替え
Right Right Enter Sleep 1.2s
# tags(リスト): Space で候補の先頭を足す
Right Enter Sleep 1.5s
Space Sleep 800ms
Enter Sleep 1.2s
# 保存: 書く前に差分を見せる
Ctrl+S Sleep 3s
Enter Sleep 1.5s
Type "q" Sleep 800ms

Type "head -12 'Tasks/Audit the current bakery site.md'" Sleep 500ms Enter
Sleep 3s
