# 見た目を整える

[English](../en/appearance.md) · [目次](index.md)

mdgrid の見た目は、設定のファイル(`~/.config/mdgrid/config.toml`)のいくつかの項目で決まります。この頁は、それぞれが何を変え、どう組み合わせるかを示します。各項目の正確な決まりは[設定](../../config.ja.md)にあります。

- [既定の見た目](#既定の見た目)
- [色: テーマ](#色-テーマ)
- [部品の形: スタイル](#部品の形-スタイル)
- [セルを部品で見せるか文字で見せるか](#セルを部品で見せるか文字で見せるか)
- [枠と選び](#枠と選び)
- [表のまわりに出すもの](#表のまわりに出すもの)
- [端末と字の形](#端末と字の形)
- [組み合わせの例](#組み合わせの例)

## 既定の見た目

色を使える端末では、落ち着いた「墨」の組(`[style] preset = "sumi"`)で始まります。窓の枠は角の丸い線、選びは反転でなく背景の色、見出しはアクセントの色で下に線。セルは部品で見せます: 状態のような値は色の点と文字(`● doing`)、リストは `·` で区切り(`ui · web`)、真偽は `✓`。色を付けるのは点だけで、値の文字は読みやすいふつうの色のままです。

![既定の見た目](images/demo-edit-list.svg)

色を使わないとき(`--no-color`・`NO_COLOR`・`TERM=dumb`・`color = false`)は、文字だけの画面で、選びは反転です。画面のどの意味も、色だけには頼りません。

## 色: テーマ

`theme` で色の組を選びます。`"default"` は端末の色のまま、`"nord"`・`"solarized-light"`・`"dracula"`・`"gruvbox"`・`"pink-monster"`・`"dozy-pink"` と、落ち着いた組の `"sumi"`(墨)・`"slate"`(石板)・`"saas"`・`"saas-dark"`・`"paper"`(紙)は画面全体を塗ります。変わるのは色だけです。見本は[テーマ](themes.md)にあります。

```toml
theme = "nord"
```

`theme = "auto"` は、端末の地が明るければ `theme_light`(既定 `"saas"`)、暗ければ `theme_dark`(既定 `"sumi"`)を使います。地の明るさは `COLORFGBG` か端末への問い合わせで知り、分からなければ暗い地とみなします。

```toml
theme = "auto"
theme_light = "paper"
theme_dark = "saas-dark"
```

## 部品の形: スタイル

`[style]` で、表と窓の部品の形を選びます。`preset` で組をまとめて選び、ほかの項目で部品を1つずつ上書きします。

| 組 | 感じ |
|---|---|
| `sumi`(既定) | 点と文字、`·` 区切りのタグ、`✓`、見出しの下の線、下線のタブ |
| `slate` | 形(○ ◐ ● ⊘)、`#タグ`、`☑`、塗ったタブ、見出しの印 |
| `saas` | 淡い地の札、切り替えの枠のタブ |
| `paper` | 文字の色だけ、`[x]`、列の縦線、角の線の枠 |
| `grid` | 文字のまま、縦線と見出しの線、キーに淡い地 |
| `classic` | 0.2.0 の見た目(四角い札、塗った選び) |
| `dozy-pink` | 柔らかい丸い札(テーマ `dozy-pink` と合わせる) |

```toml
[style]
preset = "saas"
select = "cross"      # 選んだ行と列を十字に塗る
```

部品の候補は `status`・`tags`・`check`・`select`・`rules`・`tabs`・`frames`・`band`・`icons` です(全部は[設定](../../config.ja.md#style))。丸い札(`"pill"`)は Nerd Font の字で描きます。`nerd_font` は既定の `"auto"` なら、丸い端を自分で描く端末(Ghostty・WezTerm)で丸い札にし、ほかの端末では角を落とした札で描きます。`true` でいつも丸く、`false` でいつも角を落とした札です。

**カタログ**: [`docs/catalog/index.html`](../../catalog/index.html) をブラウザで開くと、テーマと部品の形を触れる見本で見比べられます。セルを選ぶ・値を選ぶ・検索・絞り込み・詳細・関係マップ・ヘルプを、クリックとキーで試せます。選んだ形の設定の文(`config.toml` に貼るもの)も、そこで作れます。

## セルを部品で見せるか文字で見せるか

`cells` は、色を使うときの表の値の見せ方を決めます:

| 値 | `cells = "rich"`(既定) | `cells = "plain"` |
|---|---|---|
| `done: true` | `☑` | `true` |
| `tags: [ui, web]` | 札 `ui` `web`(それぞれの色) | `[ui, web]` |
| `status: doing`(くり返す短い値の列) | 札 `doing` | `doing` |
| `project: "[[mdgrid]]"` | `mdgrid`(アクセントの色) | `mdgrid` |
| 列の見出し | `◉ status`・`◷ due`・`# priority`・`☑ done`・`⋮ tags` | `status`・`due`… |

同じ値は、表でも、値の件数(`%`)でも、候補の一覧でも、いつも同じ色の札です。値の文字はいつも見せ、色を文字の代わりにはしません。ノートに書く値・直す値・検索・出力(`--print`)は、元の値のままです。

部品ごと、列ごとに選べます:

```toml
[cells]
checkbox = false      # true / false は文字のまま
select = true         # くり返す短い値は札

[cells.columns]
memo = "plain"        # この列は文字のまま
owner = "chip"        # 自動で札にならなくても札(列の設定が優先)
```

列の幅に札が入りきらないときは、入る札だけを丸ごと見せ、残りは `+N` で数を示します。

## 枠と選び

- `look = "classic"` で、前の見た目(選びは反転、アクセントの色なし)に戻せます。どちらでも画面の文字は同じです。
- 窓の枠は `[style] frames` で、`"rounded"`・`"square"`・`"heavy"`・`"ascii"`・`"none"` から選べます。`borders = "ascii"` も今までどおり使え、`frames = "ascii"` と同じです(`ambiguous_wide = true` のときは、列がずれないようにいつも ASCII)。

## 表のまわりに出すもの

| 項目 | 変わること |
|---|---|
| `view_tabs = "auto"` | ビューが1つしか無い間(保存したビューの無いフォルダ)は、タブの行を出さない。ビューを保存すると出る |
| `search_bar = false` | 表の上の検索の欄を出さない(`\` の絞り込みは最下行で打てる) |
| `[display] tabs = false` | ビューのタブをいつも出さない(`[`・`]` で切り替えは効く) |
| `[display] chips = false` | 効いている見せ方の帯を出さない |
| `[display] row_numbers = true` | 行に見えている順の番号 1・2・3… |
| `[display] zebra = true` | 一行おきに地の色(色を使うときだけ) |
| `[display] column_lines = true` | 列の間に `│` |
| `[display] group_gap = true` | 2つ目からのまとまりの見出しの上を1行空ける |

`[display]` の項目と検索の欄は、ビューごとにビューの設定(`o`)で上書きでき、保存した mdgrid のビューはそれを覚えます。

## 端末と字の形

- `○`・`※`・枠の線が2桁分の幅で描かれて列がずれるときは、`ambiguous_wide = true` にします。枠は ASCII になり、全部がそろいます。
- 端末が true color でない(`COLORTERM` が `truecolor`・`24bit` でない)ときは、256色のうちいちばん近い色を使います。
- チェックボックスと型の印(`☑ ☐ ◉ ◷ ⋮`)はふつうの文字です。字の形が崩れるときは、`[cells]` の `checkbox = false` と `icons = false` にします。

## 組み合わせの例

静かで少ない見た目:

```toml
view_tabs = "auto"
search_bar = false
cells = "plain"

[display]
chips = false
```

目印の多い、詰まった表:

```toml
[display]
row_numbers = true
zebra = true
column_lines = true
```

前の見た目:

```toml
look = "classic"
borders = "ascii"
cells = "plain"
```
