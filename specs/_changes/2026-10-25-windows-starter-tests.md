---
type: Change
id: 01M5S6YTKM3NQ9A059Q14QN56Q
title: Windows でパスの区切りの違いで落ちるスターターの試験と相対パスを直す(windows-starter-tests)
status: active
size: bugfix
created: 2026-10-25
updated: 2026-10-25
---

# Windows でパスの区切りの違いで落ちるスターターの試験と相対パスを直す(windows-starter-tests)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。順番どおり(`specs/README.md` の `pace: sequential`)なら、前のフェーズが済か対象外になるまで、次を始めない。同時進行(`pace: concurrent` か無い)なら、要求を済にしたあと不明点から実装までを並べて進め、照合はそれらが済か対象外になってから。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [決定の記録](../_decisions/2026-10-25-windows-starter-tests-locks.md)(錠の掛け直し。AI 自己採択、錨との照合あり) |
| 設計 | 対象外: 補助の直しだけ | — |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | 「実装: [ST-1][ST-2][ST-3][ST-4][ST-5] …」のコミット |
| 照合 | 進行中 | この記録の「照合」 |

## 要求

公開リポの PR #45 の CI(windows-latest)で、スターターの単体の試験 9 件が落ちた(`test_st_1_*`・`test_st_2_*`・`test_st_4_*`)。落ち方はパスの区切り(`\` と `/`)の違いで、要件の振る舞いは Windows でも同じ。何を: (1) 試験の補助 `rels`・`files_under`・`file_rels`(`src/test_starter_unit.rs`)と `files_under`(`src/ui/test_starter.rs`)が返す相対パスの区切りを `/` にそろえる、(2) `src/ui/test_starter_more.rs` の絶対パスの例を OS の一時フォルダから作る(`/abs/x` は Windows では絶対でない)。核 `src/starter.rs` は変えない(`rel` を `/` の文字で持つ案は、Windows で出力のパスが `books\.mdgrid/…` の混ざった区切りになるので、錨との照合の検証役の指摘で取り下げた)。CLI・e2e・見本の同期の試験は `#![cfg(unix)]` で Windows では組み立てない。なぜ: Windows は対象(2026-10-15-windows-target)で、v0.4.0 は Windows の実行ファイルも配るため。

受け入れの条件: PR #45 の CI の windows-latest の試験が通る。macOS・ubuntu の試験は今までどおり通る(失敗時: 期待値を OS ごとに分けない)。

関係する要件: ST-1・ST-2・ST-3・ST-4・ST-5(錠の掛け直し。文は変えない。ST-3 は同じファイルの補助を使うので錠の値だけ変わる)、CLI-3・SR-8(Windows を対象にした前例)。

## 不明点と仮定

- 仮定: 試験が比べるのは補助が作る文字だけで、`f.rel == Path::new("…/…")` のような `Path` の比べ方は Windows でも `/` で効く(`components()` で比べる)。外れたら: その比べ方も補助に通す。
- 仮定: macOS の `test_wb_5_permission_revoked_after_pending_stops_save_and_keeps_changes` の失敗(同じ CI の macos-latest。一時フォルダへの書き込みが PermissionDenied)はこの変更と無関係で、再実行で消える。外れたら: 別の変更の記録で扱う。→ 再実行(run 40180006597)で通った。

## 設計

対象外: 補助の直しだけ。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 試験の補助の区切りを `/` にそろえ、絶対パスの例を OS から作る。錠を掛け直す | ST-1, ST-2, ST-3, ST-4, ST-5 | windows-latest の CI で test_st_* が通り、手元(macOS)でも全部通る | src/test_starter_unit.rs src/ui/test_starter.rs src/ui/test_starter_more.rs specs/test-locks.json | test_st_1_*・test_st_2_*・test_st_4_*(src/test_starter_unit.rs)、test_st_5_*(src/ui/test_starter.rs・src/ui/test_starter_more.rs) | 済 |

## 実装の気づき

- レビューに渡した版: 51ca5b4023242b99579a27d6fe7d6693a45cca75(`git stash create`。HEAD は a2a1d23)。
- ST-3 の試験 5 件は本文を変えていないが、同じファイルの補助の変更で錠の値が変わる(錠はファイルの補助も含めて数える)。決定の touches と `- 錠:` に ST-3 を足して掛け直した。
- 「実装:」の直前の全体の走り: `cargo test` 2068 件 0 失敗、57 分(19:20〜20:18。試験の実行は秒の単位で、残りは束のリンク)。clippy(`--all-targets -D warnings`)と `cargo fmt --check` も通った。
- 核 `src/starter.rs` の `rel` を `/` 区切りの文字で持つ案は一度書いたが、Windows で `root.join(rel).display()` が `books\.mdgrid/…` の混ざった区切りになるので戻し、補助だけの直しにした。

## レビューの指摘

実装のレビュー(書込なしのレビュー役。版 51ca5b4 と HEAD a2a1d23 の差分)の指摘を1行ずつ。区分は 重要 / 細部 / 前から、行き先は 直した / 間違い / 意図どおり / 見送り。

| # | 出どころ | 区分 | 箇所 | 指摘 | 行き先 |
|---|---|---|---|---|---|
| 1 | レビュー | 重要(照合の前提。差分の欠けではない) | 受け入れの条件 | CI の `cargo test --locked` は最初に落ちた実行ファイルで止まるので、run 40180006597 の windows-latest は lib の単体(328 通り 9 落ち)までしか回っておらず、bin の単体(src/ui/ の試験)と tests/ はこの PR の Windows でまだ一度も回っていない。ST-5 の 7 件は読んだ限り Windows で落ちる比べ方は無いが、受け入れの条件は次の CI で初めて分かり、PR #45 の他の変更(木の画面・タブの折り返しなど)が別の落ち方をしうる | 意図どおり(照合にこの前提を書き、書き出しのあとの CI で確かめる) |
| 2 | レビュー | 細部 | src/test_starter_unit.rs の `rels`・`files_under`・`file_rels`、src/ui/test_starter.rs の `files_under` | `replace('\\', "/")` は Unix で名前に `\` を含むファイルも `/` に読み替えるので、補助の出力だけでは「`\` を含む1つの名前」と「入れ子のフォルダ」の区別が消える(今の試験は `read`・`is_file`・`Path` の部品の比べでも同じ事実を確かめるので偽陽性で通ることは無い)。核の `rel_str` と同じく部品を `/` でつなぐ形にすれば OS 差を吸収しつつ見分けられる | 直した(単体は核の `rel_str` を使い、画面の試験は同じ形の `slash` を置いた) |
| 3 | レビュー | 細部 | src/ui/test_starter_more.rs の `files_under` | `into_owned()` のままで、ほかの2つの `files_under` と揃っていない(自分の出力どうしを比べるだけなので落ちない) | 直した(同じ `slash` を通す) |
| 4 | レビュー | 前から | src/ui/test_starter_more.rs の絶対パスの確かめ | `Path::join` は絶対パスを渡すと置き換えるので、`resolve_dir` に `is_absolute` の枝が無くても同じ値になり、この assert はその枝を見分けない(`/abs/x` のときからそう。要件の観測としては足りる) | 見送り(要件 ST-5 の「絶対はそのまま」の観測は足りている。枝の有無は実装の形で、仕様の外) |
| 5 | レビュー | 前から | src/starter.rs の `rel`(OS 生)と `rel_str`(`/`)の使い分け | 書く先・CLI の出力・画面は OS 生、警告の道筋と見本の `{folder}` は `/` で、1つの文字の中で混ざる所は無い。Windows で人が `/` で打った根は `books/sub\.mdgrid\…` と混ざるが、Rust の `join` の素の振る舞いで `--init-workspace` 単体と同じ | 意図どおり(動作に影響なし。核は変えない方針どおり) |

## 照合

(未)
