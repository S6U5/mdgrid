---
type: Change
id: 01M48TJRHNCX7F50FHPWKHJ7XJ
title: .md を渡した --print は、渡したノートの行だけを出す(print-md-rows)
status: done
size: full
created: 2026-10-06
updated: 2026-10-06
---

# .md を渡した --print は、渡したノートの行だけを出す(print-md-rows)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | specs/_decisions/2026-10-06-print-md-rows.md(人の承認) |
| 設計 | 対象外: 条件に当たらない | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

採択した CLI-15 のとおり(人の判断待ちの表の5)。

## 不明点と仮定

- 仮定: `.md` のファイルが1つでも渡されたら、出す行はその `.md` のノートだけ(フォルダも一緒に渡したときも)。列はフォルダの表と同じ。画面と --pick と --apply は今までどおり(フォルダを開く)。

## 設計

対象外: 条件に当たらない

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 渡したノートの行だけ | CLI-15 | 採択した確かめ方 | src/main.rs, tests/test_open_md_file.rs, specs/test-locks.json, README.md, README.ja.md | test_cli_15_print_md_file_prints_its_row・test_cli_15_print_two_md_files・test_cli_15_print_md_file_extension_is_case_insensitive(tests/test_open_md_file.rs) | 済 |

## 実装の気づき

- CLI-15 はこの人の決定で守られる要件になり、tests/test_open_md_file.rs の試験に錠が要る(W13)。--print の2本は古い振る舞い(フォルダと同じ出力)を確かめていたので、錠をかける前に新しい振る舞いに書き直し、1本を足した。そのあと錠をかけた。

## 照合

自分で照合(行を絞るだけ)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CLI-15 | checked | コード: src/main.rs の Print の振り分けと print_view の only / テスト: tests/test_open_md_file.rs::test_cli_15_print_md_file_prints_its_row・test_cli_15_print_two_md_files・test_cli_15_print_md_file_extension_is_case_insensitive / 今: 通った(全体 1308 passed) / 前: 落ちた(実装の前に3本が落ちた) |

既存のテストの削除・skip・弱体化: なし(錠の無かった試験を新しい要件に書き直した)

確かめた: 1 / 1
