# 絞り込みの窓: 検索の欄の右の「Filter」(または |)で開き、列の値で絞る・数で比べる・外す。すぐ効いてビューに覚える。
# 上の欄(\)は語で探す検索、/ は行を減らさない「探す」(NV-29・NV-1・NV-2)。
# 撮る: vhs demos/filter.tape → demos/out/demo-filter.gif

Output demos/out/demo-filter.gif
Set Shell bash
Set FontFamily "BIZ UDGothic"
Set FontSize 15
Set LineHeight 1.25
Set Width 1500
Set Height 860
Set TypingSpeed 60ms
Env LANG "en_US.UTF-8"
Env COLORTERM "truecolor"

Source demos/setup.tape

Type "mdgrid Tasks" Sleep 400ms Enter
Sleep 2.5s

# | で絞り込みの窓 → + Add a filter → status → Pick from the values
Type "|" Sleep 1.5s
Enter Sleep 1s
Enter Sleep 1s
Enter Sleep 2s
# 隠す方式のまま done にチェックして決める
Down Sleep 600ms
Down Sleep 600ms
Space Sleep 1s
Enter Sleep 2s

# priority ≥ 2 を足す(+ Add a filter → priority → Compare → ≥ → 2)
Down Sleep 600ms
Enter Sleep 1s
Down Sleep 400ms
Down Sleep 400ms
Down Sleep 400ms
Down Sleep 400ms
Enter Sleep 1s
Down Sleep 400ms
Down Sleep 400ms
Down Sleep 400ms
Enter Sleep 1s
Down Sleep 300ms
Down Sleep 300ms
Down Sleep 300ms
Down Sleep 300ms
Down Sleep 300ms
Enter Sleep 1s
Type "2" Sleep 600ms
Enter Sleep 2s
Escape Sleep 2s

# 上の欄の検索は別(語を含む行だけ)
Type "\" Sleep 600ms
Type "launch" Sleep 1.5s
Enter Sleep 2s
Escape Sleep 1s

# 窓で条件を外す
Type "|" Sleep 1.2s
Type "d" Sleep 1.2s
Type "d" Sleep 1.5s
Escape Sleep 1s
Type "q" Sleep 800ms
