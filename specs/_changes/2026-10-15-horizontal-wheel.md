---
type: Change
id: 01M4XR037MRRAN9REJYW4G1HAZ
title: 横のホイールで表を左右に動く(horizontal-wheel)
status: done
size: full
created: 2026-10-15
updated: 2026-10-15
---

# 横のホイールで表を左右に動く(horizontal-wheel)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [2026-10-15-horizontal-wheel](../_decisions/2026-10-15-horizontal-wheel.md) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | c7afb7b |
| 照合 | 済 | この記録の「照合」 |

## 要求

トラックパッドで表を縦には流せるが横には流せない(会話 2026-10-15。プロジェクトのチャットでの問い)。原因は、端末が送り crossterm が ScrollLeft・ScrollRight として渡す横のスクロールを、`src/main.rs` の `handle` が縦のホイールしか拾わずに捨てていたこと。表の画面で、横のホイールと shift+ホイールで左右の列へ動けるようにする。終わりは、SR-6 の新しい例が試験で通り、入れ直した mdgrid で横のスワイプで列が動くこと。要件: SR-6。

## 不明点と仮定

- 仮定: 横のホイールの1回の報告で1列動く(縦のホイールが1回で1行動くのとそろえる)。トラックパッドは報告が多いので速く感じたら、間引く提案を起こす。
- 仮定: 横の向きは端末の報告のまま(ScrollRight で右の列)。縦が人の端末で自然に動いているので、横も端末の設定(ナチュラルスクロール)に任せる。
- 仮定: 関係マップ(REL-12)や詳しく見る画面には横のホイールを効かせない(要件が表の画面だけを言う)。

## 設計

- 変える場所: `src/ui/app.rs` に `hwheel(right)`(表の画面では `Action::Right`・`Action::Left`、ほかは何もしない)と `shift_wheel(down)`(表の画面では `hwheel`、ほかは `wheel`)を足す。`src/main.rs` の `handle` で、ScrollLeft・ScrollRight を `hwheel` に、shift を押した ScrollUp・ScrollDown を `shift_wheel` に渡す。
- 変えない場所: 縦のホイール(`wheel`)、キーの割り当て、列を流す決め方(`scroll_into_view`。SR-3)。
- 選んだ案: 選んだセルを左右に動かす(縦のホイールが行を動かすのとそろえ、SR-3 の「選んだセルが常に見えている」を保つ)。却下: 見える範囲だけを流す案(SR-3 と食い違う)。
- 文書: マウスの操作を書いた説明書の箇所に横のホイールを足す。
- 対応: SR-6 → app.rs の `hwheel`・`shift_wheel` と main.rs の受け口。
- 検証: `cargo test`、`python3 .decidespec/scripts/check.py .`。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 受け入れの試験を置く | SR-6 | 新しい試験が実装前に落ちる | `src/ui/test_hwheel.rs` `src/ui/mod.rs` `tests/test_hwheel_e2e.rs` | test_sr_6_hwheel_* | 済 |
| 2 | 横のホイールと shift+ホイールを受ける | SR-6 | 表で左右の列へ動き、ヘルプでは横は何もせず shift は縦のまま | `src/ui/app.rs` `src/main.rs` | test_sr_6_hwheel_* | 済 |
| 3 | 文書に横のホイールを書く | SR-6 | マウスの操作の説明に載る | 説明書のマウスの箇所 | 読んで判定 | 済 |

## 実装の気づき

- 照合のスクリプト(check.py --evidence)は、SR-6 を指す既存の試験が、ほかのスレッドのビルドと重なって 600 秒で時間切れになり gap を出した。そのあと作業の木がほかのスレッドの書きかけ(WsTable の base)でビルドできなくなったので、実装の時点で取った今と前の結果を書いた。
- `cargo fmt -- <ファイル>` はクレート全体を整形する。自分のファイルだけを `rustfmt --edition 2021 <ファイル>` で整形する。

## 照合

自分で照合(フレッシュ文脈でない)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| SR-6 | checked | コード: src/main.rs の handle(ScrollLeft・ScrollRight と shift の ScrollUp・ScrollDown)、src/ui/app.rs の hwheel・shift_wheel / テスト: tests/test_hwheel_e2e.rs::test_sr_6_hwheel_e2e_terminal_reports、src/ui/test_hwheel.rs::test_sr_6_hwheel_* の4件 / 今: 通った(cargo test --test test_hwheel_e2e、cargo test --bin mdgrid hwheel。c7afb7b の時点) / 前: 落ちた(e2e は実装前の実行ファイルで「横のホイールの右で 2/2列」を待って時間切れ。1/2列のまま。単体は hwheel・shift_wheel が無くビルドが通らない) |

既存のテストの削除・skip・弱体化: なし

確かめた: 1 / 1
