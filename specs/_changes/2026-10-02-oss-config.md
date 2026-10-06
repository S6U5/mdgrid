---
type: Change
id: 01M3XZNQD2SC7KCVCK3PSKKJ3G
title: エディタの設定・設定の文書と書き出し・ライセンス(oss-config)
status: done
size: full
created: 2026-10-02
updated: 2026-10-02
---

# エディタの設定・設定の文書と書き出し・ライセンス(oss-config)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [editor-setting](../_decisions/2026-10-02-editor-setting.md)(人)、[config-docs](../_decisions/2026-10-02-config-docs.md)(人) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

依頼の要約(会話 2026-10-02): OSS なので、設定ファイルの管理や誰かに使ってもらうための工夫が要る。編集のときのエディタも設定で変えたい。選んだもの: ライセンスは MIT と Apache-2.0 の両方、文書は英語を主に日本語も置く、始める作業は editor の設定と、設定の文書と書き出し。

終わりの条件: 設定の `editor` が効き(設定 > $VISUAL > $EDITOR > vi)、`mdgrid --print-config` が全項目を既定値とコメント付きで出し、docs/config.md と docs/config.ja.md が全項目を説明し、文書・実装・書き出しのずれを試験が落とす。LICENSE-MIT・LICENSE-APACHE と Cargo.toml の license の欄がある。関係する要件: SR-8・CLI-2・CLI-3・CLI-11・CLI-12。

## 不明点と仮定

- 仮定: ライセンスの著作権者の名前は git の作者名(S6U5)。外れたら: 人が名前を決める。
- 仮定: ライセンスの本文は crates.io の serde の LICENSE-MIT・LICENSE-APACHE と同じ標準の本文を写し、MIT の著作権の行だけ変える。
- 仮定: 公開(リモートの作成・push)はしない(SC-10)。README の英語化・入れ方・CI・CONTRIBUTING は次の変更。
- 仮定: `--print-config` は CLI-2 の起動の引数に足す(ヘルプにも出す)。

## 設計

設定の項目の一覧を1か所(src/config.rs の項目の表: 名前・型・既定値・英語の説明・日本語の説明)に持ち、設定の読み取りの「知らない項目」の判定、`--print-config` の出力、試験の文書との突き合わせが、この表を使う。`--print-config` は表から英語の説明のコメント付きの TOML を作る(キーの割り当ての表 [keys.*] は既定の割り当てを全部は出さず、書き方の例をコメントで出す)。試験は docs/config.md と docs/config.ja.md の項目の見出し(`### \`name\``)と表の名前を突き合わせ、出力を Config として読み直して警告なし・既定と同じを確かめる。エディタは Config の `editor: Option<String>` を足し、main で 設定 → VISUAL → EDITOR の順に決めた文字列を画面に渡す(今の editor_argv の空なら vi はそのまま)。ライセンスは LICENSE-MIT・LICENSE-APACHE と Cargo.toml の `license = "MIT OR Apache-2.0"`。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | editor の設定 | SR-8 | 設定 > VISUAL > EDITOR > vi、空は無いのと同じ | src/config.rs, src/main.rs, src/ui/external.rs, src/ui/app.rs, src/ui/keymap.rs, src/ui/input.rs, src/ui/test_keymap.rs, src/ui/test_click_name.rs, tests, tests/golden, src/ui/test_external.rs | test_SR_8 | 済 |
| 2 | 設定の項目の表・--print-config・文書・ずれの試験 | CLI-11, CLI-12 | 出力が読めて既定と同じ、全項目に説明、文書2つ、ずれで落ちる | src/config.rs, src/main.rs, src/test_config_unit.rs, src/test_main.rs, docs/config.md, docs/config.ja.md, tests/test_config_docs.rs, tests/test_config_docs_values.rs, README.md, examples/showcase/config.toml, docs/design.md | test_CLI_11, test_CLI_12 | 済 |
| 3 | ライセンス | なし | LICENSE-MIT・LICENSE-APACHE、Cargo.toml の license・description・repository の欄(repository は公開前なので空のまま) | LICENSE-MIT, LICENSE-APACHE, Cargo.toml | なし(読んで判定) | 済 |

受け入れの試験は実装を見ていない役が先に書く(1・2)。

## 実装の気づき

- タスク3: Cargo.toml には license = "MIT OR Apache-2.0" と description が既にあった(publish = false のまま)。LICENSE-APACHE は serde 1.0.228 のものを写し、LICENSE-MIT は同じ本文に「Copyright (c) 2026 S6U5」の行を足した。
- タスク1・2: 項目の表は config::ITEMS(KEYS は const の中で名前を写す)。editor と keys は既定の値が無いので --print-config では例をコメントで出し、`editor = ""` は読んだ時点で None にして既定と等しくした。キーの表のモード名は ui(バイナリ側)にあるので、keys の説明に全モードがあるかを ui の単体の試験で確かめる。画面の `e` の表示名「$EDITOR で開く」と誤りの文「$EDITOR を読めない」は既存の試験が見ているので変えていない。
- タスク1・2(差し戻しの直し): 画面の文言は「エディタで開く」「エディタの設定を読めない」に変えた(上の行の「変えていない」は差し戻しで変わった)。文書の型・既定・例は ITEMS と突き合わせる試験(tests/test_config_docs_values.rs)を足した。

## 照合

書込なしのフレッシュ文脈の検証役で照合した(2026-10-02)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| SR-8 | checked | コード: src/config.rs:132 / テスト: tests/test_editor_setting.rs::test_sr_8_config_editor_wins_over_env / 今: 通った(cargo test) / 前: 落ちた(変異: 順を入れ替える・空を飛ばさない・vi の既定を消す → それぞれ落ちた。eadf4b0 では resolve_editor が無くコンパイルで落ちる) |
| CLI-11 | checked | コード: src/config.rs:284 / テスト: tests/test_config_docs.rs::test_cli_11_output_reads_back_without_warnings_as_default / 今: 通った(cargo test --test test_config_docs) / 前: 落ちた(変異: 1項目を出さない・既定値を変える・コメントを消す → 落ちた) |
| CLI-12 | checked | コード: src/config.rs:164 / テスト: tests/test_config_docs.rs::test_cli_12_english_doc_matches_keys / 今: 通った(cargo test) / 前: 落ちた(変異: 文書に足さず項目を足す(仕様の例そのもの)・文書の既定値を変える → 落ちた) |
| CLI-2 | checked | コード: src/main.rs:24 / テスト: tests/test_config_docs.rs::test_cli_2_help_mentions_print_config / 今: 通った(cargo test) / 前: 落ちた(変異: USAGE から消す・解析の分岐を消す → 落ちた) |

既存のテストの削除・skip・弱体化: なし(test_external と golden の sr_5 はエディタの文言の変更に合わせただけ)

gap の行き先: main で決めたエディタと設定(add_frontmatter も)を画面と読み込み口へ渡す配線を見る自動の試験が無い(検証役が疑似端末で確かめた)。本物のバイナリを動かす e2e の試験として docs/todo.md の Q-10 に足した。

確かめた: 4 / 4
