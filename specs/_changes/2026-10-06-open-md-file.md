---
type: Change
id: 01M46RCAJ25FZJE7DXDN6790DH
title: .md のファイルを渡したらそのフォルダを開いてその行を選ぶ(open-md-file)
status: done
size: full
created: 2026-10-06
updated: 2026-10-06
---

# .md のファイルを渡したらそのフォルダを開いてその行を選ぶ(open-md-file)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [open-md-file](../_decisions/2026-10-06-open-md-file.md)(AI) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

会話 2026-10-06 の改善の繰り返しで、初めて使う人の目の点検(docs/todo.md の U-12)が、`mdgrid x.md` が「フォルダでない」で止まると指摘した。CLI-15 を足した。

終わりの条件: CLI-15 の例のとおり。関係する要件: CLI-15・CLI-1・CLI-4・BV-1。

## 不明点と仮定

- 仮定: `.md`(大文字小文字を問わない拡張子)の、在るファイルだけをフォルダに置き換える。ほかのファイルは今のまま(フォルダでない、の理由)。
- 仮定: `--print`・`--pick` でも同じくフォルダに置き換える(選ぶのは画面のときだけ)。`--pick path` のパスの形は、置き換えたフォルダを起動の引数とみなす。

## 設計

- src/main.rs: 引数を読んだあと、`open_target` の前に、`.md` の在るファイルを親のフォルダに置き換え、置き換えたノートのパス(最初の1つ)を覚える(`fn md_to_folder`)。親が空(`x.md` だけ)なら `.`。無い `.md` は今のまま開く道に渡し、CLI-4 の理由1行にする。
- src/ui/app.rs: `App` に「読み込みが終わったら選ぶノート」(`select_after_load: Option<PathBuf>`)を持たせ、読み込みが終わったとき(行が揃ったとき)に、そのノートの行(`RowId` の実体のパスと比べる)を選ぶ。見つからなければ何もしない。
- 却下: 読み込みの途中で行が出たらすぐ選ぶ(並べ替えで行が動き、途中で選び直すと画面が跳ねる)。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | .md のファイルを渡す | CLI-15 | CLI-15 の例のとおり | src/main.rs, src/ui/app.rs, src/ui/startup.rs, tests/test_open_md_file.rs, README.md, README.ja.md, src/i18n.rs, docs/design.md | test_cli_15_*(tests/test_open_md_file.rs) | 済 |

## 実装の気づき

- `Startup` に項目を足すと錠のある試験(src/ui/test_*.rs ほか)の構造体の式が壊れるので、選ぶノートは `Startup` ではなく `App::select_after_load` に main が直に入れた(src/ui/startup.rs は変えていない)。

## 照合

自分で照合(フレッシュ文脈でない。試験は実装を見ていない別の役が先に書き、実装は別の役が入れた)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CLI-15 | checked | コード: src/main.rs の md_to_folder、src/ui/app.rs の select_after_load と select_pending / テスト: tests/test_open_md_file.rs::test_cli_15_*(6本。画面は疑似端末) / 今: 通った(全体 1108 passed) / 前: 落ちた(実装の前に回し、4本が「フォルダではない: notes/b.md」と時間切れで落ちた。無い .md と比べの2本は前から通る)。実物: `mdgrid "examples/demo/Tasks/Design a new logo.md" --print` が `mdgrid examples/demo/Tasks --print` と同じ出力。無い .md は理由1行と終了コード 2 |

既存のテストの削除・skip・弱体化: なし

確かめた: 1 / 1
