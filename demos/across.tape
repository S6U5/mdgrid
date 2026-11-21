# ワークスペースを横断して探す(navigation.across-*): 見本の設定の [search] across でパレットに出るコマンドで、
# studio のワークスペースの表の全部から bakery を探す。表ごとに当たった行が並び、Enter でその表がその行を選んで開く。
# 撮る: vhs demos/across.tape → demos/out/demo-across.gif

Output demos/out/demo-across.gif
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

Type "mdgrid --config ../configs/studio.toml Tasks.base" Sleep 400ms Enter
Sleep 2.5s

# パレットから横断の検索を選び、語を打って探す
Type ":" Sleep 600ms
Type "across" Sleep 800ms
Enter Sleep 800ms
Type "bakery" Sleep 600ms
Enter Sleep 2s

# 案件の行を選んで開く(検索の欄に bakery、その行を選んだ表)
Down Sleep 500ms
Down Sleep 500ms
Down Sleep 500ms
Down Sleep 500ms
Down Sleep 500ms
Down Sleep 500ms
Down Sleep 500ms
Sleep 600ms
Enter Sleep 2.5s
Type "q" Sleep 800ms
