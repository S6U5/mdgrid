# 設定

[English](config.md)(英語が正本)

mdgrid は TOML のファイルを1つ読む:

- `$XDG_CONFIG_HOME/mdgrid/config.toml`。`XDG_CONFIG_HOME` が無ければ `~/.config/mdgrid/config.toml`(macOS でも同じ)。
- `mdgrid --config <パス>` なら、そのファイルを読む。

同じフォルダには `views.toml`(mdgrid のビュー)と `places.toml`(登録した表。[使う表を登録して切り替える](manual/ja/tasks.md#使う表を登録して切り替える))も置く。この2つは mdgrid が書く。

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

- 型: `文字列("default"・"nord"・"solarized-light"・"dracula"・"gruvbox"・"pink-monster"・"dozy-pink")`
- 既定: `"default"`

画面の色のテーマ。`"default"`・`"nord"`・`"solarized-light"`・`"dracula"`・`"gruvbox"`・`"pink-monster"`・`"dozy-pink"` のどれか。`"default"` は端末の文字と地の色のまま(テーマの無かったときの見た目)。ほかのテーマは、地・文字・1行目・列の見出し・選んでいるセル・下の帯・`zebra` の一行おきの色・`>` の印をテーマの色で塗る。意味を持つ色(ためた値・検索の一致・保存の確認の差分の足した行と消した行)も、テーマの地で読める色で塗る。色を使わない表示(`--no-color`・`NO_COLOR`・`TERM=dumb`・`color = false`)ではテーマは効かない。トゥルーカラーに対応しない端末(`COLORTERM` が `truecolor`・`24bit` でない)では、256 色のうち近い色で塗る。知らない名前は警告を出して既定の見た目にする。

```toml
theme = "nord"
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

モード: `table`・`edit`・`review`・`quit`・`help`・`palette`・`search`・`filter`・`detail`・`settings`・`settings_input`・`chips`・`list_select`・`menu`・`freq`。動作の名前はコマンドのパレット(`:`)に出る。モードごとの既定のキーと動作の名前の全部は [keys.ja.md](keys.ja.md) にある。知らないモード・キー・動作は警告して飛ばす。

```toml
[keys.table]
"ctrl+f" = "search"
"x" = "none"
```
