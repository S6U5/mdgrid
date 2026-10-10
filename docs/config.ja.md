# 設定

[English](config.md)(英語が正本)

mdgrid は TOML のファイルを1つ読む:

- `$XDG_CONFIG_HOME/mdgrid/config.toml`。`XDG_CONFIG_HOME` が無ければ `~/.config/mdgrid/config.toml`(macOS でも同じ)。
- `mdgrid --config <パス>` なら、そのファイルを読む。

同じフォルダには `views.toml`(mdgrid のビュー)と `places.toml`(登録した表。[使う表を登録して切り替える](manual/ja/tasks.md#使う表を登録して切り替える))も置く。この2つは mdgrid が書く。

ビューの設定の画面(`o`)の「見た目」の区画で選んだテーマ・組・丸い札の端は、同じフォルダの `look.toml` に書き、起動のときこのファイルの `theme`・`[style]` の `preset`・`nerd_font` に代えて使う(このファイルは書き換えない)。`look.toml` を消すか、区画の「config.toml に戻す」で、次の起動からこのファイルのとおりに戻る。区画で名前を付けて保存した組み合わせ(テンプレート)も `look.toml` の `[[template]]` に入る。

どの項目も書かなくてよい。ファイルが無ければ既定で動く。知らない項目と型の違う値は、メッセージ行に警告を出して無視し、起動は止めない。壊れた TOML は理由を1行出して止まる。

既定から始めるなら、説明のコメント付きで書き出す:

```sh
mdgrid --print-config > ~/.config/mdgrid/config.toml
```

書き出したものは警告なしに読め、ファイルが無いときと同じに動く。`editor` と `keys` には既定の値が無いので、コメントにした例として出る。

下が項目の全部。この一覧(名前・「型」と「既定」の行・例)と、mdgrid が読む項目と、`--print-config` の出力が合っているかを試験で確かめている。

## 項目

### `color`

- 型: `真偽値`
- 既定: `true`

色を使う。`false` で色なし(環境変数 `NO_COLOR`・`--no-color` と同じ)。

```toml
color = false
```

### `candidates`

- 型: `整数(0 以上)`
- 既定: `20`

テキストのセルの編集で出す値の候補の上限。異なる値がこれより多い列は候補を出さない。

```toml
candidates = 30
```

### `poll_ms`

- 型: `整数(1 以上)`
- 既定: `1000`

mdgrid の外で変わったノートを読み直す間隔(ミリ秒)。

```toml
poll_ms = 2000
```

### `ambiguous_wide`

- 型: `真偽値`
- 既定: `false`

East Asian Ambiguous の文字(`○`・`※` など)を幅2として扱う。端末が幅2で描くなら `true` にすると列がそろう。

```toml
ambiguous_wide = true
```

### `workspace_detect`

- 型: `文字列の並び`
- 既定: `["vault"]`

書いたワークスペース(`-w`・`.mdgrid/workspace.toml` の印・`workspaces.toml`)に入らない表を開いたとき、上にある Obsidian の保管庫(`"vault"`。`.obsidian/` のあるフォルダ)か git のリポ(`"git"`。`.git` のあるフォルダ)の根をワークスペースとみなす。直下の、ノートのあるフォルダと `.base` が関係マップの表になる。`[]` で検知しない。

```toml
workspace_detect = ["vault", "git"]
```

### `look`

- 型: `文字列`
- 既定: `"modern"`

色を使うときの見た目: `"modern"`(lazygit のように、選びは反転でなく背景の色、窓の枠とキーはアクセントの色、説明は薄い色)か `"classic"`(今までの反転)。どちらでも画面の文字は同じで、色を使わない表示(`NO_COLOR`・`--no-color`)ではどちらも classic。色は `theme` から。

```toml
look = "classic"
```

### `cells`

- 型: `文字列か表`
- 既定: `"rich"`

色を使うときの表のセルの見せ方。`"rich"` は GUI の表のように部品で見せます: 真偽は `☑`・`☐`、リストの要素は1つずつ色の付いた札、いくつかの値をくり返す短い文字の列(`status` など)の値も色の付いた札、リンクはアクセントの色、列の見出しには型の印(数 `#`・日付 `◷`・真偽 `☑`・リスト `⋮`・札 `◉`)。同じ値はいつも同じ色で、文字はいつも見せます。`"plain"` は文字のまま。色を使わない表示(`--no-color`・`NO_COLOR`)では、いつも文字のまま。ノートに書く値・編集・検索・出力(`--print`)の値は変わりません。

```toml
cells = "plain"
```

表で書けば、部品ごと・列ごとに選べます。列の設定は部品の設定より優先します。`"chip"` は、自動で札にならない列も札にします。

```
[cells]
style = "rich"     # か "plain"
checkbox = true    # 真偽を ☑ ☐
chips = true       # リストの要素を札
select = true      # くり返す短い文字の値を札
links = true       # リンクをアクセントの色
icons = true       # 列の見出しに型の印

[cells.columns]
status = "plain"   # この列は文字のまま
owner = "chip"     # この列は札
```

### `view_tabs`

- 型: `文字列`
- 既定: `"always"`

ヘッダーの下のビューのタブの行を出すとき: `"always"`(いつも)か `"auto"`(ビューが2つ以上のときだけ)。`.base` なしで開いたフォルダのビューは **既定の表**(全部のノートと全部のキー)の1つなので、`"auto"` ではタブの行を出さず、その行も表に使います。自分のビューを保存すると出ます。`[display]` の `tabs = false` なら、どちらでも出しません。

```toml
view_tabs = "auto"
```

### `borders`

- 型: `文字列`
- 既定: `"rounded"`

窓の枠の描き方: `"rounded"`(角の丸い、つながった罫線。`╭─╮`)か `"ascii"`(`+ - |`)。`ambiguous_wide = true` のときは、列がずれないようにいつも ASCII。

```toml
borders = "ascii"
```

### `search_bar`

- 型: `真偽値`
- 既定: `true`

表の上に検索の欄を出す(`\` で欄に入る)。`false` なら出さず、簡易の絞り込みは最下行で打つ。

```toml
search_bar = false
```

### `date_format`

- 型: `文字列`
- 既定: `"YYYY-MM-DD"`

表の日付の見せ方と打ち込みの形。部品は `YYYY`・`YY`・`MM`・`M`・`DD`・`D`・`ddd`(曜日)と、英数字でない区切りの文字(`-`・`/`・`.`・空白・`年` など)。月と日は1つずつ要り、年と曜日は1つまで。0で埋めない `M`・`D` は、ほかの数の部品と区切り無しで並べられない。ノートにはいつも `YYYY-MM-DD` で書く。読めない形は警告して既定にする。

```toml
date_format = "YYYY/MM/DD (ddd)"
```

### `week_start`

- 型: `文字列("sun" か "mon")`
- 既定: `"sun"`

日付のカレンダーの週の始まり。

```toml
week_start = "mon"
```

### `add_frontmatter`

- 型: `真偽値`
- 既定: `true`

フロントマターの無いノートと、空のフロントマター(`---` が2行だけ)のノートにも書く(フロントマターかキーの行を足す)。`false` ならこの2つは読むだけになり、理由が出る。区切りの間に空行やコメントの行があるものは空ではなく、いつも書ける。

```toml
add_frontmatter = false
```

### `editor`

- 型: `文字列`
- 既定: なし

選んだノートを開くエディタ(`e`、または選んだ行のファイル名のクリック)。`"code -w"` のような引数つきでよい。値はシェルを通さずに単語に分け(引用符とバックスラッシュは POSIX と同じ)、ノートのパスを1つの引数として足す。書かないか、空か、空白だけなら、`$VISUAL`、`$EDITOR`、`vi` の順で空でない最初のものを使う。エディタが終わると、そのノートを読み直す。

```toml
editor = "nvim"
```

### `language`

- 型: `文字列("auto"・"en"・"ja")`
- 既定: `"auto"`

画面と起動の文言(`--help`・起動できない理由・警告)の言語。`"auto"` は `LC_ALL`・`LC_MESSAGES`・`LANG` の順で最初の空でない値を見て、`ja` で始まれば日本語、ほか(どれも無いときも)は英語。`"en"`・`"ja"` は環境に依らずその言語。ノートの値・列の名前・`.base` の中身・ファイルの名前は訳さない。起動の引数の誤りは設定を読む前に出るので、環境に従う。

```toml
language = "en"
```

### `theme`

- 型: `文字列("auto"・"default"・"nord"・"solarized-light"・"dracula"・"gruvbox"・"pink-monster"・"dozy-pink"・"sumi"・"slate"・"saas"・"saas-dark"・"paper")`
- 既定: `"default"`

画面の色のテーマ。`"default"`・`"nord"`・`"solarized-light"`・`"dracula"`・`"gruvbox"`・`"pink-monster"`・`"dozy-pink"` と、落ち着いた組の `"sumi"`・`"slate"`・`"saas"`・`"saas-dark"`・`"paper"` のどれか。`"auto"` は端末の地の明るさで `theme_light` か `theme_dark` を使う。`"default"` は端末の文字と地の色のまま(テーマの無かったときの見た目)。ほかのテーマは、地・文字・1行目・列の見出し・選んでいるセル・下の帯・`zebra` の一行おきの色・`>` の印をテーマの色で塗る。意味を持つ色(ためた値・検索の一致・保存の確認の差分の足した行と消した行)も、テーマの地で読める色で塗る。色を使わない表示(`--no-color`・`NO_COLOR`・`TERM=dumb`・`color = false`)ではテーマは効かない。トゥルーカラーに対応しない端末(`COLORTERM` が `truecolor`・`24bit` でない)では、256 色のうち近い色で塗る。知らない名前は警告を出して既定の見た目にする。

```toml
theme = "nord"
```

### `theme_light`

- 型: `文字列(テーマの名前)`
- 既定: `"saas"`

`theme = "auto"` のとき、端末の地が明るいときのテーマ。

```toml
theme_light = "paper"
```

### `theme_dark`

- 型: `文字列(テーマの名前)`
- 既定: `"sumi"`

`theme = "auto"` のとき、端末の地が暗いときのテーマ。地の明るさは `COLORFGBG` か端末への問い合わせで知り、分からなければこちらを使う。

```toml
theme_dark = "saas-dark"
```

### `style`

- 型: `表([style] の下に preset・status・tags・check・select・rules・tabs・frames・band・icons)`
- 既定: `{ preset = "sumi" }`

画面の部品の形。`preset` で組をまとめて選びます: `"sumi"`(既定。状態の前に色の点、タグは `·` で区切る、`✓`、選んだ行は印 `>` をアクセントの色に、見出しの下に線)・`"slate"`・`"saas"`・`"paper"`・`"grid"`・`"classic"`(0.2.0 の見た目)・`"dozy-pink"`(テーマ dozy-pink に合う柔らかい丸い札)。ほかの項目で、部品を1つずつ上書きします:

- `status`(状態や担当のような、くり返す短い値): `"dot"`・`"shape"`(○ ◐ ●)・`"text"`・`"pill"`・`"tint"`・`"solid"`(値の色をそのまま塗った、透けない札)・`"soft"`・`"chip"`・`"plain"`
- `tags`(リスト): `"dots"`・`"hash"`・`"brackets"`・`"pill"`・`"tint"`・`"solid"`・`"soft"`・`"chip"`・`"plain"`
- `check`(真偽): `"box"`・`"tick"`・`"bracket"`・`"text"`
- `select`(選んでいる行とセル): `"bar"`・`"cross"`・`"tint"`・`"outline"`・`"fill"`・`"reverse"`
- `rules`(表の線): `"none"`・`"header"`・`"columns"`・`"grid"`
- `tabs`: `"underline"`・`"pill"`・`"segment"`・`"brackets"`・`"dim"`
- `frames`(窓の枠): `"rounded"`・`"square"`・`"heavy"`・`"ascii"`・`"none"`
- `band`(下の帯のキー): `"keys"`・`"boxed"`・`"quiet"`
- `icons`: `true` か `false`(列の見出しの型の印)

`"pill"` は丸い端を Nerd Font の字で描くので、丸い端を描けるとき(`nerd_font`。既定では Ghostty と WezTerm)だけ使い、そうでなければ `"soft"` で描きます。`borders = "ascii"` も今までどおり使え、`frames = "ascii"` と同じです。`docs/catalog/index.html` のカタログで全部の形を見比べ、この表を作れます。

```toml
[style]
preset = "saas"
select = "cross"
```

### `colors`

- 型: `表([colors] の下に色の役割、[colors.values] に値の色)`
- 既定: `{}`

テーマの色を役割ごとに上書きします。役割: `background`(地)・`text`(文字)・`header`(1行目)・`selection`(選んだ行の地)・`selection_text`・`band`(下の帯の地)・`band_text`・`accent`(見出し・キー・枠)・`strong`(強調)・`zebra`(一行おきの地)・`zebra_text`・`mark`(左端の印)・`added`(足した行)・`removed`(消した行)・`pending`(ためた変更)・`highlight`(検索の一致の地)。書かなかった役割はテーマの色のままです。`[colors.values]` は値ごとの色で(前後の空白と大文字・小文字によらず照合)、部品のどの形(`status`・`tags`)でも、候補の窓でも使い、日本語の値にも付けられます。色は `"#rrggbb"`・`"#rgb"` か名前(`black`・`white`・`gray`・`red`・`orange`・`yellow`・`green`・`teal`・`cyan`・`blue`・`purple`・`magenta`・`pink`・`brown`)。`theme = "default"` では端末の地と文字は変えず、`accent`・`selection` と値の色だけを使います。知らない役割と読めない色は警告して飛ばします。同じ値を(大文字・小文字と空白を除いて)2回書いたときも警告し、どれか1つを使います。`docs/catalog/index.html` のカタログの色の欄で、この表を作れます。

```toml
[colors]
accent = "#e0a458"

[colors.values]
done = "green"
```

### `nerd_font`

- 型: `真偽か "auto"`
- 既定: `"auto"`

丸い札の端(Nerd Font の字 U+E0B6・U+E0B4)を描いてよいか。`true` は端末の字形が Nerd Font(どの Nerd Font でもよい)。`"auto"` は、その字を字体に頼らず自分で描く端末、Ghostty と WezTerm(`TERM_PROGRAM` で見分ける)のときだけ描き、ほかの端末では字が化けないようにします。描けないときは `"pill"` を `"soft"` で描き、淡い札・塗った札の端は四角のままです。端末はどの字体を使っているかをアプリに教えないので、ほかの端末で Nerd Font を使っているなら `true` と書きます。

```toml
nerd_font = true
```

### `display`

- 型: `表([display] の下に row_numbers・zebra・column_lines・group_gap・tabs・chips)`
- 既定: `{ row_numbers = false, zebra = false, column_lines = false, group_gap = false, tabs = true, chips = true }`

表の見せ方。どれも省けて、値は `true` か `false`。

- `row_numbers`(既定 `false`): 各行の左に 1・2・3… の行番号。今の表示の並び(絞り込みと並べ替えのあと)の順。グループの見出しの行には付けず、番号はグループをまたいで続く。
- `zebra`(既定 `false`): 一行おきに背景の色を付ける。色を使わない表示(`--no-color`・`NO_COLOR`・`color = false`)では付けない。
- `column_lines`(既定 `false`): 列の間に `│` を引く。
- `group_gap`(既定 `false`): まとまりのあるビューで、2つ目からのまとまりの見出しの上に空きの行を1つ入れる。空きの行は行ではなく、数えず、番号も付けず、カーソルは止まらず、クリックしても何も起きない。
- `tabs`(既定 `true`): ビューのタブを出す。隠しても `[` `]` でビューは切り替わる。
- `chips`(既定 `true`): 効いているビューの設定の帯を出す。隠しても `f` で帯の項目を選べる。

検索の欄は最上位の `search_bar` で、`[display]` には入れない。ビューごとに、ビューの設定(`o`)の「表示」の節でこの7つ(検索の欄を含む)を切り替えられる。ビューはこの設定と違う項目だけを持ち、mdgrid のビューはビューと一緒に保存する。

```toml
[display]
row_numbers = true
zebra = true
column_lines = true
```

### `new_note`

- 型: `表([new_note] の下に mode・folder・name・ask・required・hidden・body と [new_note.set])`
- 既定: `{}`

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
| `{date:YYYY/MM/DD}` | 今日を日付の形で(`date_format` と同じ書き方) |
| `{time}` | 今の時刻(`HH:MM`) |
| `{now}` | 今の日時(`YYYY-MM-DDTHH:MM`) |
| `{weekday}` | 曜日 |
| `{name}` | ノートの名前 |
| `{folder}` | 作る場所のフォルダ |

ビューの絞り込みのうち値が1つに決まるもの(`status == "todo"`・タグを含む など)は、`[new_note.set]` より先に入れる。mdgrid のビュー(`views.toml`)の `[target.view.new_note]` に同じ形で書くと、そのビューではこの設定の代わりにそちらを使う。

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

### `keys`

- 型: `表の表([keys.<モード>] の下に キー = 動作)`
- 既定: なし

モードごとにキーを割り当て直す。書かなければ組み込みのキーの割り当て。`[keys.<モード>]` の下に `キー = "動作の名前"` と書く。動作の名前 `"none"` でそのキーを外す。キーは `"j"`・`"ctrl+s"`・`"shift+tab"` のように書き、`"g g"` のような続けて押すキーは空白で区切る。

モード: `table`・`edit`・`review`・`quit`・`help`・`palette`・`search`・`filter`・`detail`・`settings`・`settings_input`・`chips`・`list_select`・`menu`・`freq`・`sorts`。動作の名前はコマンドのパレット(`:`)に出る。モードごとの既定のキーと動作の名前の全部は [keys.ja.md](keys.ja.md) にある。知らないモード・キー・動作は警告して飛ばす。

```toml
[keys.table]
"ctrl+f" = "search"
"x" = "none"
```
