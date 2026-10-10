# 見た目を整える

[English](../en/appearance.md) · [目次](index.md)

mdgrid の見た目は、設定のファイル(`~/.config/mdgrid/config.toml`)の `[look]` と `[display]` の区画で決まります。この頁は、それぞれが何を変え、どう組み合わせるか、ワークスペースや表ごとに見た目を変える方法を示します。各項目の正確な決まりは[設定](../../config.ja.md)にあります。

- [既定の見た目](#既定の見た目)
- [色: テーマ](#色-テーマ)
- [部品の形: スタイル](#部品の形-スタイル)
- [セルを部品で見せるか文字で見せるか](#セルを部品で見せるか文字で見せるか)
- [枠と選び](#枠と選び)
- [表のまわりに出すもの](#表のまわりに出すもの)
- [ワークスペース・表・ビューごとの見た目](#ワークスペース表ビューごとの見た目)
- [端末と字の形](#端末と字の形)
- [組み合わせの例](#組み合わせの例)

## 既定の見た目

色を使える端末では、落ち着いた「墨」の組(`[look] preset = "sumi"`)で始まります。窓の枠は角の丸い線、選びは反転でなく背景の色、見出しはアクセントの色で下に線。セルは部品で見せます: 状態のような値は色の点と文字(`● doing`)、リストは `·` で区切り(`ui · web`)、真偽は `✓`。色を付けるのは点だけで、値の文字は読みやすいふつうの色のままです。

![既定の見た目](images/demo-edit-list.svg)

色を使わない表示(`--no-color`・`NO_COLOR`・`TERM=dumb`・`[terminal] color = false`)では、ふつうの文字と、選びの反転だけになります。画面のどこも、色だけで意味を伝えません。

## 色: テーマ

`[look] theme` で色の組を選びます。`"default"` は端末の色のまま。`"nord"`・`"solarized-light"`・`"dracula"`・`"gruvbox"`・`"pink-monster"`・`"dozy-pink"` と、落ち着いた `"sumi"`・`"slate"`・`"saas"`・`"saas-dark"`・`"paper"` は画面全体を塗ります。変わるのは色だけです。画面の写しは[テーマ](themes.md)にあります。

```toml
[look]
theme = "nord"
```

明暗の組 `{ light = …, dark = … }` は、端末の地の明るさで選びます。`"auto"` は `{ light = "saas", dark = "sumi" }` と同じです。地の明るさは `COLORFGBG` か端末への問い合わせで知り、分からなければ暗い地とみなします。

```toml
[look]
theme = { light = "paper", dark = "saas-dark" }
```

`[look.colors]` でテーマの色を役割ごと(`accent`・`selection` など)に、`[look.colors.values]` で値ごとの色(`done = "green"`)を決められます。

## 部品の形: スタイル

`[look] preset` で部品の形の組をまとめて選び、`[look.style]` で部品を1つずつ上書きします。

| 組 | 感じ |
|---|---|
| `sumi`(既定) | 点と文字、`·` で区切るタグ、`✓`、見出しの下の線、下線のタブ |
| `slate` | 形(○ ◐ ● ⊘)、`#タグ`、`☑`、塗ったタブ、見出しの印 |
| `saas` | 淡い地の丸い札、切り替えの枠のタブ |
| `paper` | 文字の色だけ、`[x]`、列の線、角の枠 |
| `grid` | 文字のまま、列と見出しの線、キーに淡い地 |
| `classic` | 0.2.0 の見た目(四角い札、塗った選び) |
| `dozy-pink` | 柔らかい丸い札(テーマ `dozy-pink` と合わせる) |

```toml
[look]
preset = "saas"

[look.style]
select = "cross"      # 選んだ行と列を十字に淡く塗る
```

部品は `status`・`tags`・`check`・`select`・`rules`・`tabs`・`frames`・`band`・`links`・`icons` です(全部の値は[設定](../../config.ja.md#lookstyle))。丸い札(`"pill"`)は Nerd Font の字で描きます。既定の `[terminal] nerd_font = "auto"` では、丸い端を自分で描く端末(Ghostty・WezTerm)で丸く、ほかでは角を落とした形にします。`true` ならいつも丸く、`false` ならいつも角を落とします。

**カタログ**: [`docs/catalog/index.html`](../../catalog/index.html) をブラウザで開くと、テーマと部品の形を触れる見本で見比べられます。セルを選ぶ、値を選ぶ、検索、絞り込み、詳細、関係マップ、ヘルプを、クリックとキーで試せます。`config.toml` に貼る設定の文も作れます。

## セルを部品で見せるか文字で見せるか

`[look] cells` は、色を使うときの表の値の見せ方を決めます:

| 値 | `cells = "rich"`(既定) | `cells = "plain"` |
|---|---|---|
| `done: true` | `☑` | `true` |
| `tags: [ui, web]` | 札 `ui` `web`、それぞれの色 | `[ui, web]` |
| `status: doing`(くり返す短い値の列) | 札 `doing` | `doing` |
| `project: "[[mdgrid]]"` | アクセントの色の `mdgrid` | `mdgrid` |
| 列の見出し | `◉ status`・`◷ due`・`# priority`・`☑ done`・`⋮ tags` | `status`・`due`… |

同じ値は、表でも、値の数(`%`)でも、候補の一覧でも、いつも同じ色の札です。値の文字はいつも見せ、色だけで伝えることはしません。ノートに書く値・編集・検索・出力(`--print`)の値は、文字のままの値です。

部品を1つずつ文字のままの形にし、列ごとにも選べます:

```toml
[look.style]
check = "text"        # 真偽を true / false の文字のまま
links = "plain"       # リンクをアクセントの色にしない

[look.columns]
memo = "plain"        # この列は文字のまま
owner = "chip"        # 自動で札にならない列も札に(列の設定が先)
```

列が狭くて札が入りきらないときは、入る札だけを丸ごと見せ、残りは `+N` と数えます。

## 枠と選び

- `[look] mode = "classic"` で前の見た目に戻ります。選びは反転、アクセントの色は使いません。どちらでも画面の文字は同じです。
- 窓の枠は `[look.style] frames` で、`"rounded"`・`"square"`・`"heavy"`・`"ascii"`・`"none"` から選べます(`[terminal] ambiguous_wide = true` のときは、列がずれないようにいつも ASCII)。

## 表のまわりに出すもの

| 項目 | 変わること |
|---|---|
| `[display] tabs = "auto"` | ビューが1つ(ビューを保存していないフォルダ)のあいだ、ビューのタブの行を隠す。ビューを保存すると出る。 |
| `[display] tabs = "never"` | ビューのタブをいつも隠す。`[` `]` でビューは切り替わる。 |
| `[display] search_bar = false` | 表の上の検索の欄を隠す。`\` の絞り込みは最下行で打てる。 |
| `[display] chips = false` | 効いているビューの設定の帯を隠す。 |
| `[display] row_numbers = true` | 表示の順に 1・2・3… の行番号。 |
| `[display] zebra = true` | 一行おきに背景の色(色を使うときだけ)。 |
| `[display] column_lines = true` | 列の間に `│`。 |
| `[display] group_gap = true` | 2つ目からのまとまりの見出しの上を1行空ける。 |

ビューごとに、ビューの設定(`o`)の「表示」の区画で `[display]` の項目を上書きでき、保存した mdgrid のビューはそれを覚えます。

## ワークスペース・表・ビューごとの見た目

この頁の項目(`[look]`・`[display]`、それに `[dates]`・`[edit]`・`[new_note]`)は、ワークスペース・表・ビューごとにも書け、狭い範囲が勝ちます。ワークスペースの印 `.mdgrid/workspace.toml`(ノートと一緒に git などで分け合える)に手で書くか、画面で選びます:

```toml
# notes/.mdgrid/workspace.toml
name = "notes"

[look]
theme = "nord"            # このワークスペースの表は全部

[[table]]
path = "tasks"

[table.look]
theme = "dracula"         # tasks の表だけ
```

画面では、ビューの設定(`o`)の「見た目」の区画で、「保存先」で残す場所を選びます: 全体(`ui.toml`)・このワークスペース(`workspaces.toml`)・この表(`views.toml`)・このビュー。テーマと組の横の「(この表)」などは、今の値がどこから来たかです。「この範囲の上書きを外す」で、広い範囲のとおりに戻ります。mdgrid は `config.toml` とワークスペースの印には書きません。印のワークスペースなら、印に手で書きます。

狭い範囲に書いた `preset` は広い範囲の部品の形を、`theme` は広い範囲の役割の色を使わないので、表が自分の組を選べば、その組がそのまま効きます。`mdgrid <表> --print-config --resolved` で、その表で効く値と、それぞれがどこから来たかを見られます。

テンプレート(`[templates.<名前>]`)は見た目に名前を付けて残したものです。どの範囲でも `use = "<名前>"` でそこから始められます。「見た目」の区画で、今のテーマと組をテンプレートとして保存できます。

## 端末と字の形

- `○`・`※` や罫線が2倍の幅に見えて列がずれるなら、`[terminal] ambiguous_wide = true` にします。枠も ASCII になり、全部がそろいます。
- トゥルーカラーが無い(`COLORTERM` が `truecolor`・`24bit` でない)端末では、256 色のうち近い色を使います。
- チェックと型の印(`☑ ☐ ◉ ◷ ⋮`)はふつうの文字です。字体での見え方が悪ければ、`[look.style] check = "text"` と `icons = false` にします。

## 組み合わせの例

静かで最小:

```toml
[look]
cells = "plain"

[display]
tabs = "auto"
search_bar = false
chips = false
```

案内の線の多い表:

```toml
[display]
row_numbers = true
zebra = true
column_lines = true
```

前の見た目:

```toml
[look]
mode = "classic"
cells = "plain"

[look.style]
frames = "ascii"
```
