---
type: Change
id: 01M4A79M2JP85Y7N1C19T74073
title: 人と AI のタスクを分けて見る見本(example-ai-human)
status: done
size: full
created: 2026-10-07
updated: 2026-10-07
---

# 人と AI のタスクを分けて見る見本(example-ai-human)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 対象外: 見本と試験を足すだけで、要件も振る舞いも変えない(BV-1・BV-5・BV-7 の今の動きで作る) | なし |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

人の依頼(2026-10-07): cellops で作った「人のタスクと AI のタスクを分けて並べる」表の形を、mdgrid の見本の設定(フィクスチャー)として置き、こういうこともできると示す。

人の追加の依頼(同じ日): 設定でまとまりを組み替えられることはこの OSS の良いところなので、見本の画面を README に「カスタマイズできる」として画像付きで載せる。

終わりの条件: `examples/ai-human/` を開くと、人のタスクが上、AI のタスクが下のまとまりで並ぶ。分け方は cellops と同じ。見本は試験で開けることを確かめる。README(英・日)に、見本の「人とAI」のビューの画面の画像と、組み替えられることの説明がある。

## 不明点と仮定

- 仮定: 分け方は cellops(~/Projects/personal/private/cellops の docs/spec/behavior/controls.md)の「`human_gate` が none 以外か `actor == human` なら人、それ以外は AI」。`actor` が無いノートは AI(cellops の既定)。cellops の「未確認」は mdgrid に当たる概念が無いので入れない。
- 仮定: キーは cellops の task.schema.json の名前(`task_status`・`end_planned` など)ではなく、mdgrid の見本に合わせた短い名前(`status`・`due`・`priority`・`actor`・`human_gate`・`runner`)にする。
- 仮定: 見本の言葉は日本語(examples/vault・showcase と同じ)。

## 設計

- `examples/ai-human/vault/タスク/` に11個のノート。`examples/ai-human/vault/タスク.base` の式 `担当` で「👤 人間のタスク」「🤖 ロボットのタスク」を作り、ビュー「人とAI」で `groupBy: formula.担当 ASC`(👤 U+1F464 が 🤖 U+1F916 より前なので人が上)。並べ替えは優先度 → 期限(cellops の D11 の既定)。
- ビュー「人だけ」「AIだけ」「終わった」。
- `examples/ai-human/README.md` に開き方と alias の例。
- 試験は新しいファイル `tests/test_example_ai_human.rs`(錠のある tests/test_examples.rs は変えない)。実行ファイルの `--print --with-path --view` で、まとまりの順と、式の分け方を確かめる。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 見本のノート・.base・README と試験 | なし(見本) | 上の終わりの条件 | examples/ai-human/, tests/test_example_ai_human.rs, README.md, README.ja.md | test_example_ai_human_*(tests/test_example_ai_human.rs) | 済 |
| 2 | 見本の画面を撮って README に載せる | なし(文書) | README(英・日)に画像と説明と見本へのリンク | docs/manual-scenarios.toml, docs/assets/demo-ai-human.svg, docs/assets/ja/demo-ai-human.svg, README.md, README.ja.md | 読んで判定(画像を PNG にして見る) | 済 |

## 実装の気づき

- 見本の一番上に README.md を置くと、それも保管庫のノートに数えられ、左のノートの欄が全行に共通のフォルダ(`タスク/`)を省けない。showcase と同じく、ノートと `.base` を `vault/` に置いた。
- ビューの order に `file.name` を入れると、画面ではその列が `#名前.md`(計算の列の印と拡張子つき)になり、左のノートの欄も隠れる。order から外し、試験は `--with-path` のパスから名前を取る形にした。

## 照合

自分で照合(見本と試験を足すだけ)。

| 要件 | 結果 | 証拠 |
|---|---|---|

要件の無い変更(見本)の照合: checked。見本: examples/ai-human/(vault/ のノート 11・vault/タスク.base・config.toml・README.md) / テスト: tests/test_example_ai_human.rs の3本(人とAI のまとまりの順、人だけ・AIだけの分け方、終わったタスクの担当) / 今: 3 passed、全体 1331 passed / 前: ファイルが無い / 変異: 式 `担当` から `human_gate` の条件を外すと3本とも落ちる。`--config examples/ai-human/config.toml` で警告なく開ける(--print で確かめた)。タスク2: docs/manual-scenarios.toml の demo-ai-human を app-manual の tui_shot で英・日に撮り、PNG にして見た。見出し「👤 人間のタスク (5)」「🤖 ロボットのタスク (4)」の下に、左の欄が `.md` とフォルダなしの名前で並ぶ。README・README.ja の「もう少し詳しく」に画像と説明と見本へのリンクがある

既存のテストの削除・skip・弱体化: なし(錠のある tests/test_examples.rs は変えていない)

確かめた: 0 / 0(要件の無い変更。上の文のとおり checked)
