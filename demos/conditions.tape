# 名前を付けた条件で絞る(condition.digit-keys): 見本の設定に書いた「Due this week」を数字のキー 1 で入れ、もう一度で戻す。
# 式の中で cond("week") と呼べることと、mdgrid --count で数を外に出せることも見せる。
# 撮る: vhs demos/conditions.tape → demos/out/demo-conditions.gif

Output demos/out/demo-conditions.gif
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
Sleep 2s

# 1 で「Due this week」の行だけ、もう一度で全部
Type "1" Sleep 2s
Type "1" Sleep 1.5s
Type "q" Sleep 800ms

# 同じ条件の数を外に出す
Type `mdgrid --config ../configs/studio.toml Tasks.base --count --filter 'cond("week")'` Sleep 600ms Enter
Sleep 2s
