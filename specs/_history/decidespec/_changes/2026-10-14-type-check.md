---
type: Change
id: 01M4X6D10TTGM69EXZ5QXKPH1D
title: 型と決まりに合わない値を「問題」として探し、画面と CLI で確かめられるようにする(type-check)
status: done
size: full
created: 2026-10-14
updated: 2026-10-15
---

# 型と決まりに合わない値を「問題」として探し、画面と CLI で確かめられるようにする(type-check)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [type-check](../_decisions/2026-10-14-type-check.md) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | この記録の「タスク」のコミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

人が会話 2026-10-14 で、mdgrid に型のチェックの機能を付けると決め、AI が示した3つの案(画面の中で型の不一致を探す・CLI で表の全体を確かめる・必須のキーと選べる値の決まりを設定に書く)を3つとも実装すると決めた。今は表のセルの型と合わない値に `!` が付く(CV-2)だけで、見えていない行の不一致を探せず、CI でノートの形を守る手段も無い。ブランチ type-check で進め、マージは人の許可を待つ。

終わりの条件:
- `[rules.status] required = true`・`values = ["todo", "done"]` → status の無いノートの空欄と `status: wip` に `!`。`!` で次の問題へ跳び、メッセージ行に理由。下の帯に「問題 N」。
- `mdgrid notes --check` → 問題を `パス:行: 列: 理由` で出し終了コード 1。問題が無ければ何も出さず 0。

関係する要件: CE-36・CV-9・NV-30・CLI-22(新しい)、CV-2・CE-2・CE-3・CE-35・SR-44・CLI-5・CLI-11・CLI-12。

## 不明点と仮定

- 仮定: 人が「オプションまで合わせて」と言ったのは、3つめの案(決まり)も含めて3つとも、の意味(直前に3つを並べたため)。外れたら: 決まり(CE-36)を外す。
- 仮定: 画面の問題の数・跳ぶ先・絞り込みは、ノートのキーの列(`file.*`・`formula.*` でない列)だけを見る。計算の列には型の印が今も無いため。
- 仮定: ためた値(未保存)のセルも、ためた値で問題を判定し、印を付ける(`*!wip`)。保存の前に気づけるため。
- 仮定: CLI の行の番号は Markdown のノートのキーの行だけに付ける(CSV などの読み込み口では付けない)。
- 仮定: 決まりの設定は設定ファイルにだけ書き、設定の画面の編集は今回入れない(列の型の窓のような画面は後で)。

## 設計

変える場所:
- 新しい `src/rules.rs`(lib): 決まり `Rule { required, values }`、範囲の層 `RuleLayer`(項目ごとに Option)、問題の理由 `Reason { Type(Kind), Required, NotAllowed }`、判定 `problem(kind, rule, value) -> Option<Reason>`(型 → 必須 → 選べる値の順で最初の1つ)、CLI 用の `check(src, rows, kind_of, rules) -> Vec<Found>`(行の番号は `frontmatter::key_lines`)。画面と CLI が同じ判定を使う。
- `src/profile.rs`: `Profile.rules`(列 → `RuleLayer`)の読み・書き、`Resolved.rules` に列ごと項目ごとに狭い範囲から重ねる(CE-35 の `[types]` と同じ形)。`file.*`・`formula.*` の列・知らない項目・型の違う値は警告。
- `src/schema.rs`: 項目 `rules` と `PROFILE_KEYS`。docs/config.md・docs/config.ja.md に節(CLI-12 の試験が突き合わせる)。
- 画面: `App::problem(row, col)`(ui/grid.rs。ノートのキーの列で、`prop` のためた値を重ねた値・`column_kind`・`resolved.rules`)。cell.rs の `!` の印を `problem` で付ける(型に合わない値の部品の扱いは今のまま)。nav.rs の `passes` に問題のある行だけの絞り込み(`only_problems`)、Esc と `--pick` の取りやめの条件にも足す。組み立て(grid.rs の `rebuild` の終わり)で問題の数を数えて持ち(描くたびに数えない)、下の帯の未保存の数の後ろに「問題 N」、そのクリックで絞り込みの切り替え。ヘッダーに絞っていることを出す(NV-8 の「同じ値」と同じ場所)。keymap.rs に `NextProblem`(`!`)・`PrevProblem`(`@`。`g !` は SR-16 の錠のある試験と前置きがぶつかるので変えた。check-kinds の後の提案 prev-problem-key)・`ProblemsOnly`(キーなし)、操作の一覧の列の節に。input.rs の候補のリストに `values` を足す。
- `src/main.rs`: `--check` の旗 → `Command::Check`。`--print` と同じく対象を開いてビューの行を得て(`print::table`)、`resolve_table` で `[types]` と `[rules]` を決め、`rules::check` の結果を出す。
- locales/*.toml の文言、docs/keys.md・docs/keys.ja.md の動作。

変えない場所: 書き戻し(決まりは書くことを止めない)、型の推定(CE-2)、`--print` の出力。

選んだ案と却下した案:
- 判定を lib の1か所に置き、画面と CLI が使う。却下: 画面の cell.rs の判定を CLI に写す(食い違いが生まれる)。
- 問題の数は組み立てのたびに数えて持つ。却下: 下の帯を描くたびに全部のセルを数える(1万行で重い)。
- CLI の列の型は画面と同じ `print::kind_in`(ノートのキーは `Source::kind`。読んだ全部のノートからの推定)に `[types]` を重ねる。CLI-22 の文言「確かめる行からの推定」とずれるので、照合で gap として扱い、文言の直しを提案にする。

要件ごとの対応: CE-36 → rules.rs・profile.rs・schema.rs・input.rs。CV-9 → cell.rs・rules.rs。NV-30 → keymap.rs・nav.rs・grid.rs・bands.rs・menu.rs。CLI-22 → main.rs・rules.rs。

検証: `cargo test`、`cargo clippy --all-targets -- -D warnings`、`cargo fmt --check`、`python3 .decidespec/scripts/check.py .`。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 判定と設定の読み取り | CE-36 | `[rules.status]` を読み、重ね、警告し、`problem` が3つの理由を返す | `src/rules.rs` `src/profile.rs` `src/schema.rs` docs/config*.md | test_ce_36_* | 済 |
| 2 | セルの印と候補 | CV-9, CE-36 | 決まりに合わないセルと必須の空欄に `!`、values が候補に | `src/ui/cell.rs` `src/ui/grid.rs` `src/ui/input.rs` | test_cv_9_*・test_ce_36_* | 済 |
| 3 | 跳ぶ・数・絞る | NV-30 | `!`・`@` で跳び、帯に「問題 N」、絞り込みの切り替え | `src/ui/keymap.rs` `src/ui/nav.rs` `src/ui/grid.rs` `src/ui/bands.rs` `src/ui/menu.rs` locales docs/keys*.md | test_nv_30_* | 済 |
| 4 | `--check` | CLI-23 | 問題を出して終了コード 1、無ければ 0、無いパスは 2 | `src/main.rs` `src/rules.rs` | test_cli_23_* | 済 |

## 実装の気づき

- 決まりの読み取り(タスク 1)は、テスト役の試験より先に書き始めたので、設定の試験は「テスト:」のコミットの時点で通っていた(先に落ちることを確かめたのは `--check` の3件だけ)。
- 錠を掛けたあとに `cargo fmt` を当てると関数の指紋が変わり E19 になる。試験を入れるコミットの前に整形する。今回は公開前のブランチの「テスト:」のコミットを、整形した形で作り直した。
- 既定のキー `g !` は、`g` を前置きにして SR-16 の錠のある試験とぶつかったので `@` にした(prev-problem-key)。
- `--check` の列の型を画面と同じ推定にし、`.md` の引数を `--print` と同じく扱うよう、CLI-22 の文を直した(check-kinds)。
- レビューの指摘で、問題の数を畳んだまとまりの外の行だけで数え(跳べる先とそろえる)、数・日付・日時の列でも values を候補にした。この2つの試験(`test_nv_30_folded_group_not_counted_or_jumped`・`test_ce_36_number_values_listed_as_numbers`)は、実装を見たメインが書いた(独立したテスト役ではない)。テスト役の `test_nv_30_prev_problem` は、キーを `@` に直した。

## 照合

検証役(フレッシュ文脈のサブエージェント、2026-10-15)が HEAD 8728451 と、「テスト:」のコミット c78193a の木(別の target)で確かめた。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CE-36 | checked | コード: src/rules.rs `Rule`・`RuleLayer::apply`・`problem`、src/profile.rs `read_rules`・`Resolved::apply`、src/ui/input.rs・src/ui/listpick.rs(values を候補に) / テスト: tests/test_rules_config.rs の test_ce_36_* 10本、src/ui/test_problems.rs の test_ce_36_* 2本 / 今: 通った / 前: c78193a で test_rules_config は組み立てで落ち(`mdgrid::rules` が無い)、画面の2本はアサーションで落ちた |
| CV-9 | checked | コード: src/ui/grid.rs `App::problem`、src/ui/cell.rs(`!`・必須の空欄・ためた値の `*!`)、src/rules.rs `Reason::text` / テスト: src/ui/test_problems.rs::test_cv_9_rule_violation_marked、test_cv_9_required_missing_key_marked、tests/test_rules_config.rs::test_cv_9_reason_texts_* / 今: 通った / 前: 画面の2本はアサーションで落ちた |
| NV-30 | checked | コード: src/ui/keymap.rs(`!`・`@`・`problems_only`)、src/ui/nav.rs `problem_step`・`passes`・`toggle_problems_only`、src/ui/bands.rs(下の帯の数・クリック・ヘッダー)、src/ui/grid.rs `build_slots`(見えている行で数える) / テスト: src/ui/test_problems.rs の test_nv_30_* 9本 / 今: 通った / 前: 8本はアサーションで落ち、`Built.problems` を使う1本は組み立てで落ちた |
| CLI-23 | checked | コード: src/main.rs `--check`・`check_rows`・`md_files`、src/rules.rs `check` / テスト: tests/test_check_cli.rs 4本、tests/test_check_cli_more.rs 2本 / 今: 通った / 前: c78193a で3本と test_check_cli_more の2本がアサーションで落ちた(test_cli_23_missing_path_exits_2 は前も通る) |

既存のテストの削除・skip・弱体化: なし(tests/golden/sr_5.txt はヘルプの行の数の1か所だけ)

確かめた: 4 / 4

`--check` の要件は、公開リポで先に CLI-22 がエージェントのスキルに使われたので、2026-10-15 に CLI-23 へ付け替えた(check-id。文と確かめ方は同じ)。この記録の「要求」と「設計」の CLI-22 は、付け替える前の番号。

指摘と行き先:
- 画面の試験の多くは、実装前の木では起動の前提の `assert!(warnings.is_empty())`(`rules` を知らない項目として警告する)で落ちる。振る舞いのアサーションの手前だが、`[rules]` を読めないこと自体が要件の欠けなので、落ちたと数えた。
- test_cli_23_missing_path_exits_2 は実装前でも通る(知らない旗でも終了コード 2)。ほかの5本で前後を見分けているので足さない。
- 決まりのある列をどのノートも持たないとき、画面ではその列が出ず数に入らないが、`--check` は違反として出す。画面は見えている列で数える(NV-30)ので要件どおり。必要なら列を足す(CE-28)と出る。
- 小数の値は `2.0` の形の文字で比べるので、`values = ["2"]` には合わない。CE-36 の「値の文字で比べる」どおり。
