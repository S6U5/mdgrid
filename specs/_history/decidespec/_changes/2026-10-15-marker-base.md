---
type: Change
id: 01M4XY3DXV7KHG804270VR184F
title: 印の表に .base と同じ中身と where を TOML で書く(marker-base)
status: active
size: full
created: 2026-10-15
updated: 2026-10-15
---

# 印の表に .base と同じ中身と where を TOML で書く(marker-base)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [2026-10-15-marker-base](../_decisions/2026-10-15-marker-base.md)・[2026-10-15-marker-where](../_decisions/2026-10-15-marker-where.md) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 進行中 | (実装のコミット) |
| 照合 | 未 | この記録の「照合」 |

## 要求

台帳の見え方を `.base` のファイルではなく印 `.mdgrid/workspace.toml` で決める(会話 2026-10-15。決定 marker-base・marker-where)。印の `[[table]]` に `[table.base]` で `.base` と同じ中身を書き、状態などを `where = { 列 = [値…] }` の配列で絞れること。要件: WS-9・WS-10。

## 不明点と仮定

- 仮定: 中身を書いた表は、印の根に `<表の名前>.base` を置いたとみなす(ファイルは無い)。表の名前の `/` は `_` にする。`this` は印のファイル。外れたら: 見える名前や this の扱いを直す。
- 仮定: `where` の値は `list(列).contains(値)` の式に直す(スカラーもリストも同じ式で当たる)。

## 設計

- 変える場所: `src/workspace.rs`(`WsTable.base`、`read_tables` で `base` を JSON の文にし、`where` を `filters` に直す `inline_base_json`・`where_to_filters`、開くときに中身を引く `inline_base`、印の外を指す表を外す決まりから除く)、`src/main.rs`(`check_paths` と `open_target` で中身を引き、印の根で開き、this を印のファイルにする)、`src/source/markdown.rs`(`open_vault_dir`)、`locales/{en,ja}.toml`。
- 変えない場所: `.base` の読み取り(base.rs)。JSON は YAML として読めるので、そのまま `base::parse` に渡す。
- 検証: `cargo test`、`python3 .decidespec/scripts/check.py .`、台帳を移して画面で確かめる。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 印の base を読み、印の根で開く | WS-9 | `[table.base]` の表が .base と同じに開き、--print・--view・--check が効く | `src/workspace.rs` `src/main.rs` `src/source/markdown.rs` `src/ui/drawer.rs` `src/ui/places.rs` `src/ui/workspace.rs` `locales/en.toml` `locales/ja.toml` `cliff.toml` | test_ws_9_* | 済 |
| 2 | where を filters に直す | WS-10 | 値の配列のどれか、列どうしは全部、filters と両方、悪い値は警告 | `src/workspace.rs` `tests/test_where_formula.rs` `plugins/mdgrid/skills/mdgrid-views/SKILL.md` | test_ws_10_* | 済 |
| 3 | 受け入れの試験を置く | WS-9, WS-10 | 例が通る | `tests/test_marker_base.rs` `tests/test_marker_base_name.rs` `tests/test_marker_base_switch.rs` | test_ws_9_* / test_ws_10_* | 済 |

## 実装の気づき

- TOML では ASCII でないキー(`担当` など)は引用符で囲む(`"担当" = …`)。囲まないと印ごと読めない。
- 試験役の指摘: `base` が表でないとき、表ごと落として既定の表を開いていた。表でない中身も表として残し、開くときに .base の読めない中身と同じ理由を出すよう直した。
- 印の警告(`table[1].path`・`where.*`)は画面の起動の警告にだけ出て、`--print`・`--check` の標準エラーには出ない(印を読む既存の道と同じ)。
- 安全の見直しの指摘: 根に同じ名前の `.base`(リンクを含む)があると、中身を書いた表として印の外を指す決まりを素通りし、リンク先を読めた。同じ名前のファイルがあるときは中身を書いた表とみなさず、ほかの表と同じく外を指さないかを確かめるよう直した。
- 人の指摘: 引き出しでほかの表へ移ったあと、台帳(中身を印に書いた表)に戻れなかった。表を移る前に「パスが在るか」を見ていて、ファイルの無い表を「無い」と断っていた。在るか中身を書いた表かを見る `openable` にした(引き出し・登録した表・ワークスペースを開く、の3か所)。
- `where` の鍵の計算の列(`formula.期限` など)を `note["formula.期限"]` に直していて当たらなかった。`formula.`・`file.`・`note.` で始まる鍵はそのまま式に書くよう直した(台帳の期限のタブで、期限の区分を計算の列で1回定義して選ぶため)。
- この記録の直しのコミットを「直し:」で始め、cliff.toml の書き出しの決まりに当たらず tests/test_cliff.rs が落ちた。履歴は書き直さず、cliff.toml で「直し:」を修正と同じに扱う。以後は「修正:」で始める。
- 試験は実装のあとに独立の試験役が書く(人が台帳を急いで移したがっているため、実装を先にした)。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| WS-9 | unassessed | |
| WS-10 | unassessed | |

既存のテストの削除・skip・弱体化: なし

確かめた: 0 / 2
