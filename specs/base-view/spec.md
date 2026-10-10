---
type: Spec
id: 01M3S4GG11CMN4C40175QQMQWV
title: 表の組み立て(base-view)
description: 何を読んで、どの行とどの列の表を作るか。.base の table ビューの解釈、ビューの切り替え、解釈できないときの振る舞い。
status: active
load_when: 起動の引数・.base の読み込み・絞り込み・並べ替え・グループ分け・式の評価・ビューの切り替えを作る・変えるとき
created: 2026-09-30
updated: 2026-10-10
---

# 表の組み立て(base-view)

ノートのフォルダと `.base` の定義から、表の行(ノート)と列(プロパティ)を決める。`.base` には公開された文法が無く、Obsidian の版で意味が変わりうるので、互換は「table ビューのよく使う部分を正しく、それ以外は正直に未対応と出す」を方針にする(`.base` の table ビューの一部と互換)。セルの編集は [cell-edit](../cell-edit/spec.md)、画面の中での移動・検索・一時的な並べ替えは [navigation](../navigation/spec.md) にある。

強さの読み方: 「しなければならない」= 例外なし、「するべき」= 理由があれば外してよい、「してよい」= 任意。

## 要件: 版1 — 何を読むか

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|
| BV-1 | `.base` のファイルを渡されたら、その定義で表を作るべき。フォルダだけを渡されたら、その中の Markdown のノートを行、フロントマターのキーを列にした既定の表を作るべき。 | 設定も `.base` も無いフォルダを渡す → 全ノートと全キーの表(`test_BV_1`) | [2026-09-30](../_decisions/2026-09-30-v1-design.md) |
| BV-2 | ノートを探す範囲(保管庫の根)は、`.obsidian/` を持つ最寄りの上のフォルダとし、無ければ渡したフォルダにするべき。範囲は引数で1つ以上のフォルダを並べて広げてよい。 | 2つのリポのフォルダを並べて渡す → 両方のノートが1つの表に入る(`test_BV_2`) | [2026-09-30](../_decisions/2026-09-30-v1-design.md) |
| BV-3 | `.base` のファイルと、ノートの中の ````base` のブロックを書き換えてはならない。 | 表の操作の前後で `.base` のバイトが同じ(`test_BV_3`) | [2026-10-03](../_decisions/2026-10-03-strict-write-safety.md)、[2026-09-30](../_decisions/2026-09-30-v1-design.md) |
| BV-11 | 渡したフォルダの重なりやシンボリックリンクで同じノートが2度見つかったら、実体のパスで1行にするべき。 | 同じフォルダを2回と、そこを指すリンクを渡す → 行は1つ(`test_BV_11`) | [2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) |
| BV-12 | 保管庫の根が複数あって `.obsidian/types.json` の型が列で食い違うときは、その列をテキストとして扱い、読むだけにするべき。 | 根 A で date・根 B で text の `due` → テキストで表示し、編集できない理由を出す(`test_BV_12`) | [2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) |

## 要件: 版1 — 解釈

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|
| BV-4 | Obsidian のヘルプの Bases の構文どおり、`filters`(`and`・`or`・`not` の入れ子と式の文字列)を、全体とビューの両方で解釈し、両者を AND でつなぐべき。 | 全体 `file.hasTag("a")`・ビュー `status == "done"` → 両方を満たすノートだけ(`test_BV_4`) | [2026-09-30](../_decisions/2026-09-30-v1-full-spec.md)、[2026-09-30](../_decisions/2026-09-30-v1-design.md) |
| BV-5 | table ビューの `order`・`sort`・`groupBy`・`limit` と、`properties` の `displayName` を解釈するべき。 | `groupBy: {property: status, direction: ASC}` → status ごとのまとまりで並ぶ(`test_BV_5`) | [2026-09-30](../_decisions/2026-09-30-v1-design.md) |
| BV-24 | `properties` に displayName の無い列の見出しは、Obsidian の既定に合わせるべき: 式の列(`formula.x`)は式の名前(`x`)、ファイルの項目は `file.name` → `file name`・`file.ctime` → `created time`・`file.mtime` → `modified time`・`file.ext` → `file extension`、それ以外の `file.x` → `file x`、ノートのキーの列はキーの名前。displayName があればそれを使うべき。 | order に `file.name`・`formula.価格`・`status` を並べた `.base` → 見出しは `file name`・`価格`・`status`。`file.name` に displayName `名前` → `名前`。`--print --format json` の鍵も同じ見出し(`test_bv_24_*`) | [2026-10-07](../_decisions/2026-10-07-default-headings.md) |
| BV-6 | 式は、比較・論理・算術、`note.`・`file.`・`formula.` の参照、よく使う関数(`file.hasTag`・`file.inFolder`・`file.hasProperty`・`contains` 系・`if`・`date`・`now`・`today` と期間の足し引き)を評価するべき。 | 関数ごとに Obsidian での結果を写した小さな例で一致(`test_BV_6`) | [2026-09-30](../_decisions/2026-09-30-v1-design.md) |
| BV-7 | 評価できない式・関数・ビューの型に出会ったら、推測で行を作ってはならない。絞り込みが評価できないビューは開かずに理由を出し、評価できない列は、値の代わりに未対応の印(`?`)を出して読むだけにし(空・null・キーが無い の見せ方(CV-1)とも、本当の値の `?` とも見分けられるように、印は薄い表示にし、そのセルを選んだら下の帯に未対応と出す。色だけに頼らない(SR-15))、どの式が未対応かを画面に出さなければならない。 | 未実装の関数を使った filters → そのビューは開かず、関数名つきで未対応と表示。評価できない式の列 → 全行に薄い `?` が出て、キーが無いときの空欄と違う。値が `?` のテキストの列 → 普通の `?`。評価できない式のセルを選ぶ → 下の帯に未対応の式が出る(`TERM=dumb` で薄い表示が出なくても、選べば見分けられる)(`test_BV_7`) | [2026-10-03](../_decisions/2026-10-03-strict-write-safety.md)、[2026-10-01](../_decisions/2026-10-01-ime-keys-marks.md)、[2026-10-01](../_decisions/2026-10-01-edit-view-gaps.md)、[2026-09-30](../_decisions/2026-09-30-v1-design.md) |
| BV-21 | `.base` のうち mdgrid が解釈するもの(読む項目・ビューの型・演算子・関数とメソッド・`file.*` の値)と、解釈しないもの(BV-7 で未対応と出すもの)を、英語(`docs/obsidian-bases.md`)と日本語(`docs/obsidian-bases.ja.md`)の文書に並べるべき。文書の関数・メソッド・`file.*` の一覧と、式の評価が受け付ける一覧が食い違えば試験で落ちるべき(文書が古くなるのを防ぐため)。 | 式の評価に関数を1つ足して文書を直さない → 試験が落ちる。文書に評価できない関数を書く → 落ちる。文書の対応する関数を使った絞り込みは BV-7 の未対応にならない(`test_BV_21`) | [2026-10-03](../_decisions/2026-10-03-bases-docs.md) |
| BV-8 | `.base` の中の知らないキーは、エラーにせず無視し、消するべきでない。 | ビューに独自のキーを足した `.base` → 開ける(`test_BV_8`) | [2026-09-30](../_decisions/2026-09-30-v1-design.md) |
| BV-13 | `.base` の views が2つ以上あれば、ビューを切り替えられるべき。起動時は既定に選んだビュー(NV-25)を、無ければ先頭のビューを開くべき。 | ビューが3つの `.base` → タブに3つ、先頭が選ばれている。切り替えると行と列が変わる(`test_BV_13`) | [2026-10-10](../_decisions/2026-10-10-sort-views.md)、[2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) |
| BV-22 | 式は、ノートのリンクを評価するべき: `file.links`(そのノートの本文とフロントマターにある `[[名前]]`・`[[名前\|表示]]`・`[[名前#見出し]]`・`[文字](相対のパス.md)` の行き先のノートのリスト。行き先は Obsidian と同じく、保管庫の根からのパス、無ければ同じ名前のノート(同じ名前が複数なら根に近い・パスの短いもの)に解き、解けないリンクは文字のまま入れる)・`file.backlinks`(そのノートを行き先に持つノートのリスト)・`file.hasLink(x)`(x はノートの名前・パス・`this.file`)。`.base` を直接開いたときの `this` はその `.base` のファイルとし、`this.file.name`・`this.file.folder` などと `file.hasLink(this.file)` を評価するべき。 | a.md の本文に `[[b]]` と `[x](sub/c.md)` → a の file.links に b と sub/c。`[[無い]]` は文字 "無い" のまま入る。Projects.base を開き `file.inFolder(this.file.folder)` → .base と同じフォルダのノート。`file.links.contains(this.file)` → .base を指すノート。b.md を a と c が指す → b の file.backlinks に a と c。`file.hasLink("b")` → b を指すノートで真。`file.hasLink(this.file)` → .base を指すノート(`test_bv_22_*`) | [2026-10-07](../_decisions/2026-10-07-links-backlinks.md)、[2026-10-06](../_decisions/2026-10-06-links-this-narrow.md)、[2026-10-06](../_decisions/2026-10-06-links-this.md) |
| BV-23 | ノートを探すときは、名前が `.` で始まるフォルダ(Obsidian が保管庫に入れない隠しフォルダ)と `node_modules` の下を探さないべき(コードのリポを開いたときに依存のパッケージの文書を行にしないため)。渡したフォルダそのものが隠しフォルダでも、その中は探してよい。 | `a.md`・`.hidden/b.md`・`node_modules/p/README.md`・`sub/c.md` のフォルダを --print → a と sub/c の2行。`.hidden` を直接渡す → b の1行(`test_bv_23_*`) | [2026-10-06](../_decisions/2026-10-06-skip-hidden-dirs.md) |
| BV-14 | `.base` のビューの `summaries`(列の id → 集計の名前)のうち、Obsidian の組み込みの集計(Average・Min・Max・Sum・Range・Median・Stddev・Earliest・Latest・Checked・Unchecked・Empty・Filled・Unique)を、今のビューの行(絞り込みのあと)で計算し、表の下に1行で、集計した列の下に「集計の名前 値」を出すべき。集計の名前は英語と日本語で見せ、型の合わない値は数えないべき。式で書いた集計(最上位の `summaries` の formula)とまとまりごとの集計は解釈しないで、未対応の理由を出すべき。 | `summaries: {estimate: Sum, due: Earliest, done: Checked}` → 表の下に「Sum 18」「Earliest 2026-10-05」「Checked 2」。絞り込みで行が減ると値も変わる。`values.mean()` の集計 → 未対応の理由が出て、ビューは開く(`test_bv_14_*`) | [2026-10-06](../_decisions/2026-10-06-summaries.md)、[2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) |

## 要件: 版1 — 集まりの変化

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|
| BV-9 | ノートの変化は、ファイルの更新時刻と大きさを一定の間隔(既定 1 秒、設定で変えられる)で見て、変わったノートだけを読み直して表に反映するべき。ファイルの変更通知は、使うなら補助にとどめるべき。編集中・リストを開いている間は、読み直しを止めるべき。何もしていなくても CPU を使い続けないよう、ノートが 2,000 を超える保管庫では、1回の間隔で見るノートを 2,000 に限り、順に回して全部を見てよい。 | 外でノートを1つ直す → 次の間隔で、そのノートの行だけが変わる。編集中は変わらない(`test_BV_9`)。2,500 ノートの保管庫で後ろのほうのノートを直す → 2回の間隔のうちに変わる(`test_bv_9_round_robin_*`) | [2026-10-06](../_decisions/2026-10-06-poll-round-robin.md)、[2026-09-30](../_decisions/2026-09-30-v1-full-spec.md)、[2026-09-30](../_decisions/2026-09-30-v1-design.md) |
| BV-10 | 読み直したあとも、同じノートの行を選んだままにし、その行が消えたら最寄りの行を選ぶべき。 | 選んだ行のノートを消す → 下の行が選ばれる(`test_BV_10`) | [2026-09-30](../_decisions/2026-09-30-v1-design.md) |
| BV-16 | ノートが1万あっても、最初の画面は読み込みを待たずに出し、読み込み中は進捗を出し、読み込みの中止の操作(既定は Ctrl+G。Ctrl+C はコピー(OUT-1))で読み込みを止められるべき。 | 1万ノートの材料 → 1秒以内に画面が出て、進捗が増える。Ctrl+G → 読み込みが止まり、読んだ分の表が残る(`test_BV_16`) | [2026-10-01](../_decisions/2026-10-01-key-defaults.md)、[2026-09-30](../_decisions/2026-09-30-v1-review-fixes.md) |

## 要件: 版1 — mdgrid のビュー

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|
| BV-17 | `.base` を使わずに、開いた対象(フォルダか `.base`)ごとに、mdgrid 独自のビューを名前つきで複数持ち、ビューのタブで切り替えられるべきで、その定義は mdgrid の設定の置き場の、人が読み書きできる TOML に置き、ノートのフォルダには書かないべき。mdgrid のビューは、`.base` のビューより多くの機能を持ってよい。 | `.base` の無いフォルダで開き、設定の画面の「名前を付けて保存」でビューを2つ作る → タブに「既定の表」と2つの名前が並び、切り替えると絞り込みや列が変わる。終了して開き直すと同じ。設定の置き場の TOML を手で書き足す → タブが増える。ノートのフォルダに新しいファイルが無い(`test_BV_17`) | [2026-10-10](../_decisions/2026-10-10-settings-layout-examples.md) [2026-10-02](../_decisions/2026-10-02-native-views.md) |
| BV-18 | ビューの設定の画面(NV-13)から、今の見せ方を名前を付けて mdgrid のビューとして保存でき、今のビューへの上書き・名前の変更・削除ができるべき。 | `o` で done を外し「名前を付けて保存」で「未完了」→ タブに「未完了」が増える。別の絞り込みにして上書き → 開き直しても上書きした見せ方。名前を変える・消す → タブが変わる(`test_BV_18`) | [2026-10-02](../_decisions/2026-10-02-native-views.md) |
| BV-19 | mdgrid のビューを `.base` のビューとして書き出し、`.base` のビューを mdgrid のビューとして取り込めてよい。`.base` で表せない部分(または mdgrid で解釈できない部分)は落とし、何を落としたかを知らせてよい。書き出しは、利用者が名前を決めた新しい `.base` に書き、既存の `.base` を書き換えないべき(BV-3)。 | 絞り込みと並べ替えとグループのあるビューを書き出す → Obsidian で開ける `.base` ができ、同じ行が出る。表せない設定を持つビュー → 書き出しで落とした項目の一覧が出る。既存の `.base` の名前を選ぶ → 書かずに理由(読んで判定) | [2026-10-02](../_decisions/2026-10-02-native-views.md) |
| BV-20 | mdgrid のビューの定義は、設定の置き場の `views.toml` に、開いた対象の実体のパスごとに、ビューの名前・列の並びと隠す列・ビューの設定(NV-13〜NV-22 のフィルター・並べ替え・グループ)を持つべきで、知らない項目は警告にとどめ(CLI-3)、壊れた `views.toml` は警告して読まずに起動するべき。`.base` で開いたときのタブは `.base` のビューのあとに mdgrid のビューを並べ、起動時は BV-13 のとおり既定に選んだビューか先頭のビューを開くべき。保存は一時ファイル → 名前の変更で書き、`--readonly`(WB-15)では書かないべき。見た目の状態(SR-12)の列の幅と畳んだまとまりは、mdgrid のビューにも名前ごとに当てるべき。 | `views.toml` を手で壊す → 起動して警告、mdgrid のビューのタブは出ない。知らない項目 `color = 1` → 警告して読む。`.base` と mdgrid のビューの両方がある対象 → タブは `.base` のビューが先。`--readonly` で保存 → 何も書かれず「読むだけ」(`test_BV_20`) | [2026-10-10](../_decisions/2026-10-10-sort-views.md)、[2026-10-02](../_decisions/2026-10-02-native-views-details.md) |

## 要件: 次の段

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|

## 範囲外

- 次の段: ノートの中の ```` ```base ```` のブロックの解釈と、そのときの `this`(`.base` を直接開いたときの this は BV-22)。
- BV-15 は 2026-09-30 に削除([2026-09-30](../_decisions/2026-09-30-v1-review-fixes.md))。移行先: base-view の範囲外(次の段: `this` を使う式とノートの中の base のブロック)。
- 採らない: table 以外のビュー(cards・list・map・kanban)を表の見た目の切り替えで作ること(scope の SC-8・SC-9)。
- 採らない: `.base` への書き戻し(列の順・幅・並べ替えの保存を含む。BV-3)。見た目の状態は screen の SR-11・SR-12 で `.base` の外に持つ。
- 採らない: Dataview の inline フィールド・FLATTEN・GROUP BY の集計(Bases 自体が持たない)。

## 前提(AI の仮定)

- `.base` の形式と型の置き場所は 2026-09-30 の調査による(Obsidian 1.13.7〜1.14.3。公開の文法・JSON Schema は無く、`sort`・`columnSize`・`rowHeight` はヘルプに無いが実在)。Obsidian の版が上がって意味が変わったら、提案で直す。
- 関数の一覧(68個)のうち、版1で入れるのは BV-6 の範囲。

## 未決の問い(人の判断待ち。最大3)

- なし

<!-- decidespec:history:begin -->
<!-- この区画は check.py --write が書く。手で直さない。 -->

## この機能の決定

| 日付 | 記録 | 結果 | 決定者 | きっかけ | 触った要件 |
|---|---|---|---|---|---|
| 2026-10-10 | [並べ替えを選んで覚える(並べ替えの窓・見出しの並べ替えの保存・既定のビュー)(sort-views)](../_decisions/2026-10-10-sort-views.md) | accepted | 人 | 人の発言 | NV-3, NV-20, SR-12 ほか 4 |
| 2026-10-10 | [設定の画面の作り直しに合わせて、関わる要件の例(確かめ方)の操作を直す(settings-layout-examples)](../_decisions/2026-10-10-settings-layout-examples.md) | accepted | 人 | 人の発言 | NV-13, NV-16, NV-23 ほか 4 |
| 2026-10-09 | [モダンな見た目を既定にしたのに合わせて、色の試験の比べる元を今までの見た目にして錠を掛け直す(modern-look-locks)](../_decisions/2026-10-09-modern-look-locks.md) | accepted | 人 | 人の発言 | SR-20, SR-26, BV-19, NV-18 |
| 2026-10-09 | [窓の枠をつながった罫線にしたのに合わせて、枠の文字を読む試験の錠を掛け直す(modern-borders-locks)](../_decisions/2026-10-09-modern-borders-locks.md) | accepted | 人 | 人の発言 | CE-10, CE-20, CE-22 ほか 7 |
| 2026-10-07 | [左のノートの欄の名前に合わせて、画面の名前を見る試験の錠を掛け直す(note-column-locks)](../_decisions/2026-10-07-note-column-locks.md) | accepted | 人 | 人の発言 | SR-21, BV-17, CE-16 ほか 9 |
| 2026-10-07 | [フォームに合わせて、新しいノートの試験の錠を掛け直す(new-note-form-locks)](../_decisions/2026-10-07-new-note-form-locks.md) | accepted | 人 | 人の発言 | CE-25, CE-26, BV-17, CE-20, WB-2, SR-23 |
| 2026-10-07 | [file.backlinks と file.hasLink() を評価する(links-backlinks)](../_decisions/2026-10-07-links-backlinks.md) | accepted | 人 | 人の発言 | BV-22, BV-7, SR-23, WB-3, WB-5, CE-22 |
| 2026-10-07 | [見本の .base の未対応の式の例を file.embeds に替える(links-backlinks-sample)](../_decisions/2026-10-07-links-backlinks-sample.md) | accepted | 人 | 人の発言 | BV-3, BV-7 |
| 2026-10-07 | [group_gap に合わせて、表示の設定の試験の錠を掛け直す(group-gap-locks)](../_decisions/2026-10-07-group-gap-locks.md) | accepted | 人 | 人の発言 | SR-20, SR-21, NV-16 ほか 4 |
| 2026-10-07 | [displayName の無い列の見出しを Obsidian の既定にする(default-headings)](../_decisions/2026-10-07-default-headings.md) | accepted | 人 | 人の発言 | BV-24 |
| 2026-10-07 | [既定の見出しに合わせて、見出しを鍵に使う試験の錠を掛け直す(default-headings-locks)](../_decisions/2026-10-07-default-headings-locks.md) | accepted | 人 | 人の発言 | BV-22, BV-7 |
| 2026-10-06 | [.base の組み込みの集計(summaries)を版1に入れる(summaries)](../_decisions/2026-10-06-summaries.md) | accepted | AI | 検証の指摘 | BV-14 |
| 2026-10-06 | [隠しフォルダと node_modules の下のノートを探さない(skip-hidden-dirs)](../_decisions/2026-10-06-skip-hidden-dirs.md) | accepted | AI | 検証の指摘 | BV-23 |
| 2026-10-06 | [大きな保管庫では1回の見回りで見るノートを限って順に回す(poll-round-robin)](../_decisions/2026-10-06-poll-round-robin.md) | accepted | AI | 検証の指摘 | BV-9 |
| 2026-10-06 | [リンク・被リンク・.base を開いたときの this を評価する(links-this)](../_decisions/2026-10-06-links-this.md) | accepted | AI | 検証の指摘 | BV-22 |
| 2026-10-06 | [BV-22 から file.backlinks・file.hasLink を外す(links-this-narrow)](../_decisions/2026-10-06-links-this-narrow.md) | accepted | AI | 検証の指摘 | BV-22 |
| 2026-10-03 | [書き戻しの安全を「しなければならない」に上げる(strict-write-safety)](../_decisions/2026-10-03-strict-write-safety.md) | accepted | 人 | 人の発言 | WB-1, WB-2, WB-4 ほか 6 |
| 2026-10-03 | [.base の対応範囲の文書(bases-docs)](../_decisions/2026-10-03-bases-docs.md) | accepted | AI | AI の提案 | BV-21 |
| 2026-10-02 | [mdgrid 独自のビューの定義(native-views)](../_decisions/2026-10-02-native-views.md) | accepted | 人 | 人の発言 | BV-17, BV-18, BV-19 |
| 2026-10-02 | [mdgrid のビューの定義の細部(native-views-details)](../_decisions/2026-10-02-native-views-details.md) | accepted | AI | AI の提案 | BV-20 |
| 2026-10-01 | [既定のキーのぶつかりと抜けを決める(key-defaults)](../_decisions/2026-10-01-key-defaults.md) | accepted | AI | 検証の指摘 | SR-16, WB-10, BV-16, SR-18 |
| 2026-10-01 | [日本語入力の読点と中黒、キーの無いセルの Backspace、未対応の印と本当の値の見分け(ime-keys-marks)](../_decisions/2026-10-01-ime-keys-marks.md) | accepted | AI | 検証の指摘 | SR-17, SR-18, BV-7 |
| 2026-10-01 | [見せ方と編集の食い違いと抜けを直す(edit-view-gaps)](../_decisions/2026-10-01-edit-view-gaps.md) | accepted | AI | 検証の指摘 | SR-10, BV-7, CE-3, CE-5, CE-8, CE-10 |
| 2026-09-30 | [仕様の点検の指摘を直す(v1-review-fixes)](../_decisions/2026-09-30-v1-review-fixes.md) | accepted | AI | 検証の指摘 | SR-13, SR-10, SR-12 ほか 18 |
| 2026-09-30 | [最初の版の仕様を、表の TUI の調査に合わせて書き切る(v1-full-spec)](../_decisions/2026-09-30-v1-full-spec.md) | accepted | AI | 人の発言 | BV-4, BV-9, CE-1 ほか 52 |
| 2026-09-30 | [最初の版の設計(v1-design)](../_decisions/2026-09-30-v1-design.md) | accepted | AI | AI の提案 | BV-1, BV-2, BV-3 ほか 35 |
<!-- decidespec:history:end -->
