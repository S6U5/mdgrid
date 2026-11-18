---
type: Change
id: 01M3XDKG4X715PH20MADVG15EB
title: mdgrid 独自のビューの定義(native-views)
status: done
size: full
created: 2026-10-02
updated: 2026-10-02
---

# mdgrid 独自のビューの定義(native-views)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [native-views](../_decisions/2026-10-02-native-views.md)(人)、[native-views-details](../_decisions/2026-10-02-native-views-details.md)(AI) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

依頼の要約(会話 2026-10-02): `.base` はおまけ(オプション)にして、それを経由しないで表を組んで見られるようにしたい。mdgrid 独自のビューの定義で構わず、`.base` より機能が多くてよい。`.base` の形と相互に変換できるとなおよい。

今は `.base` なしでも既定の表とビューの設定(`o`)が使えるが、ビューを複数持てず、名前で残せず、手で書けない。終わりの条件: `.base` を使わずに、フォルダごとに名前つきのビューを複数持ってタブで切り替えられ、画面から保存・上書き・名前の変更・削除ができ、定義は mdgrid の設定の置き場の人が読める TOML にある。関係する要件: BV-1・BV-13・NV-13〜NV-22・SR-11・SR-12・CLI-3・CLI-6〜CLI-10(sessions)。

## 不明点と仮定

- 問い → 答え(要約): 定義の置き場 → mdgrid の設定の置き場(`~/.config/mdgrid/`)。ノートのフォルダには書かない。
- 依頼の続き(要約): 独自の定義で構わない、`.base` より機能が多くてよい、`.base` と相互に変換できるとよい。
- 仮定: `.base` で開いたときも、同じ対象に mdgrid のビューがあれば、`.base` のビューのあとにタブとして並べる。外れたら: フォルダで開いたときだけにする。
- 仮定: sessions(CLI-6〜CLI-10)は、この変更のあとに、mdgrid のビューを名前で呼ぶ形で実装する(別の変更の記録 sessions)。

## 設計

設計の本文は docs/design.md の「mdgrid のビュー(BV-17〜BV-20)」。要点: 核の `src/views.rs` に NativeView(名前・列の並び・隠す列・式の絞り込み・ビューの設定)と views.toml の読み書き、`.base` との変換。式の絞り込みの欄を持たせることで、`.base` の filters は落とさずに取り込め、`.base` より機能の多い定義になる。画面はタブに mdgrid のビューを並べ、ビューの設定の画面から保存・上書き・名前の変更・削除、パレットから書き出しと取り込み。

選ばなかった案: `.base` の YAML をそのまま持つ(機能を足せない)、式の絞り込みを持たない(`.base` から取り込むと filters を落とす)。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | mdgrid のビューの核(views.toml の読み書きと `.base` との変換) | BV-17, BV-19, BV-20 | 対象ごとの読み書き、ほかの対象を保つ、知らない項目と壊れたファイルの警告、書き出しと取り込みと落とした部分の説明 | src/views.rs, src/test_views_unit.rs, src/lib.rs, src/base.rs, src/config.rs, src/expr.rs, tests/test_views.rs | test_BV_17, test_BV_19, test_BV_20 | 済 |
| 2 | 画面のタブ・保存と上書きと名前の変更と削除・書き出しと取り込み | BV-17, BV-18, BV-19, BV-20 | タブで切り替え、画面から保存などができ、開き直しても同じ、読むだけでは書かない | src/ui, src/main.rs, src/test_main.rs, src/config.rs, tests/golden | test_BV_17, test_BV_18, test_BV_19, test_BV_20 | 済 |

順: 1 → 2。1 の受け入れの試験(tests/test_views.rs)は実装を見ていない役が先に書く。

## 実装の気づき

- タスク 1: 大文字小文字を区別しない Contains を式で表すため expr に Obsidian の `lower()` を足した(触る範囲の外)。to_base は列の型を知らないので Cmp の読み方を値の形(数・日付・ほか)で決め、テキストの列に数・日付の形の値で比べる条件や、リストの列の Cmp・`, ` を含む Contains は settings と行がずれうる。
- タスク 1(差し戻し): save_views は views.toml を toml の表として読んで書き直すので、知らない項目と読めなかったビューは保つが、コメントと書式は保てない(依存を足さない限りの限界として受け入れた)。
- タスク 1(再レビューの残り、低): 条件の段の知らない項目は保存で消える。`[target.view]`(並びでない表)は読むとき無視するが保存で並びの要素になる。2^53 を超える整数の Keep はずれる。空・数・日付の条件はずれの無い値でも近似の注意を出すので多め。式が Obsidian で同じ行になるかは実物で未確認(照合で読んで判定する)。
- タスク 2: mdgrid のビューは定義の列の並びと filters_expr を to_base → base::parse した `.base` で組むので、`.base` で開いた対象でも mdgrid のビューでは `.base` の formulas・displayName・全体の filters は効かない(取り込みと同じ扱い)。表で変えた分は名前ごとの状態に定義の指紋つきで重ね(状態は置き場の `views/` に分け、名前の変更・削除のため核に config::remove_state を足した)、並びのあるビューでは並びに無いキーを隠した列で出す(受け入れの手書きの order が列をそのビューの列だけにするため、並びを保存したビューにはあとで増えたキーが見えない)。受け入れの test_native_views.rs は rustfmt の形でなく ci.sh の `cargo fmt --check` で落ちるので、空白だけを rustfmt で整えた(字句は同じ)。
- タスク 2(再レビューの残り、低): 同じ名前のビューが手で2つあると削除は先頭の1つを消す。ほかで消されたビューの上書きは断る(main が直した)。他プロセスと同時に動かすと読み直しの後の重ねが別の定義に当たりうる。並びのあるビューでは保管庫の全部のキーが隠した列に入り「隠した列 N」が大きくなる。

## 照合

書込なしのフレッシュ文脈の検証役で照合した(2026-10-02)。検証役は BV-17・BV-18 を checked、BV-19・BV-20 を gap とした。main が、BV-19 の書き出しの知らせ(帯の「(空を隠す)」でも通っていた)を知らせだけに締め、BV-20 の読むだけのボタンの防ぎの試験を足し、どちらも変異で落ちることを確かめた。

| 要件 | 結果 | 証拠 |
|---|---|---|
| BV-17 | checked | コード: src/ui/native_views.rs:163 / テスト: src/ui/test_native_views.rs::test_bv_17_hand_written_views_toml_becomes_tabs / 今: 通った(cargo test) / 前: 落ちた(変異: タブに mdgrid のビューを並べない → 3件落ちた。load_views を飛ばす・置き場をノートのフォルダにする → 落ちた) |
| BV-18 | checked | コード: src/ui/native_views.rs:281 / テスト: src/ui/test_native_views.rs::test_bv_18_overwrite_survives_restart / 今: 通った(cargo test) / 前: 落ちた(変異: 保存・置き換え・名前の変更・削除・読み直しをそれぞれ外す → 該当の1件ずつ落ちた) |
| BV-19 | gap | コード: src/ui/native_io.rs:339 / テスト: src/ui/test_native_views.rs::test_bv_19_export_writes_new_base_and_reports_dropped / 今: 通った(cargo test) / 前: 落ちた(変異: 既存の確認を外す・groupBy を書かない・取り込みで filters を捨てる・落とした項目を知らせない → 落ちた)。欠け: 「Obsidian で開けて同じ行」は Obsidian の実物で未確認(mdgrid の base::parse と build、YAML の妥当性、関数名は確かめた) |
| BV-20 | checked | コード: src/views.rs:125 / テスト: src/ui/test_native_state.rs::test_bv_20_readonly_view_buttons_say_read_only / 今: 通った(cargo test) / 前: 落ちた(変異: 壊れたファイルの警告を捨てる・知らない項目を警告しない・mdgrid のビューを先に並べる・状態を名前ごとにしない・readonly の判定を外す → 落ちた) |

既存のテストの削除・skip・弱体化: なし(ゴールデンはボタンの行とヘルプの行が増えた分、Startup に config_dir: None を足しただけ、受け入れの画面の試験は fmt と、BV-19 の知らせの確かめを締めた1か所)

gap の行き先: BV-19 の Obsidian の実物での確かめは人に聞く(Obsidian で書き出した .base を開いて行を突き合わせる。リストの toString の区切り・数でない値の length・日付の toString の桁に依る)。注記: 保存の一時ファイル → 名前の変更(BV-20 の文にあり例に無い)はコードで満たすが、直接書く変異で落ちる試験は無い。

確かめた: 3 / 4
