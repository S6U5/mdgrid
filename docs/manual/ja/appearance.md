# 見た目を整える

[English](../en/appearance.md) · [目次](index.md)

mdgrid の見た目は、設定のファイル(`~/.config/mdgrid/config.toml`)のいくつかの項目で決まります。この頁は、それぞれが何を変え、どう組み合わせるかを示します。各項目の正確な決まりは[設定](../../config.ja.md)にあります。

- [既定の見た目](#既定の見た目)
- [色: テーマ](#色-テーマ)
- [セルを部品で見せるか文字で見せるか](#セルを部品で見せるか文字で見せるか)
- [枠と選び](#枠と選び)
- [表のまわりに出すもの](#表のまわりに出すもの)
- [端末と字の形](#端末と字の形)
- [組み合わせの例](#組み合わせの例)

## 既定の見た目

色を使える端末では、lazygit のような見た目で始まります。窓の枠は角の丸い線、選びは反転でなく背景の色、キーと見出しはアクセントの色です。セルは部品で見せます: チェックボックス、リストやくり返す短い値の色の付いた札、列の見出しの型の印。

![既定の見た目](../../assets/ja/demo-edit-list.svg)

色を使わないとき(`--no-color`・`NO_COLOR`・`TERM=dumb`・`color = false`)は、文字だけの画面で、選びは反転です。画面のどの意味も、色だけには頼りません。

## 色: テーマ

`theme` で7つの色の組から選びます。`"default"` は端末の色のまま、`"nord"`・`"solarized-light"`・`"dracula"`・`"gruvbox"`・`"pink-monster"`・`"dozy-pink"` は画面全体を塗ります。変わるのは色だけです。見本は[テーマ](themes.md)にあります。

```toml
theme = "nord"
```

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
- `borders = "ascii"` で、窓の枠を角の丸い線でなく `+ - |` で描きます(`ambiguous_wide = true` のときは、列がずれないようにいつも ASCII)。

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
