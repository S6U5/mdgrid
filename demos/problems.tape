# 型と決まりの問題(CE-36・CV-9・NV-30): [rules.<列>] に必須と選べる値を書くと、型か決まりに合わない値に `!` が付き、
# 下の帯に数が出る。`!` で次の問題へ跳び、理由が下に出る。パレットの problems_only で問題のある行だけにする。
# 撮る: sh demos/record.sh problems

Output demos/out/demo-problems.gif
Set Shell bash
Set FontFamily "BIZ UDGothic"
Set FontSize 15
Set LineHeight 1.25
Set Width 1100
Set Height 620
Set TypingSpeed 80ms
Env LANG "en_US.UTF-8"
Env COLORTERM "truecolor"

Source demos/setup.tape

Hide
Type `printf '[rules.status]\nrequired = true\nvalues = ["todo", "doing"]\n' > "$XDG_CONFIG_HOME/mdgrid/config.toml" && mkdir -p $ROOT/demos/out/problems && clear`
Enter
Show

Type "mdgrid Tasks" Sleep 300ms Enter
Sleep 2s
Screenshot demos/out/problems/1-marks.png
# 次の問題へ(理由が下の行に出る)。
Type "!" Sleep 1.2s
Type "!" Sleep 1.2s
Screenshot demos/out/problems/2-jump.png
# 問題のある行だけ。
Type ":" Sleep 400ms
Type "problems_only" Sleep 600ms
Enter Sleep 1.5s
Screenshot demos/out/problems/3-only.png
Escape Sleep 800ms
Type "q" Sleep 600ms
Hide
# 同じ決まりを CLI で確かめる(CI 向け。あれば終了コード 1)。
Type `clear` Enter
Show
Type "mdgrid Tasks --check; echo exit=$?" Sleep 300ms Enter
Sleep 2s
Screenshot demos/out/problems/4-check.png
