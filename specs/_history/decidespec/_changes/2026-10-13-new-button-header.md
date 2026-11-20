---
type: Change
id: 01M4S6R4ERPDK1WR6DSGYX1HX2
title: ビューの帯を出さないときは「+ 新規」をヘッダーに出す
status: done
size: full
created: 2026-10-13
updated: 2026-10-13
---

# ビューの帯を出さないときは「+ 新規」をヘッダーに出す

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [new-button-header](../_decisions/2026-10-13-new-button-header.md) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | |
| 照合 | 済 | この記録の「照合」 |

## 要求

header-menu の照合(2026-10-12)の任意の補足: ビューの帯を全部隠すと、表の画面に「+ 新規」が出ず、関係マップには出る。残件として片付ける(人は 2026-10-13 に残件を全部終えることをゴールにした)。

終わりの条件: 帯を出さないとき、表の画面と関係マップのヘッダーの画面の型のメニューの左に「+ 新規」。幅が足りなければ設定を先に省き、次に「+ 新規」を省く。帯を出すときは今までどおり。読むだけの起動では出さない。

関係する要件: CE-25・SR-1・SR-42。

## 不明点と仮定

- 仮定: 関係マップでは画面の型のボタンの文字が長いので、「+ 新規」の桁は表の画面と違ってよい(同じ行で、メニューのすぐ左)。

## 設計

変える場所: `src/ui/bands.rs` の `header_buttons`(残す順に並べた組み合わせから入る最初のものを選ぶ。`HeaderButton::New`)・`header_new_shown`・`relmap_new_button`(帯を出さないときは窓の縁に出さない)、`src/ui/app.rs` の `press_header_button`。変えない場所: 帯を出すときの置き場。

却下: 帯の1行だけ残す・下の帯に出す(提案に記録)。

要件との対応: CE-25・SR-1・SR-42 は `bands.rs`。

検証: `cargo test --bin mdgrid test_new_button_header`、`cargo test --no-fail-fast`。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 受け入れの試験と実装 | CE-25, SR-1, SR-42 | 試験が通り、実装の前の木で落ちる | `src/ui/bands.rs` `src/ui/app.rs` `src/ui/mod.rs` `src/ui/test_new_button_header.rs` | test_ce_25_new_in_header_* / test_ce_25_relmap_new_in_header_when_bar_hidden | 済 |

## 実装の気づき

- 試験は実装と並べて書いたので、実装の前のコミット(5a23ba8)を書き出した木に試験だけを足して落ちることを確かめた(ヘッダーに出す3本が落ち、今の振る舞いを守る3本は通った)。
- 関係マップの試験は最初「表の画面と同じ桁」を求めたが、関係マップの画面の型のボタンの文字が長く桁がずれるので、仕様の「メニューの左」に合わせて「同じ行で、メニューのすぐ左」にした。

## 照合

検証役(フレッシュ文脈のサブエージェント、2026-10-13)が 2503975 を書き出した木と、実装の前の 5a23ba8 に試験だけを足した木(別の target)で確かめた。全体の試験は 1834 通り、落ちは 0(前からの ignore 1)。指摘の試験の穴は埋めた。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CE-25 | checked | コード: src/ui/bands.rs `header_new_shown`・`header_buttons`(新規・メニュー・設定の順)・`relmap_new_button`(帯を出さないときは窓の縁に出さない)、src/ui/app.rs `press_header_button`(`H::New` → 新しいノート) / テスト: src/ui/test_new_button_header.rs の test_ce_25_new_in_header_when_bar_hidden・test_ce_25_relmap_new_in_header_when_bar_hidden・test_ce_25_relmap_header_new_click_opens_name・test_ce_25_new_in_header_tabs_auto_single_view ほか / 今: 通った / 前: 5a23ba8 で、ヘッダーに出す試験が落ちた(ヘッダーに「+ 新規」が無い) |
| SR-1 | checked | コード: src/ui/bands.rs `header`(`header_buttons` の並びを描く) / テスト: test_ce_25_new_in_header_when_bar_hidden / 今: 通った / 前: 5a23ba8 で落ちた |
| SR-42 | checked | コード: src/ui/bands.rs `header_buttons`(幅 79 以上で新規・メニュー・設定、次に新規・メニュー、最後にメニュー。パンくずの後ろに2桁) / テスト: test_ce_25_new_in_header_narrow_keeps_new_over_settings・test_ce_25_new_in_header_dropped_when_narrow / 今: 通った / 前: 5a23ba8 で前者が落ちた |

既存のテストの削除・skip・弱体化: なし(変わったのは src/ui/app.rs・src/ui/bands.rs・src/ui/mod.rs と新しい試験のファイル。ignore の数も同じ)

確かめた: 3 / 3

指摘と行き先:
- 関係マップでの「+ 新規」のクリックと、`tabs = "auto"` でビューが1つのときの試験が無かった。足した(test_ce_25_relmap_header_new_click_opens_name・test_ce_25_new_in_header_tabs_auto_single_view)。
- 低い端末・キーの字の見た目・パンくずとの重なりは、コードで不具合なし。

任意の補足: `HeaderButton::New` の注釈を「表の画面と関係マップ」に直した。
