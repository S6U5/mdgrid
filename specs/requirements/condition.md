# 名前を付けた条件(condition) の要求

## 目的

よく使う行の条件(期限が7日以内・始まりが30日より先 など)に名前を付けて設定に1回書き、式の中・数字のキー・ビューの設定・いつも隠す行・件数の出力・通知の欄(notice)から、同じ名前で使い回せるようにする。条件を表やビューごとに書き写さなくてよくし、1回書いた条件が画面の中にも外にも出るようにする。

要求を足す・変える・消すときは、PR で人がレビューして承認する(AI だけで要求を変えない)。要求の文の「しなければならない・するべき・してよい」の読み方は [README.md](README.md)。2026-11-21 に足した。

## 範囲外

- 採らない: 条件をワークスペースの印(workspace.marker)や `workspaces.toml` に書くこと(2026-11-21 に人が「設定だけ」と決めた)。
- 採らない: vim のような回数(`5j`)。数字のキーは条件に使う(理由は condition.digit-keys)。
- 採らない: 条件を `.base` に書き足すこと(mdgrid は `.base` を書き換えない。base-view.base-untouched)。
- 条件に当たる行の数を画面の上の欄に出すのは、通知の欄(notice)。

## 要求

### 条件を書く

#### 設定(cli.config-file-location)の `[conditions.<名前>]` に式 `expr`(base-view.expr-eval の式)を書いて、名前の付いた条件にできるべき
`req~condition.define~1` P1
- Rationale: よく使う絞り込みを1か所に書き、表やビューごとに書き写さなくてよくするため
- Acceptance:
  - Given: config.toml に `[conditions.soon]`・`expr = 'due <= today() + "7d"'`、今日が 2026-11-21
  - When:
  - Then: `mdgrid notes --print --filter 'cond("soon")'` は due が 2026-11-28 の行を出し、2026-11-29 の行を出さない。
- Needs: stest

#### 条件には、見せる名前 `label` を書いてよく、書かなければ条件の名前を見せるべき
`req~condition.label~1` P1
- Rationale: 設定の帯や通知の欄に、人が読んで分かる名前を出すため
- Acceptance:
  - Given:
  - When:
  - Then: soon に `label = "期限が近い"` → 数字のキーで入れたとき、設定の帯に「期限が近い」。`label` の無い far → 帯に「far」。

#### 条件の名前は英字で始まり英数字・`_`・`-` だけからなるものとし、それに外れる名前と、式として読めない `expr` は、cli.config-file-warning-form の形で警告してその条件だけを使わず、起動は止めないべき
`req~condition.invalid~1` P1
- Rationale: 1つの書き損じで表が開けなくならないようにし、どこが悪いかを人に伝えるため
- Acceptance:
  - Given: `[conditions."期限 近い"]` と、`expr = 'due <='` の `[conditions.broken]` と、正しい soon
  - When:
  - Then: 起動して警告が2つ出て表が開き、`cond("soon")` は使える。

#### 条件の式に今の表に無い列が出てきたら、その列を値の無い列として読み、エラーにしないべき
`req~condition.any-table~1` P1
- Rationale: 1つの条件を、列の違う表にもそのまま当てて使い回すため
- Acceptance:
  - Given:
  - When: due の列の無い表で `cond("soon")` で絞る
  - Then: エラーも警告も出ず、どの行も当たらない。

### 式の中で使う

#### 式の中の `cond("<名前>")` は、その行がその名前の条件に当たるかどうかの真偽になるべき
`req~condition.expr-call~1` P1
- Rationale: 名前の条件をほかの式と組み合わせる(期限が近く、まだ終わっていない、など)ため。保存した絞り込みを名前で式に入れる形は、JQL の `filter = "名前"` と同じ考え
- Acceptance:
  - Given: due が3日後で status が todo の行・due が3日後で done の行・due が20日後の行
  - When:
  - Then: `cond("soon") && status != "done"` は1つ目だけ、`!cond("soon")` は3つ目だけを残す。
- Needs: utest

#### `cond()` に書いていない名前を渡した式は、評価できない式(base-view.unsupported-no-guess)と同じに扱い、理由に無い名前とある名前を出すべき
`req~condition.unknown-name~1` P1
- Rationale: 名前の打ち間違いで、黙って行が消えたり全部出たりしないようにするため
- Acceptance:
  - Given:
  - When: soon と far を書いた設定で `mdgrid notes --print --filter 'cond("sooon")'`
  - Then: 「条件「sooon」はありません(ある名前: far, soon)」の1行と終了コード 2(cli.fail-fast)。

#### 条件の式が `cond()` をたどって自分に戻るときは、輪になった条件を警告し、輪の条件をどれも書いていない名前と同じに扱うべき
`req~condition.cycle~1` P1
- Rationale: 条件どうしの呼び合いで、評価が終わらなくならないようにするため
- Acceptance:
  - Given: a の `expr` が `cond("b")`、b の `expr` が `cond("a")`、c の `expr` が `cond("soon")`
  - When:
  - Then: 起動して a と b の輪の警告が1つ出て、`cond("a")` は無い名前の扱い、`cond("c")` は soon と同じ行に当たる。

### 数字のキー

#### 条件に数字のキー `key`(1〜9)を書いたら、表の画面でその数字を押すたびに、その条件に当たる行だけに絞るのと解くのを切り替えるべき
`req~condition.digit-keys~1` P1
- Rationale: よく使う絞り込みを1打で入れたり切ったりするため。数字だけのキーで切り替えるのは、同じ種類の端末の道具の慣わし(yazi はタブ、lazygit は区画、k9s はお気に入りの名前空間)で、どれも vim の回数を取らない。回数を取るのは vim をまるごとまねる道具(vifm・ranger)。mdgrid は表を見る道具なので前者に合わせる(2026-11-21 に人が「ベストを調べて」と任せた)
- Acceptance:
  - Given:
  - When:
  - Then: 5行のうち2行が soon に当たる表で、soon に `key = 1` → `1` で2行になり、もう一度 `1` で5行に戻る。どの条件にも書いていない `2` → 行は変わらない。
- Needs: stest

#### 数字のキーで入れた条件は、設定の帯(navigation.settings-bar-items)に数字と見せる名前で並べ、2つ以上入れたらどれにも当たる行だけにするべき
`req~condition.digit-stack~1` P1
- Rationale: 今どの条件で絞っているかを見て分かるようにし、ほかの絞り込み(検索と同じ値の絞り込み)と同じく重ねて狭めるため
- Acceptance:
  - Given:
  - When:
  - Then: soon(`key = 1`)と mine(`key = 2`)を入れる → 帯に「1 期限が近い」と「2 mine」が出て、両方に当たる行だけ。`1` をもう一度押す → 帯から「1 期限が近い」が消え、mine に当たる行だけ。

#### 数字のキーで入れた条件は、ビューを移っても入れたままにし、ほかの表へ移るか起動し直したら解くべき
`req~condition.digit-lifetime~1` P1
- Rationale: 同じ表のビューを見比べるあいだは同じ条件で見たいが、条件に合う列の無いほかの表にまで持ち越すと、黙って行が消えたように見えるため
- Acceptance:
  - Given:
  - When:
  - Then: `1` で soon を入れてビューのタブを「全部」へ → 「全部」でも soon で絞ったまま。引き出しでほかの表へ移る → 絞っていない。

#### 数字のキーとそれで入れ切りする条件の見せる名前を、ヘルプとコマンドのパレットに出すべき
`req~condition.digit-hints~1` P1
- Rationale: 番号を覚えなくても押せるようにするため(k9s がヘッダーに番号と名前空間を並べるのと同じ考え)
- Acceptance:
  - Given:
  - When:
  - Then: soon に `key = 1` → ヘルプに「1」と「期限が近い」の行。どの条件にも `key` が無い → ヘルプに数字の行が無い。

#### 2つの条件に同じ数字を書いたら、設定で先に書いた条件にその数字を当て、あとの条件の `key` を警告して使わないべき
`req~condition.digit-dup~1` P1
- Rationale: 1つのキーで2つの条件が切り替わる曖昧さを無くし、どちらが効くかを書いた順で決めるため
- Acceptance:
  - Given:
  - When:
  - Then: soon と mine の両方に `key = 1`(soon が先)→ 起動して mine の警告が出て、`1` は soon を切り替える。mine は `cond("mine")` では使える。

#### 数字のキー(1〜9)に別の動作を割り当て直した(screen.keys-vim-and-arrows-remap)ときは、割り当て直した方を効かせ、その数字を `key` に書いた条件を警告するべき
`req~condition.digit-remap~1` P1
- Rationale: 人が明示した割り当てを、条件の設定が黙って上書きしないため
- Acceptance:
  - Given: `[keys.table]` で `1` を別の動作に割り当て、soon に `key = 1`
  - When:
  - Then: `1` は割り当て直した動作をし、soon の `key` の警告が出る。

### ビューと、いつも隠す条件

#### ビューの設定の画面のフィルターの区画で、名前の条件を選び、当たる行だけを残すか、当たる行を隠すかを決められるべき
`req~condition.view-settings~1` P1
- Rationale: 決まったビューでいつも同じ名前の条件を掛けるのを、式を打たずに選んで済ませるため
- Acceptance:
  - Given:
  - When: `o` → フィルターの区画に書いた条件の並び(far・soon)→ far を「隠す」にして反映
  - Then: far に当たる行が消え、帯に「far を隠す」。navigation.settings-remember と同じく、同じビューで開き直しても掛かっている。

#### 条件に `hide = true` を書いたら、どの表でも、その条件に当たる行を初めから隠し、設定の帯にその条件の見せる名前と隠した行の数を見せるべき
`req~condition.hide-always~1` P1
- Rationale: 先すぎる予定のように、ふだんは見なくてよい行を、表ごとに書き足さずに隠すため(Taskwarrior が wait の日まで札を隠すのと同じ考え)。隠した数を見せるのは、行が黙って消えたと思わせないため
- Acceptance:
  - Given:
  - When:
  - Then: far に `expr = 'start > today() + "30d" && !(due <= today() + "7d")'`・`hide = true`、今日が 2026-11-21 → start が 2027-01-20 で due の無い行は出ず、start が 2027-01-20 で due が 2026-11-24 の行は出る。帯に「far を隠す」と隠した行の数。当たる行の無い表 → 帯に far の項目が出ない。
- Needs: stest

#### いつも隠す条件(condition.hide-always)に数字のキーを書いたら、そのキーで、隠すのを止めるのと戻すのを切り替えるべき
`req~condition.hide-toggle~1` P1
- Rationale: 隠した行を、設定を書き換えずにすぐ見られるようにするため
- Acceptance:
  - Given:
  - When:
  - Then: far に `hide = true`・`key = 9` → `9` で隠していた行が出て、帯の far の項目が「far を出す」に替わる。もう一度 `9` → また隠れる。表を移るか起動し直す → 隠した形に戻る(condition.digit-lifetime と同じ)。

#### いつも隠す条件は、表の画面のほか、ガント・カンバン・グラフ・木の画面と、`--print`・`--pick`・`--count` にも当てるべき
`req~condition.hide-everywhere~1` P1
- Rationale: 画面の型や出し方によって、同じ行が出たり出なかったりしないようにするため
- Acceptance:
  - Given: far を hide にした表
  - When:
  - Then: カンバンの札にも `--print` の行にも far の行が無い。

#### `--print`・`--pick`・`--count` に `--show-hidden` を添えたら、いつも隠す条件を当てないべき
`req~condition.show-hidden-cli~1` P1
- Rationale: 書き出して `--apply` で戻すときなど、スクリプトで全部の行が要る場面のため
- Acceptance:
  - Given:
  - When:
  - Then: far を hide にした表で `--print --show-hidden` → far に当たる行も出る。`--show-hidden` を `--print`・`--pick`・`--count` の無い起動に渡す → cli.fail-fast と同じく理由1行と終了コード 2。

### 件数を外に出す

#### `mdgrid [<.base | フォルダ> …] --count` は、画面を出さずに、表の行の数(ビューの絞り込み・いつも隠す条件・`--filter` を当てたあと)を数だけの1行で出し、終了コード 0 で終わるべき
`req~condition.count~1` P1
- Rationale: 条件に当たる行の数を、tmux の帯・シェルのプロンプト・Herdr のサイドバーなど、mdgrid の外に出すため
- Acceptance:
  - Given:
  - When:
  - Then: 5行のうち2行が soon に当たるフォルダで `mdgrid notes --count --filter 'cond("soon")'` → `2` の1行と終了コード 0。どの行も当たらない → `0`。`--view 進行中` を添える → そのビューの絞り込みのあとで数える。
- Needs: stest

#### `--count` は、`--print` と同じく `--view`・`-w`・`--filter` を受け、標準出力がパイプでも動き、読めない表や式には cli.fail-fast と同じく理由1行と終了コード 2 を返すべき
`req~condition.count-like-print~1` P1
- Rationale: 行を出す `--print` と数える `--count` で、表の選び方と失敗のしかたを揃えるため
- Acceptance:
  - Given:
  - When:
  - Then: `mdgrid notes --count | cat` → 数の1行。`mdgrid 無いフォルダ --count` → 理由1行と終了コード 2。`--count` と `--print` を一緒に渡す → 理由1行と終了コード 2。

## 未確定事項

### 式の中で名前の条件を呼ぶ書き方
`tbd~condition.call-form`
- Decider: 人
- Provisional: `cond("名前")`。名前をそのまま関数にする `soon()` は読みやすいが、`.base` の式の関数(base-view.expr-eval。Obsidian の Bases の関数と同じ並び)にあとから同じ名前が足されたときにぶつかるので採らない

### `--print` などでいつも隠す条件を当てない旗の名前
`tbd~condition.show-hidden-flag`
- Decider: 人
- Provisional: `--show-hidden`
