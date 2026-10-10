# 設定

[English](config.md)(英語が正本)

mdgrid は TOML のファイルを1つ読む:

- `$XDG_CONFIG_HOME/mdgrid/config.toml`。`XDG_CONFIG_HOME` が無ければ `~/.config/mdgrid/config.toml`(macOS でも同じ)。
- `mdgrid --config <パス>` なら、そのファイルを読む。

同じフォルダには mdgrid が書くファイルも置く: `ui.toml`(画面で選んだもの。ビューの設定の「見た目」の区画のテーマなど)、`views.toml`(mdgrid のビューと表ごとの設定)、`workspaces.toml`(ワークスペース)、`places.toml`(登録した表。[使う表を登録して切り替える](manual/ja/tasks.md#使う表を登録して切り替える))。mdgrid は `config.toml` を書き換えない。

どの項目も書かなくてよい。ファイルが無ければ既定で動く。知らない項目・型の違う値・その場所に書けない項目は、メッセージ行に警告を出して無視し、起動は止めない。警告はどれも「ファイル: 項目の道筋: 理由」の形。壊れた TOML は理由を1行出して止まる。

既定から始めるなら、説明のコメント付きで書き出す:

```sh
mdgrid --print-config > ~/.config/mdgrid/config.toml
```

書き出したものは警告なしに読め、ファイルが無いときと同じに動く。既定の無い項目は、コメントにした例として出る。

## 2種類の項目

- **アプリ全体の項目**(範囲「全体だけ」): 最上位の `language`・`editor`・`poll_ms` と、区画 `[terminal]`・`[workspace]`・`[keys]`・`[templates]`。`config.toml` にだけ書く(`ui.toml` には `[terminal] nerd_font` を書ける)。
- **表のプロファイル**(範囲「全体・ワークスペース・表・ビュー」): `use`・`[look]`・`[display]`・`[dates]`・`[edit]`・`[new_note]`。下のどの場所にも同じ形で書け、狭い範囲が広い範囲を上書きする。

## ワークスペース・表・ビューごとの上書き

| 範囲 | 手で書く | mdgrid が書く |
|---|---|---|
| 全体 | `config.toml` | `ui.toml` |
| ワークスペース | 印 `.mdgrid/workspace.toml`(最上位)か、`workspaces.toml` の `[[workspace]]`(`[workspace.look]` など) | `workspaces.toml`(印には書かない) |
| 表 | ワークスペースの表の項目(印の `[[table]]` と `[table.look]`、`[[workspace.table]]` と `[workspace.table.look]`) | `views.toml` の `[[table]]` |
| ビュー | — | ビューの設定(mdgrid のビューは定義にも保存) |

各項目の値は、それを書いた一番狭い範囲から取る(ビュー → 表 → ワークスペース → 全体 → 既定)。同じ範囲では、mdgrid が書いたものが手で書いたものより先(`ui.toml` は `config.toml` より、`views.toml` はワークスペースの表の項目より)。見た目が混ざらないように、2つだけ例外がある: `preset` を書いた範囲は、それより広い範囲の部品の形(`[look.style]`)を使わず、`theme` を書いた範囲は、それより広い範囲の役割の色(`[look.colors]`。値の色は除く)を使わない。`use = "<テンプレート>"` は、その範囲の値の下にテンプレートを敷く。

`mdgrid [<表>] --print-config --resolved` で、その表で効く設定を、項目ごとにどこから来たかのコメント付きで出せる。

```toml
# notes/.mdgrid/workspace.toml — notes のワークスペースの表は nord、tasks だけ dracula
name = "notes"

[look]
theme = "nord"

[[table]]
path = "tasks"

[table.look]
theme = "dracula"
```

## 旧い名前(0.3.0)

0.3.0 の書き方の設定もそのまま動く: 旧い名前は新しい道筋として読み、警告を出す。`mdgrid --migrate-config` で、今の設定を新しい形にした文を出せる(注釈は移らない)。前の版の `look.toml` は `ui.toml` が無いときに読み、次に書くときに `ui.toml` へ移す。`views.toml` の `[[target]]` も同じく `[[table]]` になる。

| 旧い名前 | 新しい道筋 |
|---|---|
| `color` | `terminal.color` |
| `ambiguous_wide` | `terminal.ambiguous_wide` |
| `nerd_font` | `terminal.nerd_font` |
| `workspace_detect` | `workspace.detect` |
| `theme` | `look.theme` |
| `theme_light` | `look.theme` |
| `theme_dark` | `look.theme` |
| `look` | `look.mode` |
| `borders` | `look.style.frames` |
| `style` | `look.style` |
| `style.preset` | `look.preset` |
| `cells` | `look.cells` |
| `cells.style` | `look.cells` |
| `cells.checkbox` | `look.style.check` |
| `cells.chips` | `look.style.tags` |
| `cells.select` | `look.style.status` |
| `cells.links` | `look.style.links` |
| `cells.icons` | `look.style.icons` |
| `cells.columns` | `look.columns` |
| `colors` | `look.colors` |
| `view_tabs` | `display.tabs` |
| `search_bar` | `display.search_bar` |
| `date_format` | `dates.format` |
| `week_start` | `dates.week_start` |
| `candidates` | `edit.candidates` |
| `add_frontmatter` | `edit.add_frontmatter` |

## 項目

下が項目の全部。この一覧(道筋・「型」「既定」「範囲」の行・例)と、mdgrid が読む項目と、`--print-config` の出力が合っているかを試験で確かめている。

## 最上位(アプリ全体)

### `language`

- 型: `文字列("auto"・"en"・"ja")`
- 既定: `"auto"`
- 範囲: 全体だけ

画面と起動の文言(`--help`・起動できない理由・警告)の言語。`"auto"` は `LC_ALL`・`LC_MESSAGES`・`LANG` の順で最初の空でない値を見て、`ja` で始まれば日本語、ほか(どれも無いときも)は英語。`"en"`・`"ja"` は環境に依らずその言語。ノートの値・列の名前・`.base` の中身・ファイルの名前は訳さない。起動の引数の誤りは設定を読む前に出るので、環境に従う。

```toml
language = "en"
```

### `editor`

- 型: `文字列`
- 既定: なし
- 範囲: 全体だけ

選んだノートを開くエディタ(`e`、または選んだ行のファイル名のクリック)。`"code -w"` のような引数つきでよい。値はシェルを通さずに単語に分け(引用符とバックスラッシュは POSIX と同じ)、ノートのパスを1つの引数として足す。書かないか、空か、空白だけなら、`$VISUAL`、`$EDITOR`、`vi` の順で空でない最初のものを使う。エディタが終わると、そのノートを読み直す。

```toml
editor = "nvim"
```

### `poll_ms`

- 型: `整数(1 以上)`
- 既定: `1000`
- 範囲: 全体だけ

mdgrid の外で変わったノートを読み直す間隔(ミリ秒)。

```toml
poll_ms = 2000
```

### `use`

- 型: `文字列(テンプレートの名前)`
- 既定: なし
- 範囲: 全体・ワークスペース・表・ビュー

テンプレート `[templates.<名前>]` を、同じ場所に書いた値の下に敷く。config.toml なら全体の、ワークスペース・表・ビューならその範囲の土台になる。無い名前は警告して無視する。

```toml
use = "night"
```

## `[terminal]` — 端末

### `terminal.color`

- 型: `真偽値`
- 既定: `true`
- 範囲: 全体だけ

色を使う。`false` で色なし(環境変数 `NO_COLOR`・`--no-color` と同じ)。

```toml
[terminal]
color = false
```

### `terminal.ambiguous_wide`

- 型: `真偽値`
- 既定: `false`
- 範囲: 全体だけ

East Asian Ambiguous の文字(`○`・`※` など)を幅2として扱う。端末が幅2で描くなら `true` にすると列がそろう。このとき窓の枠はいつも ASCII。

```toml
[terminal]
ambiguous_wide = true
```

### `terminal.nerd_font`

- 型: `真偽か "auto"`
- 既定: `"auto"`
- 範囲: 全体だけ

丸い札の端(Nerd Font の字 U+E0B6・U+E0B4)を描いてよいか。`true` は端末の字形が Nerd Font(どの Nerd Font でもよい)。`"auto"` は、その字を字体に頼らず自分で描く端末、Ghostty と WezTerm(`TERM_PROGRAM` で見分ける)のときだけ描き、ほかの端末では字が化けないようにする。描けないときは `"pill"` を `"soft"` で描き、淡い札・塗った札の端は四角のまま。端末はどの字体を使っているかをアプリに教えないので、ほかの端末で Nerd Font を使っているなら `true` と書く。ビューの設定の「見た目」の区画は、この項目を ui.toml に書く。

```toml
[terminal]
nerd_font = true
```

## `[workspace]` — ワークスペース

### `workspace.detect`

- 型: `文字列の並び`
- 既定: `["vault"]`
- 範囲: 全体だけ

書いたワークスペース(`-w`・`.mdgrid/workspace.toml` の印・`workspaces.toml`)に入らない表を開いたとき、上にある Obsidian の保管庫(`"vault"`。`.obsidian/` のあるフォルダ)か git のリポ(`"git"`。`.git` のあるフォルダ)の根をワークスペースとみなす。直下の、ノートのあるフォルダが関係マップの表になる。`[]` で検知しない。検知したワークスペースには、上書きのワークスペースの範囲は無い。

```toml
[workspace]
detect = ["vault", "git"]
```

## `[look]` — テーマ・部品・色

### `look.theme`

- 型: `文字列("auto"・"default"・"nord"・"solarized-light"・"dracula"・"gruvbox"・"pink-monster"・"dozy-pink"・"sumi"・"slate"・"saas"・"saas-dark"・"paper")か表({ light, dark })`
- 既定: `"default"`
- 範囲: 全体・ワークスペース・表・ビュー

画面の色のテーマ。`"default"`・`"nord"`・`"solarized-light"`・`"dracula"`・`"gruvbox"`・`"pink-monster"`・`"dozy-pink"` と、落ち着いた組の `"sumi"`・`"slate"`・`"saas"`・`"saas-dark"`・`"paper"` のどれか。表 `{ light = "…", dark = "…" }` は端末の地の明るさ(`COLORFGBG` か端末への問い合わせ。分からなければ暗い地)で選び、省いた側は既定を使う。`"auto"` は `{ light = "saas", dark = "sumi" }` と同じ。`"default"` は端末の文字と地の色のまま。ほかのテーマは、地・文字・1行目・列の見出し・選んでいるセル・下の帯・一行おきの色・`>` の印をテーマの色で塗り、意味を持つ色(ためた値・検索の一致・差分の足した行と消した行)もテーマの地で読める色で塗る。色を使わない表示(`--no-color`・`NO_COLOR`・`TERM=dumb`・`terminal.color = false`)ではテーマは効かない。トゥルーカラーに対応しない端末では 256 色のうち近い色で塗る。知らない名前は警告して既定の見た目にする。狭い範囲に書いたテーマは、広い範囲の役割の色(`[look.colors]`)を使わない。

```toml
[look]
theme = { light = "paper", dark = "sumi" }
```

### `look.preset`

- 型: `文字列("sumi"・"slate"・"saas"・"paper"・"grid"・"classic"・"dozy-pink")`
- 既定: `"sumi"`
- 範囲: 全体・ワークスペース・表・ビュー

部品の形の組: `"sumi"`(既定。状態の前に色の点、タグは `·` で区切る、`✓`、選んだ行は印 `>` をアクセントの色に、見出しの下に線)・`"slate"`・`"saas"`・`"paper"`・`"grid"`・`"classic"`(0.2.0 の見た目)・`"dozy-pink"`(テーマ dozy-pink に合う柔らかい丸い札)。`[look.style]` で部品を1つずつ上書きする。狭い範囲に書いた組は、広い範囲の部品の形を使わない。

```toml
[look]
preset = "saas"
```

### `look.mode`

- 型: `文字列("modern" か "classic")`
- 既定: `"modern"`
- 範囲: 全体・ワークスペース・表・ビュー

色を使うときの見た目: `"modern"`(lazygit のように、選びは反転でなく背景の色、窓の枠とキーはアクセントの色、説明は薄い色)か `"classic"`(今までの反転)。どちらでも画面の文字は同じで、色を使わない表示ではどちらも classic。

```toml
[look]
mode = "classic"
```

### `look.cells`

- 型: `文字列("rich" か "plain")`
- 既定: `"rich"`
- 範囲: 全体・ワークスペース・表・ビュー

色を使うときの表のセルの見せ方。`"rich"` は `[look.style]` の形の部品で見せる: 真偽(`check`)、リストの要素(`tags`)、いくつかの値をくり返す短い文字の列(`status` など)の値(`status`)、リンクはアクセントの色(`links`)、列の見出しの型の印(`icons`)。同じ値はいつも同じ色で、文字はいつも見せる。`"plain"` は文字のまま。色を使わない表示ではいつも文字のまま。ノートに書く値・編集・検索・出力(`--print`)の値は変わらない。

```toml
[look]
cells = "plain"
```

### `look.style`

- 型: `表(status・tags・check・select・rules・tabs・frames・band・links・icons)`
- 既定: なし
- 範囲: 全体・ワークスペース・表・ビュー

組の部品の形を1つずつ上書きする:

- `status`(状態や担当のような、くり返す短い値): `"dot"`・`"shape"`(○ ◐ ●)・`"text"`・`"pill"`・`"tint"`・`"solid"`(値の色をそのまま塗った札)・`"soft"`・`"chip"`・`"plain"`
- `tags`(リスト): `"dots"`・`"hash"`・`"brackets"`・`"pill"`・`"tint"`・`"solid"`・`"soft"`・`"chip"`・`"plain"`
- `check`(真偽): `"box"`・`"tick"`・`"bracket"`・`"text"`
- `select`(選んでいる行とセル): `"bar"`・`"cross"`・`"tint"`・`"outline"`・`"fill"`・`"reverse"`
- `rules`(表の線): `"none"`・`"header"`・`"columns"`・`"grid"`
- `tabs`: `"underline"`・`"pill"`・`"segment"`・`"brackets"`・`"dim"`
- `frames`(窓の枠): `"rounded"`・`"square"`・`"heavy"`・`"ascii"`・`"none"`
- `band`(下の帯のキー): `"keys"`・`"boxed"`・`"quiet"`
- `links`: `"accent"` か `"plain"`
- `icons`: `true` か `false`(列の見出しの型の印)

`"plain"`(`check` は `"text"`、`icons` は `false`)はその部品を文字のままにする。`"pill"` は丸い端を Nerd Font の字で描くので、丸い端を描けるとき(`terminal.nerd_font`)だけ使い、そうでなければ `"soft"` で描く。`docs/catalog/index.html` のカタログで全部の形を見比べ、この表を作れる。

```toml
[look.style]
status = "chip"
select = "cross"
```

### `look.columns`

- 型: `表(列 = "rich"・"plain"・"chip")`
- 既定: なし
- 範囲: 全体・ワークスペース・表・ビュー

列ごとの見せ方: `"rich"`(部品の形が文字のままでも部品に)・`"plain"`(文字のまま)・`"chip"`(自動で札にならない列も札に)。列の設定は部品の形より優先する。

```toml
[look.columns]
status = "plain"
owner = "chip"
```

### `look.colors`

- 型: `表(色の役割と、[look.colors.values] に値の色)`
- 既定: なし
- 範囲: 全体・ワークスペース・表・ビュー

テーマの色を役割ごとに上書きする。役割: `background`(地)・`text`(文字)・`header`(1行目)・`selection`(選んだ行の地)・`selection_text`・`band`(下の帯の地)・`band_text`・`accent`(見出し・キー・枠)・`strong`(強調)・`zebra`(一行おきの地)・`zebra_text`・`mark`(左端の印)・`added`(足した行)・`removed`(消した行)・`pending`(ためた変更)・`highlight`(検索の一致の地)。書かなかった役割はテーマの色のまま。`[look.colors.values]` は値ごとの色で(前後の空白と大文字・小文字によらず照合)、部品のどの形でも、候補の窓でも使い、日本語の値にも付けられる。色は `"#rrggbb"`・`"#rgb"` か名前(`black`・`white`・`gray`・`red`・`orange`・`yellow`・`green`・`teal`・`cyan`・`blue`・`purple`・`magenta`・`pink`・`brown`)。`theme = "default"` では端末の地と文字は変えず、`accent`・`selection` と値の色だけを使う。知らない役割と読めない色は警告して飛ばし、同じ値を2回書いたときも警告する。狭い範囲に書いたテーマは、広い範囲の役割の色を使わない(値の色は使う)。

```toml
[look.colors]
accent = "#e0a458"

[look.colors.values]
done = "green"
```

## `[display]` — 表の見せ方

### `display.row_numbers`

- 型: `真偽値`
- 既定: `false`
- 範囲: 全体・ワークスペース・表・ビュー

各行の左に 1・2・3… の行番号。今の表示の並び(絞り込みと並べ替えのあと)の順。グループの見出しの行には付けず、番号はグループをまたいで続く。ビューごとに、ビューの設定(`o`)の「表示」の区画で切り替えられる。

```toml
[display]
row_numbers = true
```

### `display.zebra`

- 型: `真偽値`
- 既定: `false`
- 範囲: 全体・ワークスペース・表・ビュー

一行おきに背景の色を付ける。色を使わない表示では付けない。ビューごとに、ビューの設定(`o`)の「表示」の区画で切り替えられる。

```toml
[display]
zebra = true
```

### `display.column_lines`

- 型: `真偽値`
- 既定: `false`
- 範囲: 全体・ワークスペース・表・ビュー

列の間に `│` を引く。ビューごとに、ビューの設定(`o`)の「表示」の区画で切り替えられる。

```toml
[display]
column_lines = true
```

### `display.group_gap`

- 型: `真偽値`
- 既定: `false`
- 範囲: 全体・ワークスペース・表・ビュー

まとまりのあるビューで、2つ目からのまとまりの見出しの上に空きの行を1つ入れる。空きの行は行ではなく、数えず、番号も付けず、カーソルは止まらず、クリックしても何も起きない。ビューごとに、ビューの設定(`o`)の「表示」の区画で切り替えられる。

```toml
[display]
group_gap = true
```

### `display.tabs`

- 型: `文字列("always"・"auto"・"never")`
- 既定: `"always"`
- 範囲: 全体・ワークスペース・表・ビュー

ヘッダーの下のビューのタブの行: `"always"`(いつも)・`"auto"`(ビューが2つ以上のときだけ。`.base` なしで開いたフォルダのビューは **既定の表** の1つなので、自分のビューを保存するまでタブの行を出さず、その行も表に使う)・`"never"`(出さない)。どれでも `[` `]` でビューは切り替わる。ビューの設定の「表示」の区画では、この3つを順に切り替える。ビューごとに、ビューの設定(`o`)の「表示」の区画で切り替えられる。

```toml
[display]
tabs = "auto"
```

### `display.search_bar`

- 型: `真偽値`
- 既定: `true`
- 範囲: 全体・ワークスペース・表・ビュー

表の上に検索の欄を出す(`\` で欄に入る)。`false` なら出さず、簡易の絞り込みは最下行で打つ。ビューごとに、ビューの設定(`o`)の「表示」の区画で切り替えられる。

```toml
[display]
search_bar = false
```

### `display.chips`

- 型: `真偽値`
- 既定: `true`
- 範囲: 全体・ワークスペース・表・ビュー

効いているビューの設定の帯を出す。隠しても `f` で帯の項目を選べる。ビューごとに、ビューの設定(`o`)の「表示」の区画で切り替えられる。

```toml
[display]
chips = false
```

## `[dates]` — 日付

### `dates.format`

- 型: `文字列`
- 既定: `"YYYY-MM-DD"`
- 範囲: 全体・ワークスペース・表・ビュー

表の日付の見せ方と打ち込みの形。部品は `YYYY`・`YY`・`MM`・`M`・`DD`・`D`・`ddd`(曜日)と、英数字でない区切りの文字(`-`・`/`・`.`・空白・`年` など)。月と日は1つずつ要り、年と曜日は1つまで。0で埋めない `M`・`D` は、ほかの数の部品と区切り無しで並べられない。ノートにはいつも `YYYY-MM-DD` で書く。読めない形は警告して既定にする。

```toml
[dates]
format = "YYYY/MM/DD (ddd)"
```

### `dates.week_start`

- 型: `文字列("sun" か "mon")`
- 既定: `"sun"`
- 範囲: 全体・ワークスペース・表・ビュー

日付のカレンダーの週の始まり。

```toml
[dates]
week_start = "mon"
```

## `[edit]` — 編集

### `edit.candidates`

- 型: `整数(0 以上)`
- 既定: `20`
- 範囲: 全体・ワークスペース・表・ビュー

テキストのセルの編集で出す値の候補の上限。異なる値がこれより多い列は候補を出さない。

```toml
[edit]
candidates = 30
```

### `edit.add_frontmatter`

- 型: `真偽値`
- 既定: `true`
- 範囲: 全体・ワークスペース・表・ビュー

フロントマターの無いノートと、空のフロントマター(`---` が2行だけ)のノートにも書く(フロントマターかキーの行を足す)。`false` ならこの2つは読むだけになり、理由が出る。区切りの間に空行やコメントの行があるものは空ではなく、いつも書ける。

```toml
[edit]
add_frontmatter = false
```

## `[new_note]` — 新しいノート

### `new_note`

- 型: `表(mode・folder・name・ask・required・hidden・body と [new_note.set])`
- 既定: なし
- 範囲: 全体・ワークスペース・表・ビュー

表から新しいノートを作るとき(画面の「+ 新規」・`a`)の決まり。どれも省ける。

- `folder`: 作る既定のフォルダ(開いたフォルダからの相対。空なら開いたフォルダ)。`..` や絶対パスで外に出る値には作らない。`.base` のビューの絞り込みに `file.inFolder("X")` があるときは、作った行がビューに残るように、そのフォルダ(保管庫の根からの `X`)に作る。`folder` はそのフォルダの中(`X/inbox` など)のときだけ使う。
- `name`: 名前の欄に前もって入れる雛形。`{date}` は今日の日付(`YYYY-MM-DD`)。
- `ask`: 新しいノートの窓に欄として並べる列の並び。無ければ表で見えている列を並べる。欄は列の型に合った入力で直し、空の欄は書かない。どの欄からでも `Ctrl+S` で作る。
- `required`: 空では作らない列。空のあいだは作らず、その欄に理由を出す。
- `hidden`: 窓に欄を出さずに値だけ入れる列(`[new_note.set]` と組んで、作成日などを入れる)。
- `mode`: `"form"`(既定。全部の欄を並べた窓)か `"editor"`(名前だけを聞き、前もって入れる値と本文の雛形で作って、すぐエディタで開く)。窓からも `Ctrl+E` で、作ってすぐエディタで開ける。
- `body`: 本文の雛形のファイル(開いたフォルダからの相対)。その中身を新しいノートの本文にする。
- `[new_note.set]`: 新しいノートに前もって入れる値(列 = 値)。文字列・数・真偽・日付・その並び(リストとして縦の形で書く)。

雛形(`name`・`[new_note.set]` の文字の値・`body` のファイル)には次の変数を書ける。知らない変数はそのまま残る。

| 変数 | 値 |
|---|---|
| `{date}` | 今日(`YYYY-MM-DD`) |
| `{date+7}`・`{date-1}` | 今日から何日後・前 |
| `{date:YYYY/MM/DD}` | 今日を日付の形で(`dates.format` と同じ書き方) |
| `{time}` | 今の時刻(`HH:MM`) |
| `{now}` | 今の日時(`YYYY-MM-DDTHH:MM`) |
| `{weekday}` | 曜日 |
| `{name}` | ノートの名前 |
| `{folder}` | 作る場所のフォルダ |

ビューの絞り込みのうち値が1つに決まるもの(`status == "todo"`・タグを含む など)は、`[new_note.set]` より先に入れる。`[new_note]` は1つの項目で、狭い範囲(ワークスペース・表、`views.toml` の `[table.view.new_note]` の mdgrid のビュー)に書けば丸ごと代わる。

```toml
[new_note]
folder = "inbox"
name = "{date} "
ask = ["priority", "due"]
required = ["due"]
hidden = ["created"]
body = "templates/note.md"

[new_note.set]
tags = ["inbox"]
due = "{date+7}"
created = "{now}"
```

## `[keys]` — キー

### `keys`

- 型: `表の表([keys.<モード>] の下に キー = 動作)`
- 既定: なし
- 範囲: 全体だけ

モードごとにキーを割り当て直す。書かなければ組み込みのキーの割り当て。`[keys.<モード>]` の下に `キー = "動作の名前"` と書く。動作の名前 `"none"` でそのキーを外す。キーは `"j"`・`"ctrl+s"`・`"shift+tab"` のように書き、`"g g"` のような続けて押すキーは空白で区切る。

モード: `table`・`edit`・`review`・`quit`・`help`・`palette`・`search`・`filter`・`detail`・`settings`・`settings_input`・`chips`・`list_select`・`menu`・`freq`・`sorts`・`relations`。動作の名前はコマンドのパレット(`:`)に出る。モードごとの既定のキーと動作の名前の全部は [keys.ja.md](keys.ja.md) にある。知らないモード・キー・動作は警告して飛ばす。

```toml
[keys.table]
"ctrl+f" = "search"
"x" = "none"
```

## `[templates]` — テンプレート

### `templates`

- 型: `表の表([templates.<名前>] の下にプロファイルの項目)`
- 既定: なし
- 範囲: 全体だけ

名前を付けたプロファイルの断片(`[look]`・`[display]`・`[dates]`・`[edit]`・`[new_note]`)。どの範囲でも `use = "<名前>"` で、その範囲の値の下に敷く。テンプレートの中に `use` は書けない。ビューの設定の「見た目」の区画で保存したテンプレートは ui.toml に入り、同じ名前なら ui.toml のものが config.toml より先。

```toml
[templates.night.look]
theme = "dracula"
preset = "dozy-pink"
```
