# 日付を言葉で入れる(cell-edit.date-words): 期限のセルで「next fri」「eom」「+2w」と打つ。打っている途中は欄の横に
# 「not a date」、言葉がそろうと読んだ日が出て、カレンダーがその日へ動く。Enter で決め、保存の確認で差分を見る。
# 日本語の「来週の金曜」「月末」も、画面の言語によらず読む(docs/manual/ja/tasks.md「日付を言葉で入れる」)。
# 撮る: vhs demos/date-words.tape → demos/out/demo-date-words.gif

Output demos/out/demo-date-words.gif
Set Shell bash
Set FontFamily "BIZ UDGothic"
Set FontSize 16
Set LineHeight 1.25
Set Width 1280
Set Height 680
Set TypingSpeed 120ms
Env LANG "en_US.UTF-8"

Source demos/setup.tape

Type "mdgrid Tasks" Sleep 500ms Enter
Sleep 1.5s

# due(日付)の列へ。今日は 2026-10-06(火)
Right Right Right Sleep 600ms

# 1行目: next fri(打っている途中は not a date、fri までそろうと 2026-10-16 (Fri))
Enter Sleep 1s
Type "next fr" Sleep 1.5s
Type "i" Sleep 2s
Enter Sleep 1s

# 2行目: eom(今月の最後の日)
Down Sleep 500ms
Enter Sleep 1s
Type "eom" Sleep 2s
Enter Sleep 1s

# 3行目: +2w(2週間後)
Down Sleep 500ms
Enter Sleep 1s
Type "+2w" Sleep 2s
Enter Sleep 1s

# 保存の確認で、書く前の差分を見る(書かずに戻る)
Ctrl+S Sleep 3s
Escape Sleep 1s
Type "q" Sleep 800ms
Type "d" Sleep 800ms
