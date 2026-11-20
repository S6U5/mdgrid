---
type: Change
id: 01M48V1169WAE7E54ZF1DB0514
title: キーの名前の変更と削除を、全部のノートにまとめて行う(key-rename-delete)
status: done
size: full
created: 2026-10-06
updated: 2026-10-06
---

# キーの名前の変更と削除を、全部のノートにまとめて行う(key-rename-delete)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | specs/_decisions/2026-10-06-key-rename-delete.md(人の承認) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

採択した WB-1・WB-2・CE-29 のとおり(人の判断待ちの表の6)。

## 不明点と仮定

- 仮定: 対象は読み込んだ全部のノートのうち、そのキーを持つノート(値が null の `key:` も持つとみなす)。
- 仮定: 削除の確かめは、パレットの続きの入力で `y` と打って Enter(Esc でやめる)。
- 仮定: 同じセルにためた値の直しがあれば、名前の変更・削除に置き換わる(1つのセルに1つのためる変更)。

## 設計

- 値の種類 `NewValue::RenameKey(新しい名前)`・`NewValue::DeleteKey` を足し、セル(行, 古いキー)のためる変更として持つ(取り消し・印・差分は今の仕組み)。
- 書き戻し(writeback): 編集が RenameKey ならキーの行のキーの文字の範囲を `render_key` で置き換え、DeleteKey ならキーの行と値の続きの行を消す(続きの行の後ろの空行とコメントは残す)。範囲は frontmatter の `key_lines`(キーの文字の範囲と、消す行の範囲)。読み直し(WB-6)は、名前の変わったキーが元の値と形のまま新しい名前にあり、消したキーが無く、ほかのキーと本文が同じことを確かめる。
- 画面: 動作 `rename_key`・`delete_key`(既定のキーなし)をパレットと操作の一覧の列の節に。名前と確かめはパレットの続きの入力(Ask)。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 書き戻しの名前の変更と削除 | WB-1, WB-2 | 採択した確かめ方 | src/writeback.rs, src/frontmatter.rs, src/source.rs, src/changes.rs, src/newnote.rs, src/test_key_ops_unit.rs | test_wb_1_key_ops_*(src/test_key_ops_unit.rs) | 済 |
| 2 | 画面の操作 | CE-29 | 採択した確かめ方 | src/ui/keymap.rs, src/ui/menu.rs, src/ui/native_io.rs, src/ui/cell.rs, src/ui/grid.rs, src/ui/mod.rs, src/ui/app.rs, src/ui/external.rs, src/ui/input.rs, src/i18n.rs, src/ui/*.rs(NewValue の腕), src/ui/test_key_ops.rs, docs/keys.md, docs/keys.ja.md, README.md, tests/golden | test_ce_29_*(src/ui/test_key_ops.rs) | 済 |

## 実装の気づき

- 書込なしのレビュー役が、約80の形と 30 万のでたらめなフロントマターで、範囲の外のバイトが変わらないこと・本文が変わらないこと・落ちないことを確かめた。直したもの: 同じセルの直した値を名前の変更で黙って捨てていた(飛ばして理由)、アンカーを使われているキーを消すと読めない YAML になる(消さない)、大文字小文字だけ違う名前と重ねていた(重ねない)、全部のキーを消す削除と値の直しが一緒だと書けなかった(値の直しを先に)、ブロックの文字の `#` の行を消し残した(中身として消す)、ためた値の列との重なりを見ていなかった。
- ヘルプのゴールデン(tests/golden/sr_5.txt)は、パレットのコマンドが2つ増えて作り直した。パレットのコマンドの名前はヘルプの欄(幅 43)に収まる長さにした。

## 照合

フレッシュ文脈の書込なしのレビュー役と、自分の照合。

| 要件 | 結果 | 証拠 |
|---|---|---|
| WB-1 | checked | コード: src/writeback.rs の key_op・verify、src/frontmatter.rs の key_lines / テスト: src/test_key_ops_unit.rs の5本 / 今: 通った(全体 1317 passed) / 前: 落ちた(実装の前は組み立てられなかった) |
| WB-2 | checked | 読んで判定: 書き込むのはキーの名前と行(key_op)と値だけで、本文と他のファイルを書かない(verify が本文の同じを確かめる) |
| CE-29 | checked | コード: src/ui/native_io.rs の start_rename_key・start_delete_key・stage_key_op / テスト: src/ui/test_key_ops.rs の4本 / 今: 通った / 前: 落ちた(実装の前は動作が無かった) |

既存のテストの削除・skip・弱体化: なし

確かめた: 3 / 3
