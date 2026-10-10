//! 設定の項目の表(CLI-11・CLI-12)。config.rs から分けた(`mdgrid::config::{Item, ITEMS, KEYS}` で使う)。

/// 設定の項目の表の1行(CLI-11・CLI-12)。項目の一覧はここ1か所に持ち、知らない項目の判定(`parse`)・
/// `--print-config`(`default_toml`)・文書との突き合わせの試験がこれを使う。
#[derive(Debug, Clone, Copy)]
pub struct Item {
    /// 設定の最上位の名前。
    pub name: &'static str,
    /// 値の型(英語)。docs/config.md の `- Type: `…`` の行と同じ(試験で突き合わせる)。
    pub ty: &'static str,
    /// 値の型(日本語)。docs/config.ja.md の `- 型: `…`` の行と同じ。
    pub ty_ja: &'static str,
    /// 既定値の TOML の表記。None は「既定では書かない」項目で、`example` をコメントで出す。
    /// 文書の `- Default: `…`` / `- 既定: `…``(None は `- Default: none` / `- 既定: なし`)と同じ。
    pub default: Option<&'static str>,
    /// 書き方の例(TOML の行。`default` が None のときは `--print-config` にコメントで出す)。
    /// 文書の各項目の ```toml の区画と同じ。
    pub example: &'static str,
    /// 英語の説明(`--print-config` のコメント。行ごとに `# ` を付けて出す)。
    pub en: &'static str,
    /// 日本語の説明。
    pub ja: &'static str,
}

/// 設定の項目の表。`--print-config` はこの順で出す(表の見出しになる `keys` は最後)。
pub const ITEMS: &[Item] = &[
    Item {
        name: "color",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("true"),
        example: "color = false",
        en: "Use colors. false draws without colors (same as NO_COLOR or --no-color).",
        ja: "色を使う。false で色なし(NO_COLOR・--no-color と同じ)。",
    },
    Item {
        name: "candidates",
        ty: "integer (0 or more)",
        ty_ja: "整数(0 以上)",
        default: Some("20"),
        example: "candidates = 30",
        en: "Maximum number of value candidates offered when editing a text cell.\n\
             Columns with more distinct values offer no candidates.",
        ja: "テキストのセルの編集で出す値の候補の上限。異なる値がこれより多い列は候補を出さない。",
    },
    Item {
        name: "poll_ms",
        ty: "integer (1 or more)",
        ty_ja: "整数(1 以上)",
        default: Some("1000"),
        example: "poll_ms = 2000",
        en: "Interval in milliseconds for re-reading notes changed outside mdgrid.",
        ja: "mdgrid の外で変わったノートを読み直す間隔(ミリ秒)。",
    },
    Item {
        name: "ambiguous_wide",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("false"),
        example: "ambiguous_wide = true",
        en: "Treat East Asian Ambiguous characters (such as ○ and ※) as two columns wide.\n\
             Set true if your terminal draws them wide.",
        ja: "East Asian Ambiguous の文字(○・※ など)を幅2として扱う。端末が幅2で描くなら true。",
    },
    Item {
        name: "workspace_detect",
        ty: "array of strings",
        ty_ja: "文字列の並び",
        default: Some("[\"vault\"]"),
        example: "workspace_detect = [\"vault\", \"git\"]",
        en: "When the table you open is in no written workspace, treat the root of the Obsidian vault\n\
             (\"vault\", a folder with .obsidian/) or the git repository (\"git\", a folder with .git) above it\n\
             as a workspace: its folders with notes and .base files become the tables of the relation map.\n\
             [] turns detection off.",
        ja: "書いたワークスペースに入らない表を開いたとき、上にある Obsidian の保管庫(\"vault\"。.obsidian/ のあるフォルダ)か\n\
             git のリポ(\"git\"。.git のあるフォルダ)の根をワークスペースとみなす。直下の、ノートのあるフォルダと .base が\n\
             関係マップの表になる。[] で検知しない。",
    },
    Item {
        name: "look",
        ty: "string",
        ty_ja: "文字列",
        default: Some("\"modern\""),
        example: "look = \"classic\"",
        en: "The look when colors are on: \"modern\" (lazygit-like: tinted selection, accent borders and keys,\n\
             dim labels) or \"classic\" (inverse video). The text on screen is the same; without colors both\n\
             look like classic.",
        ja: "色を使うときの見た目: \"modern\"(lazygit のように、選びは背景の色、枠とキーはアクセントの色、説明は薄い色)\n\
             か \"classic\"(反転)。画面の文字は同じ。色なしではどちらも classic。",
    },
    Item {
        name: "cells",
        ty: "string or table",
        ty_ja: "文字列か表",
        default: Some("\"rich\""),
        example: "cells = \"plain\"",
        en: "How table cells look when colors are on: \"rich\" (booleans as checkboxes, list items and the values\n\
             of short repeated text columns as colored chips, links in the accent color, a type mark in each column\n\
             heading) or \"plain\" (the text as it is). As a [cells] table: style (\"rich\" or \"plain\"), checkbox,\n\
             chips, select, links and icons (true or false for each part), and [cells.columns] with column =\n\
             \"rich\", \"plain\" or \"chip\" (chips even if the column is not detected as one). A column setting wins\n\
             over the part settings. Without colors cells are always plain.",
        ja: "色を使うときの表のセルの見せ方: \"rich\"(真偽はチェックボックス、リストの要素と種類の少ない短い文字の列の\n\
             値は色の付いた札、リンクはアクセントの色、列の見出しに型の印)か \"plain\"(文字のまま)。[cells] の表なら\n\
             style(\"rich\" か \"plain\")、部品ごとの checkbox・chips・select・links・icons(true か false)、\n\
             [cells.columns] に 列 = \"rich\"・\"plain\"・\"chip\"(自動で札にならない列も札に)。列の設定は部品の設定より\n\
             優先。色を使わない表示ではいつも文字のまま。",
    },
    Item {
        name: "view_tabs",
        ty: "string",
        ty_ja: "文字列",
        default: Some("\"always\""),
        example: "view_tabs = \"auto\"",
        en: "When to show the row of view tabs: \"always\" or \"auto\" (only when there are two or more views,\n\
             for example after you save a view; with one view the table gets that row). [display] tabs = false\n\
             hides the tabs either way.",
        ja: "ビューのタブの行を出すとき: \"always\"(いつも)か \"auto\"(ビューが2つ以上のときだけ。たとえばビューを\n\
             保存したあと。1つなら、その行も表に使う)。[display] の tabs = false なら、どちらでも出さない。",
    },
    Item {
        name: "borders",
        ty: "string",
        ty_ja: "文字列",
        default: Some("\"rounded\""),
        example: "borders = \"ascii\"",
        en: "How window frames are drawn: \"rounded\" (connected lines with round corners) or \"ascii\" (+ - |).\n\
             With ambiguous_wide = true the frames are always ASCII, so the columns stay aligned.",
        ja: "窓の枠の描き方: \"rounded\"(角の丸い、つながった罫線)か \"ascii\"(+ - |)。\n\
             ambiguous_wide = true のときは、列がずれないようにいつも ASCII。",
    },
    Item {
        name: "search_bar",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("true"),
        example: "search_bar = false",
        en: "Show a search bar above the table. false hides it; the quick filter is then typed\n\
             on the bottom line.",
        ja: "表の上に検索の欄を出す。false なら出さず、簡易の絞り込みは最下行で打つ。",
    },
    Item {
        name: "date_format",
        ty: "string",
        ty_ja: "文字列",
        default: Some("\"YYYY-MM-DD\""),
        example: "date_format = \"YYYY/MM/DD (ddd)\"",
        en: "How dates are shown and typed in the table. Parts: YYYY, YY, MM, M, DD, D, ddd\n\
             (weekday) and non-alphanumeric separators. Notes are always written as YYYY-MM-DD.",
        ja: "表の日付の見せ方と打ち込みの形。部品は YYYY・YY・MM・M・DD・D・ddd(曜日)と英数字でない区切り。\
             ノートにはいつも YYYY-MM-DD で書く。",
    },
    Item {
        name: "week_start",
        ty: "string (\"sun\" or \"mon\")",
        ty_ja: "文字列(\"sun\" か \"mon\")",
        default: Some("\"sun\""),
        example: "week_start = \"mon\"",
        en: "First day of the week in the date calendar.",
        ja: "日付のカレンダーの週の始まり。",
    },
    Item {
        name: "add_frontmatter",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("true"),
        example: "add_frontmatter = false",
        en: "Allow writing to notes with no front matter or an empty one (a front matter or a key\n\
             line is added). false makes those notes read-only.",
        ja: "フロントマターの無いノートと空のフロントマターのノートにも書く(フロントマターかキーの行を足す)。\
             false ならこの2つは読むだけ。",
    },
    Item {
        name: "editor",
        ty: "string",
        ty_ja: "文字列",
        default: None,
        example: "editor = \"nvim\"",
        en: "Editor that opens the selected note (arguments allowed, e.g. \"code -w\"; run without\n\
             a shell). Unset or blank: $VISUAL, then $EDITOR, then vi.",
        ja: "選んだノートを開くエディタ(引数つきでよい。例 \"code -w\"。シェルは通さない)。\
             無いか空(空白だけも)なら $VISUAL、$EDITOR、vi の順。",
    },
    Item {
        name: "language",
        ty: "string (\"auto\", \"en\" or \"ja\")",
        ty_ja: "文字列(\"auto\"・\"en\"・\"ja\")",
        default: Some("\"auto\""),
        example: "language = \"en\"",
        en: "Language of the screen and startup messages. \"auto\" follows LC_ALL, LC_MESSAGES\n\
             and LANG (the first non-empty one): Japanese if it starts with ja, otherwise English.",
        ja: "画面と起動の文言の言語。\"auto\" は LC_ALL・LC_MESSAGES・LANG の順で最初の空でない値が\
             ja で始まれば日本語、ほかは英語。",
    },
    Item {
        name: "theme",
        ty: "string (\"auto\", \"default\", \"nord\", \"solarized-light\", \"dracula\", \"gruvbox\", \"pink-monster\", \"dozy-pink\", \"sumi\", \"slate\", \"saas\", \"saas-dark\" or \"paper\")",
        ty_ja: "文字列(\"auto\"・\"default\"・\"nord\"・\"solarized-light\"・\"dracula\"・\"gruvbox\"・\"pink-monster\"・\"dozy-pink\"・\"sumi\"・\"slate\"・\"saas\"・\"saas-dark\"・\"paper\")",
        default: Some("\"default\""),
        example: "theme = \"nord\"",
        en: "Color theme of the screen. \"default\" keeps the terminal's own colors. \"auto\" picks\n\
             theme_light or theme_dark by the terminal's background. Ignored without colors (NO_COLOR,\n\
             --no-color, TERM=dumb, color = false); 256-color terminals get the nearest colors.",
        ja: "画面の色のテーマ。\"default\" は端末の色のまま。\"auto\" は端末の地の明るさで theme_light か \
             theme_dark を使う。色なし(NO_COLOR・--no-color・TERM=dumb・color = false)では効かない。256 色の\
             端末では近い色で塗る。",
    },
    Item {
        name: "theme_light",
        ty: "string (a theme name)",
        ty_ja: "文字列(テーマの名前)",
        default: Some("\"saas\""),
        example: "theme_light = \"paper\"",
        en: "With theme = \"auto\", the theme for a light terminal background.",
        ja: "theme = \"auto\" のとき、端末の地が明るいときのテーマ。",
    },
    Item {
        name: "theme_dark",
        ty: "string (a theme name)",
        ty_ja: "文字列(テーマの名前)",
        default: Some("\"sumi\""),
        example: "theme_dark = \"saas-dark\"",
        en: "With theme = \"auto\", the theme for a dark terminal background (also used when the\n\
             background cannot be told).",
        ja: "theme = \"auto\" のとき、端末の地が暗いときのテーマ(明るさが分からないときも)。",
    },
    Item {
        name: "style",
        ty: "table ([style] with preset, status, tags, check, select, rules, tabs, frames, band and icons)",
        ty_ja: "表([style] の下に preset・status・tags・check・select・rules・tabs・frames・band・icons)",
        default: Some("{ preset = \"sumi\" }"),
        example: "[style]\npreset = \"saas\"\nselect = \"cross\"",
        en: "The shape of each part of the screen. preset picks a whole set (\"sumi\", \"slate\", \"saas\",\n\
             \"paper\", \"grid\", \"classic\", \"dozy-pink\"); the other items override single parts: status (\"dot\", \"shape\",\n\
             \"text\", \"pill\", \"tint\", \"solid\", \"soft\", \"chip\", \"plain\"), tags (\"dots\", \"hash\", \"brackets\", \"pill\",\n\
             \"tint\", \"solid\", \"soft\", \"chip\", \"plain\"), check (\"box\", \"tick\", \"bracket\", \"text\"), select (\"bar\",\n\
             \"cross\", \"tint\", \"outline\", \"fill\", \"reverse\"), rules (\"none\", \"header\", \"columns\", \"grid\"),\n\
             tabs (\"underline\", \"pill\", \"segment\", \"brackets\", \"dim\"), frames (\"rounded\", \"square\",\n\
             \"heavy\", \"ascii\", \"none\"), band (\"keys\", \"boxed\", \"quiet\") and icons (true or false).\n\
             docs/catalog/index.html shows every choice and writes this table for you.",
        ja: "画面の部品の形。preset で組をまとめて選び(\"sumi\"・\"slate\"・\"saas\"・\"paper\"・\"grid\"・\"classic\"・\"dozy-pink\")、\
             ほかの項目で部品を1つずつ上書きする: status(\"dot\"・\"shape\"・\"text\"・\"pill\"・\"tint\"・\"solid\"・\"soft\"・\"chip\"・\
             \"plain\")・tags(\"dots\"・\"hash\"・\"brackets\"・\"pill\"・\"tint\"・\"solid\"・\"soft\"・\"chip\"・\"plain\")・check(\"box\"・\
             \"tick\"・\"bracket\"・\"text\")・select(\"bar\"・\"cross\"・\"tint\"・\"outline\"・\"fill\"・\"reverse\")・rules(\"none\"・\
             \"header\"・\"columns\"・\"grid\")・tabs(\"underline\"・\"pill\"・\"segment\"・\"brackets\"・\"dim\")・frames(\
             \"rounded\"・\"square\"・\"heavy\"・\"ascii\"・\"none\")・band(\"keys\"・\"boxed\"・\"quiet\")・icons(true か false)。\
             docs/catalog/index.html で全部の形を見比べ、この表を作れる。",
    },
    Item {
        name: "colors",
        ty: "table ([colors] with color roles, and [colors.values] for value colors)",
        ty_ja: "表([colors] の下に色の役割、[colors.values] に値の色)",
        default: Some("{}"),
        example: "[colors]\naccent = \"#e0a458\"\n\n[colors.values]\ndone = \"green\"",
        en: "Override the theme's colors by role: background, text, header, selection, selection_text,\n\
             band, band_text, accent, strong, zebra, zebra_text, mark, added, removed, pending and\n\
             highlight. [colors.values] sets a color per value (case-insensitive), used by every part\n\
             shape. Colors are \"#rrggbb\", \"#rgb\" or a name (black, white, gray, red, orange,\n\
             yellow, green, teal, cyan, blue, purple, magenta, pink, brown). With theme = \"default\",\n\
             only accent, selection and value colors are used.",
        ja: "テーマの色を役割ごとに上書きする: background・text・header・selection・selection_text・band・\
             band_text・accent・strong・zebra・zebra_text・mark・added・removed・pending・highlight。\
             [colors.values] は値ごとの色(大文字・小文字によらない)で、部品のどの形でも使う。色は \"#rrggbb\"・\
             \"#rgb\" か名前(black・white・gray・red・orange・yellow・green・teal・cyan・blue・purple・magenta・\
             pink・brown)。theme = \"default\" では accent・selection と値の色だけを使う。",
    },
    Item {
        name: "nerd_font",
        ty: "boolean or \"auto\"",
        ty_ja: "真偽か \"auto\"",
        default: Some("\"auto\""),
        example: "nerd_font = true",
        en: "Whether round pill ends (Nerd Font glyphs) can be drawn. true: your terminal font is a Nerd\n\
             Font. \"auto\": only in terminals that draw these glyphs themselves (Ghostty, WezTerm), so\n\
             nothing breaks elsewhere. Without round ends, \"pill\" is drawn as \"soft\".",
        ja: "丸い札の端(Nerd Font の字)を描いてよいか。true は端末の字形が Nerd Font。\"auto\" は、その字を\
             字体に頼らず自分で描く端末(Ghostty・WezTerm)のときだけ(ほかの端末で字が化けないように)。\
             描けなければ \"pill\" は \"soft\" で描く。",
    },
    Item {
        name: "display",
        ty: "table ([display] with row_numbers, zebra, column_lines, group_gap, tabs and chips)",
        ty_ja: "表([display] の下に row_numbers・zebra・column_lines・group_gap・tabs・chips)",
        default: Some(
            "{ row_numbers = false, zebra = false, column_lines = false, group_gap = false, tabs = true, \
             chips = true }",
        ),
        example: "[display]\nrow_numbers = true\nzebra = true\ncolumn_lines = true",
        en: "How the table is shown (each true or false): row_numbers (numbers 1, 2, 3... on the\n\
             left), zebra (every other row shaded; not without colors), column_lines (a │ between\n\
             columns), group_gap (an empty line above every group heading but the first),
             tabs (the view tabs) and chips (the band of active view settings).\n\
             The search bar stays the top-level search_bar. Each view can override these in\n\
             its view settings (o).",
        ja: "表の見せ方(どれも true か false)。row_numbers(左に 1・2・3… の行番号)・zebra(一行おきの色。\
             色なしでは付けない)・column_lines(列の間に │)・group_gap(2つ目からのまとまりの見出しの上に空きの行)・tabs(ビューのタブ)・chips(設定の帯)。\
             検索の欄は最上位の search_bar のまま。ビューごとにビューの設定(o)で切り替えられる。",
    },
    Item {
        name: "new_note",
        ty: "table ([new_note] with mode, folder, name, ask, required, hidden, body and [new_note.set])",
        ty_ja: "表([new_note] の下に mode・folder・name・ask・required・hidden・body と [new_note.set])",
        default: Some("{}"),
        example: "[new_note]\nfolder = \"inbox\"\nname = \"{date} \"\nask = [\"priority\", \"due\"]\nrequired = [\"due\"]\nhidden = [\"created\"]\nbody = \"templates/note.md\"\n\n[new_note.set]\ntags = [\"inbox\"]\ndue = \"{date+7}\"\ncreated = \"{now}\"",
        en: "How a new note is made from the table: mode (\"form\": a form with every field, or\n\
             \"editor\": create and open in the editor), folder (relative to\n\
             the opened folder), name (name template), ask (the form's columns; default: the\n\
             visible columns), required (columns that must be filled), hidden (columns written\n\
             without a field), body (a body template file) and [new_note.set] (column = value).\n\
             Templates may use {date}, {date+7}, {date:YYYY/MM/DD}, {time}, {now}, {weekday},\n\
             {name} and {folder}. Values from the view's filter come first. A mdgrid view can\n\
             override this with [target.view.new_note].",
        ja: "表から新しいノートを作るときの決まり。mode(\"form\" は全部の欄を並べた窓、\"editor\" は作ってすぐエディタで開く)・folder(開いたフォルダからの相対)・\
             name(名前の雛形)・ask(窓に出す列。既定は見えている列)・required(空では作らない列)・\
             hidden(欄を出さずに値だけ入れる列)・body(本文の雛形のファイル)・[new_note.set](列 = 値)。\
             雛形には {date}・{date+7}・{date:YYYY/MM/DD}・{time}・{now}・{weekday}・{name}・{folder} を書ける。\
             ビューの絞り込みの値が先。mdgrid のビューでは [target.view.new_note] で置き換えられる。",
    },
    Item {
        name: "keys",
        ty: "table of tables ([keys.<mode>] with key = action)",
        ty_ja: "表の表([keys.<モード>] の下に キー = 動作)",
        default: None,
        example: "[keys.table]\n\"ctrl+f\" = \"search\"\n\"x\" = \"none\"",
        en: "Key rebinding per mode: under [keys.<mode>], write key = action name.\n\
             \"none\" removes the key. Modes: table, edit, review, quit, help, palette, search,\n\
             filter, detail, settings, settings_input, chips, list_select, menu, freq, sorts, relations. Action\n\
             names are shown in the command palette (:).",
        ja: "モードごとのキーの割り当て直し。[keys.<モード>] の下に キー = 動作の名前。\
             \"none\" でそのキーを外す。モードは table・edit・review・quit・help・palette・search・\
             filter・detail・settings・settings_input・chips・list_select・menu・freq・sorts・relations。動作の名前はコマンドのパレット(:)に出る。",
    },
];

/// 実装が読む設定の最上位の項目の名前の一覧(`ITEMS` の名前。CLI-12)。
pub const KEYS: &[&str] = &{
    let mut names = [""; ITEMS.len()];
    let mut i = 0;
    while i < ITEMS.len() {
        names[i] = ITEMS[i].name;
        i += 1;
    }
    names
};
