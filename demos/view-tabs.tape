# ビューのタブを設定の画面の「ビュー」の区画で整える: 隠す・順・既定・切り替えの案内(NV-26)。
# 撮る: vhs demos/view-tabs.tape → demos/out/demo-view-tabs.gif

Output demos/out/demo-view-tabs.gif
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

Type "mdgrid ." Sleep 400ms Enter
Sleep 2s

# 「名前を付けて保存」でビューを2つ作る(Shift+Tab 2回で下のボタン、↓3回で Save as)
Type "o" Sleep 800ms
Shift+Tab Shift+Tab Sleep 400ms
Down 3 Sleep 600ms
Enter Sleep 600ms
Type "Doing" Sleep 400ms Enter
Sleep 1.2s
Type "o" Sleep 800ms
Shift+Tab Shift+Tab Sleep 400ms
Down 3 Sleep 600ms
Enter Sleep 600ms
Type "Open" Sleep 400ms Enter
Sleep 1.5s

# ビューの区画(Tab 4回): Doing を隠す → すぐタブから消える
Type "o" Sleep 800ms
Tab 4 Sleep 1.2s
Down Sleep 500ms
Space Sleep 1.5s
# Open を上へ、既定にする
Down Sleep 500ms
Type "K" Sleep 1.2s
Enter Sleep 800ms
Enter Sleep 1.5s
# 切り替えの案内をオフ
Down 3 Sleep 500ms
Space Sleep 1.5s
Escape Sleep 1.5s

# タブの行: Doing は出ず、] でも移らない
Type "]" Sleep 1s
Type "]" Sleep 1.5s
Type "q" Sleep 800ms
