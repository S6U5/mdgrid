//! 設定の項目の表(CLI-3・CLI-11・CLI-12・CLI-20)。形は docs/design.md の「設定の形と範囲ごとの上書き」。
//!
//! 項目の一覧はここ1か所に持ち、読み取りの知らない項目の判定(`config::parse`)・`--print-config`
//! (`config::default_toml`)・文書との突き合わせの試験・カタログの試験がこれを使う。旧い名前(0.3.0 まで)と
//! 移った先の対応も `LEGACY` に持ち、読み取りの写し(`legacy::lift`)と文書の表の試験が使う。

/// 書ける範囲(CLI-11・SR-44)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// 全体(config.toml)だけ。アプリ全体の項目。
    Global,
    /// 表のプロファイル: 全体・ワークスペース・表・ビュー。
    Profile,
}

impl Scope {
    /// 文書の「書ける範囲」の行と `--print-config` のコメント(英語)。
    pub fn en(self) -> &'static str {
        match self {
            Scope::Global => "global only",
            Scope::Profile => "global, workspace, table, view",
        }
    }

    /// 文書の「書ける範囲」の行(日本語)。
    pub fn ja(self) -> &'static str {
        match self {
            Scope::Global => "全体だけ",
            Scope::Profile => "全体・ワークスペース・表・ビュー",
        }
    }
}

/// 設定の項目の表の1行。
#[derive(Debug, Clone, Copy)]
pub struct Item {
    /// 項目の道筋(`look.theme`)。区画の下の項目は `区画.名前`、表の項目(`look.style`・`new_note`)は表の道筋。
    pub path: &'static str,
    /// 値の型(英語)。docs/config.md の `- Type: `…`` の行と同じ(試験で突き合わせる)。
    pub ty: &'static str,
    /// 値の型(日本語)。docs/config.ja.md の `- 型: `…`` の行と同じ。
    pub ty_ja: &'static str,
    /// 既定値の TOML の表記。None は「既定では書かない」項目で、`example` をコメントで出す。
    pub default: Option<&'static str>,
    /// 書き方の例(そのまま設定に貼れる TOML。区画の下の項目は区画の見出しから)。
    pub example: &'static str,
    /// 書ける範囲。
    pub scope: Scope,
    /// 英語の説明(`--print-config` のコメント)。
    pub en: &'static str,
    /// 日本語の説明。
    pub ja: &'static str,
}

impl Item {
    /// 区画の名前(最上位の項目は "")。表の項目(既定なしで見出しの例を持つもの)も、その道筋の親。
    pub fn section(&self) -> &'static str {
        match self.path.rfind('.') {
            Some(i) => &self.path[..i],
            None => "",
        }
    }

    /// 区画の中の名前。
    pub fn leaf(&self) -> &'static str {
        match self.path.rfind('.') {
            Some(i) => &self.path[i + 1..],
            None => self.path,
        }
    }

    /// 表の項目(`[look.style]` のように、その道筋の見出しで書く項目)か。
    pub fn is_table(&self) -> bool {
        self.example.starts_with(&format!("[{}", self.path))
    }
}

/// 設定の項目の表。`--print-config` はこの順で出す(最上位 → 区画の順)。
pub const ITEMS: &[Item] = &[
    Item {
        path: "language",
        ty: "string (\"auto\", \"en\" or \"ja\")",
        ty_ja: "文字列(\"auto\"・\"en\"・\"ja\")",
        default: Some("\"auto\""),
        example: "language = \"en\"",
        scope: Scope::Global,
        en: "Language of the screen and startup messages. \"auto\" follows LC_ALL, LC_MESSAGES\n\
             and LANG (the first non-empty one): Japanese if it starts with ja, otherwise English.",
        ja: "画面と起動の文言の言語。\"auto\" は LC_ALL・LC_MESSAGES・LANG の順で最初の空でない値が\
             ja で始まれば日本語、ほかは英語。",
    },
    Item {
        path: "editor",
        ty: "string",
        ty_ja: "文字列",
        default: None,
        example: "editor = \"nvim\"",
        scope: Scope::Global,
        en: "Editor that opens the selected note (arguments allowed, e.g. \"code -w\"; run without\n\
             a shell). Unset or blank: $VISUAL, then $EDITOR, then vi (notepad on Windows).",
        ja: "選んだノートを開くエディタ(引数つきでよい。例 \"code -w\"。シェルは通さない)。\
             無いか空(空白だけも)なら $VISUAL、$EDITOR、vi(Windows は notepad)の順。",
    },
    Item {
        path: "poll_ms",
        ty: "integer (1 or more)",
        ty_ja: "整数(1 以上)",
        default: Some("1000"),
        example: "poll_ms = 2000",
        scope: Scope::Global,
        en: "Interval in milliseconds for re-reading notes changed outside mdgrid.",
        ja: "mdgrid の外で変わったノートを読み直す間隔(ミリ秒)。",
    },
    Item {
        path: "use",
        ty: "string (a template name)",
        ty_ja: "文字列(テンプレートの名前)",
        default: None,
        example: "use = \"night\"",
        scope: Scope::Profile,
        en: "Lay the template [templates.<name>] under the values written in this place.",
        ja: "テンプレート [templates.<名前>] を、この場所に書いた値の下に敷く。",
    },
    Item {
        path: "types",
        ty: "table (column = \"text\", \"number\", \"checkbox\", \"date\", \"datetime\", \"list\" or \"auto\")",
        ty_ja: "表(列 = \"text\"・\"number\"・\"checkbox\"・\"date\"・\"datetime\"・\"list\"・\"auto\")",
        default: Some("{}"),
        example: "[types]\ndue = \"date\"\nestimate = \"number\"",
        scope: Scope::Profile,
        en: "The type of single columns. It wins over the root folder's type settings (.obsidian/types.json)\n\
             and over guessing from the values; \"auto\" goes back to guessing (a narrower scope can undo a\n\
             wider one). The type sets the cell input, the ! mark on values that do not fit, sorting and the\n\
             type mark of the column heading. Notes are not changed. Clicking the type mark of a heading, or\n\
             column_type in the palette or the action menu, picks a type and writes it to the table in\n\
             views.toml.",
        ja: "列ごとの型。ルートフォルダの型の設定(.obsidian/types.json)と、値からの推定より優先する。\"auto\" は推定に\
             戻す(狭い範囲で広い範囲の型を外せる)。型はセルの入力・合わない値の ! の印・並べ替え・列の見出しの型の印に\
             効く。ノートは書き換えない。見出しの型の印のクリック、パレットか操作の一覧の column_type で型を選ぶと、\
             views.toml の表に書く。",
    },
    Item {
        path: "rules",
        ty: "table of tables (column = { required = boolean, values = [string, …] })",
        ty_ja: "表の表(列 = { required = 真偽値, values = [文字列, …] })",
        default: Some("{}"),
        example: "[rules.status]\nrequired = true\nvalues = [\"todo\", \"doing\", \"done\"]",
        scope: Scope::Profile,
        en: "Rules for single columns, checked together with the column type. required = true makes a\n\
             missing key, null, an empty string or an empty list a problem; values lists the allowed values\n\
             (each item of a list must be one of them, case-sensitive). A narrower scope wins item by item.\n\
             Problems get the ! mark, the bottom band counts them, ! and @ jump between them, and\n\
             mdgrid --check lists them. Rules never change notes or stop you writing a value; a column with\n\
             values offers them in the cell input. Computed columns (file.*, formula.*) cannot have rules.",
        ja: "列ごとの決まり。列の型と合わせて確かめる。required = true は、キーが無い・null・空の文字列・空のリストを問題に\
             する。values は選べる値の並び(リストは各要素がこのどれか。大文字小文字を区別する)。狭い範囲が項目ごとに勝つ。\
             問題には ! の印が付き、下の帯に数が出て、! と @ で跳べ、mdgrid --check で並べて出せる。決まりはノートを\
             書き換えず、値を書くことも止めない。values のある列は、セルの入力の候補にその値を並べる。計算の列(file.*・\
             formula.*)には書けない。",
    },
    Item {
        path: "terminal.color",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("true"),
        example: "[terminal]\ncolor = false",
        scope: Scope::Global,
        en: "Use colors. false draws without colors (same as NO_COLOR or --no-color).",
        ja: "色を使う。false で色なし(NO_COLOR・--no-color と同じ)。",
    },
    Item {
        path: "terminal.ambiguous_wide",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("false"),
        example: "[terminal]\nambiguous_wide = true",
        scope: Scope::Global,
        en: "Treat East Asian Ambiguous characters (such as ○ and ※) as two columns wide.\n\
             Set true if your terminal draws them wide. Window frames are then always ASCII.",
        ja: "East Asian Ambiguous の文字(○・※ など)を幅2として扱う。端末が幅2で描くなら true。\
             このとき窓の枠はいつも ASCII。",
    },
    Item {
        path: "terminal.nerd_font",
        ty: "boolean or \"auto\"",
        ty_ja: "真偽か \"auto\"",
        default: Some("\"auto\""),
        example: "[terminal]\nnerd_font = true",
        scope: Scope::Global,
        en: "Whether round pill ends (Nerd Font glyphs) can be drawn. true: your terminal font is a Nerd\n\
             Font. \"auto\": only in terminals that draw these glyphs themselves (Ghostty, WezTerm), so\n\
             nothing breaks elsewhere. Without round ends, \"pill\" is drawn as \"soft\".",
        ja: "丸い札の端(Nerd Font の字)を描いてよいか。true は端末の字形が Nerd Font。\"auto\" は、その字を\
             字体に頼らず自分で描く端末(Ghostty・WezTerm)のときだけ(ほかの端末で字が化けないように)。\
             描けなければ \"pill\" は \"soft\" で描く。",
    },
    Item {
        path: "terminal.nerd_icons",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("false"),
        example: "[terminal]\nnerd_icons = true",
        scope: Scope::Global,
        en: "Use icon glyphs from a Nerd Font for the settings button, the screen type button and menu, the\n\
             type marks of column headings and the calendar title. Set true only if your terminal font is a\n\
             Nerd Font; by default plain characters are used (⚙, ▦, ◇, #, ◷, ⋮, ◉, ✓). Emoji are never used.",
        ja: "設定のボタン・画面の型のボタンと窓・列の見出しの型の印・カレンダーの題に、アイコン用のフォント(Nerd Font)の\
             記号を使う。端末の字形が Nerd Font のときだけ true にする。既定は字の記号(⚙・▦・◇・#・◷・⋮・◉・✓)。\
             絵文字は使わない。",
    },
    Item {
        path: "workspace.detect",
        ty: "array of strings",
        ty_ja: "文字列の並び",
        default: Some("[\".obsidian\"]"),
        example: "[workspace]\ndetect = [\".obsidian\", \".git\"]",
        scope: Scope::Global,
        en: "When the table you open is in no written workspace, treat the nearest folder above it that\n\
             has one of these marker folders as a workspace (the root folder): \".obsidian\" (a folder with\n\
             .obsidian/) or \".git\" (a git repository). Its folders with notes become the tables of the\n\
             relation map. [] turns detection off. The old values \"vault\" and \"git\" are read the same\n\
             way, with a warning.",
        ja: "書いたワークスペースに入らない表を開いたとき、上にたどって、この目印のフォルダを持つ最寄りのフォルダ\
             (ルートフォルダ)をワークスペースとみなす: \".obsidian\"(.obsidian/ のあるフォルダ)か \".git\"(git のリポ)。\
             直下の、ノートのあるフォルダが関係マップの表になる。[] で検知しない。旧い値 \"vault\"・\"git\" も\
             同じ意味で読み、警告する。",
    },
    Item {
        path: "look.theme",
        ty: "string (\"auto\", \"default\", \"nord\", \"solarized-light\", \"dracula\", \"gruvbox\", \"pink-monster\", \"dozy-pink\", \"sumi\", \"slate\", \"saas\", \"saas-dark\" or \"paper\") or table ({ light, dark })",
        ty_ja: "文字列(\"auto\"・\"default\"・\"nord\"・\"solarized-light\"・\"dracula\"・\"gruvbox\"・\"pink-monster\"・\"dozy-pink\"・\"sumi\"・\"slate\"・\"saas\"・\"saas-dark\"・\"paper\")か表({ light, dark })",
        default: Some("\"default\""),
        example: "[look]\ntheme = { light = \"paper\", dark = \"sumi\" }",
        scope: Scope::Profile,
        en: "Color theme of the screen. \"default\" keeps the terminal's own colors. A table\n\
             { light = ..., dark = ... } picks by the terminal's background; \"auto\" is\n\
             { light = \"saas\", dark = \"sumi\" }. Ignored without colors; 256-color terminals get\n\
             the nearest colors.",
        ja: "画面の色のテーマ。\"default\" は端末の色のまま。表 { light = …, dark = … } は端末の地の明るさで選び、\
             \"auto\" は { light = \"saas\", dark = \"sumi\" } と同じ。色なしでは効かない。256 色の端末では近い色で塗る。",
    },
    Item {
        path: "look.preset",
        ty: "string (\"sumi\", \"slate\", \"saas\", \"paper\", \"grid\", \"classic\" or \"dozy-pink\")",
        ty_ja: "文字列(\"sumi\"・\"slate\"・\"saas\"・\"paper\"・\"grid\"・\"classic\"・\"dozy-pink\")",
        default: Some("\"sumi\""),
        example: "[look]\npreset = \"saas\"",
        scope: Scope::Profile,
        en: "A whole set of part shapes. [look.style] overrides single parts. A preset written in a\n\
             narrower place drops the part shapes written in wider places.",
        ja: "部品の形の組。[look.style] で部品を1つずつ上書きする。狭い範囲に書いた組は、\
             広い範囲の部品の形を使わない。",
    },
    Item {
        path: "look.mode",
        ty: "string (\"modern\" or \"classic\")",
        ty_ja: "文字列(\"modern\" か \"classic\")",
        default: Some("\"modern\""),
        example: "[look]\nmode = \"classic\"",
        scope: Scope::Profile,
        en: "The look when colors are on: \"modern\" (lazygit-like: tinted selection, accent borders and keys,\n\
             dim labels) or \"classic\" (inverse video). The text on screen is the same; without colors both\n\
             look like classic.",
        ja: "色を使うときの見た目: \"modern\"(lazygit のように、選びは背景の色、枠とキーはアクセントの色、説明は薄い色)\
             か \"classic\"(反転)。画面の文字は同じ。色なしではどちらも classic。",
    },
    Item {
        path: "look.cells",
        ty: "string (\"rich\" or \"plain\")",
        ty_ja: "文字列(\"rich\" か \"plain\")",
        default: Some("\"rich\""),
        example: "[look]\ncells = \"plain\"",
        scope: Scope::Profile,
        en: "How table cells look when colors are on: \"rich\" (booleans, lists, short repeated values,\n\
             links and column type marks drawn as parts in the shapes of [look.style]) or \"plain\" (the\n\
             text as it is). Without colors cells are always plain.",
        ja: "色を使うときの表のセルの見せ方: \"rich\"(真偽・リスト・くり返す短い値・リンク・列の型の印を [look.style] の\
             形の部品で見せる)か \"plain\"(文字のまま)。色を使わない表示ではいつも文字のまま。",
    },
    Item {
        path: "look.style",
        ty: "table (status, tags, check, select, rules, tabs, frames, band, links and icons)",
        ty_ja: "表(status・tags・check・select・rules・tabs・frames・band・links・icons)",
        default: None,
        example: "[look.style]\nstatus = \"chip\"\nselect = \"cross\"",
        scope: Scope::Profile,
        en: "Override single part shapes of the preset: status (\"dot\", \"shape\", \"text\", \"pill\", \"tint\",\n\
             \"solid\", \"soft\", \"chip\", \"plain\"), tags (\"dots\", \"hash\", \"brackets\", \"pill\", \"tint\",\n\
             \"solid\", \"soft\", \"chip\", \"plain\"), check (\"box\", \"tick\", \"bracket\", \"text\"), select\n\
             (\"bar\", \"cross\", \"tint\", \"outline\", \"fill\", \"reverse\"), rules (\"none\", \"header\",\n\
             \"columns\", \"grid\"), tabs (\"underline\", \"pill\", \"segment\", \"brackets\", \"dim\"), frames\n\
             (\"rounded\", \"square\", \"heavy\", \"ascii\", \"none\"), band (\"keys\", \"boxed\", \"quiet\"), links\n\
             (\"accent\", \"plain\") and icons (true or false). \"plain\" (\"text\" for check) keeps that part as\n\
             text. docs/catalog/index.html shows every choice and writes this table for you.",
        ja: "組の部品の形を1つずつ上書きする: status(\"dot\"・\"shape\"・\"text\"・\"pill\"・\"tint\"・\"solid\"・\"soft\"・\"chip\"・\
             \"plain\")・tags(\"dots\"・\"hash\"・\"brackets\"・\"pill\"・\"tint\"・\"solid\"・\"soft\"・\"chip\"・\"plain\")・check(\"box\"・\
             \"tick\"・\"bracket\"・\"text\")・select(\"bar\"・\"cross\"・\"tint\"・\"outline\"・\"fill\"・\"reverse\")・rules(\"none\"・\
             \"header\"・\"columns\"・\"grid\")・tabs(\"underline\"・\"pill\"・\"segment\"・\"brackets\"・\"dim\")・frames(\
             \"rounded\"・\"square\"・\"heavy\"・\"ascii\"・\"none\")・band(\"keys\"・\"boxed\"・\"quiet\")・links(\"accent\"・\"plain\")・\
             icons(true か false)。\"plain\"(check は \"text\")はその部品を文字のままにする。\
             docs/catalog/index.html で全部の形を見比べ、この表を作れる。",
    },
    Item {
        path: "look.columns",
        ty: "table (column = \"rich\", \"plain\" or \"chip\")",
        ty_ja: "表(列 = \"rich\"・\"plain\"・\"chip\")",
        default: None,
        example: "[look.columns]\nstatus = \"plain\"\nowner = \"chip\"",
        scope: Scope::Profile,
        en: "How single columns look: \"rich\", \"plain\" or \"chip\" (chips even if the column is not\n\
             detected as one). A column setting wins over the part shapes.",
        ja: "列ごとの見せ方: \"rich\"・\"plain\"・\"chip\"(自動で札にならない列も札に)。列の設定は部品の形より優先。",
    },
    Item {
        path: "look.colors",
        ty: "table (color roles, and [look.colors.values] for value colors)",
        ty_ja: "表(色の役割と、[look.colors.values] に値の色)",
        default: None,
        example: "[look.colors]\naccent = \"#e0a458\"\n\n[look.colors.values]\ndone = \"green\"",
        scope: Scope::Profile,
        en: "Override the theme's colors by role: background, text, header, selection, selection_text,\n\
             band, band_text, accent, strong, zebra, zebra_text, mark, added, removed, pending and\n\
             highlight. [look.colors.values] sets a color per value (case-insensitive), used by every part\n\
             shape. Colors are \"#rrggbb\", \"#rgb\" or a name (black, white, gray, red, orange,\n\
             yellow, green, teal, cyan, blue, purple, magenta, pink, brown). With theme = \"default\",\n\
             only accent, selection and value colors are used. A theme written in a narrower place drops\n\
             the role colors written in wider places.",
        ja: "テーマの色を役割ごとに上書きする: background・text・header・selection・selection_text・band・\
             band_text・accent・strong・zebra・zebra_text・mark・added・removed・pending・highlight。\
             [look.colors.values] は値ごとの色(大文字・小文字によらない)で、部品のどの形でも使う。色は \"#rrggbb\"・\
             \"#rgb\" か名前(black・white・gray・red・orange・yellow・green・teal・cyan・blue・purple・magenta・\
             pink・brown)。theme = \"default\" では accent・selection と値の色だけを使う。狭い範囲に書いたテーマは、\
             広い範囲の役割の色を使わない。",
    },
    Item {
        path: "display.row_numbers",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("false"),
        example: "[display]\nrow_numbers = true",
        scope: Scope::Profile,
        en: "Numbers 1, 2, 3... on the left of each row, in the shown order.",
        ja: "各行の左に、今の表示の並びの 1・2・3… の行番号。",
    },
    Item {
        path: "display.zebra",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("false"),
        example: "[display]\nzebra = true",
        scope: Scope::Profile,
        en: "Shade every other row (not without colors).",
        ja: "一行おきに背景の色を付ける(色を使わない表示では付けない)。",
    },
    Item {
        path: "display.column_lines",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("false"),
        example: "[display]\ncolumn_lines = true",
        scope: Scope::Profile,
        en: "Draw │ between columns.",
        ja: "列の間に │ を引く。",
    },
    Item {
        path: "display.group_gap",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("false"),
        example: "[display]\ngroup_gap = true",
        scope: Scope::Profile,
        en: "An empty line above every group heading but the first.",
        ja: "2つ目からのまとまりの見出しの上に空きの行を1つ入れる。",
    },
    Item {
        path: "display.tabs",
        ty: "string (\"always\", \"auto\" or \"never\")",
        ty_ja: "文字列(\"always\"・\"auto\"・\"never\")",
        default: Some("\"always\""),
        example: "[display]\ntabs = \"auto\"",
        scope: Scope::Profile,
        en: "The row of view tabs: \"always\", \"auto\" (only with two or more views; with one view the\n\
             table gets that row) or \"never\". The tab keys work either way.",
        ja: "ビューのタブの行: \"always\"(いつも)・\"auto\"(ビューが2つ以上のときだけ。1つなら、その行も表に使う)・\
             \"never\"(出さない)。どれでもタブの切り替えのキーは効く。",
    },
    Item {
        path: "display.search_bar",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("true"),
        example: "[display]\nsearch_bar = false",
        scope: Scope::Profile,
        en: "Show a search bar above the table. false hides it; the quick filter is then typed\n\
             on the bottom line.",
        ja: "表の上に検索の欄を出す。false なら出さず、簡易の絞り込みは最下行で打つ。",
    },
    Item {
        path: "display.chips",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("true"),
        example: "[display]\nchips = false",
        scope: Scope::Profile,
        en: "Show the band of active view settings. f still picks its items when hidden.",
        ja: "効いているビューの設定の帯を出す。隠しても f で帯の項目を選べる。",
    },
    Item {
        path: "display.buttons",
        ty: "string (\"plain\" or \"box\")",
        ty_ja: "文字列(\"plain\" か \"box\")",
        default: Some("\"plain\""),
        example: "[display]\nbuttons = \"box\"",
        scope: Scope::Profile,
        en: "How the search bar and the Filter and Sort buttons in the view bar are drawn: \"plain\"\n\
             (shown by their background) or \"box\" (each in a box drawn with lines on the padding lines\n\
             of the view bar, not filled). With no padding (display.bar_padding = 0) no lines are drawn.",
        ja: "ビューの帯の検索の欄と絞り込み・並べ替えのボタンの描き方: \"plain\"(地の色で示す)か \"box\"(それぞれを\
             帯の上下の余白の行に引いた線で四角く囲む。中は塗らない)。余白が無い(display.bar_padding = 0)と線は\
             描かない。",
    },
    Item {
        path: "display.bar_padding",
        ty: "number (0 to 2 in steps of 0.5)",
        ty_ja: "数(0〜2 の 0.5 刻み)",
        default: Some("1"),
        example: "[display]\nbar_padding = 0.5",
        scope: Scope::Profile,
        en: "Padding above and below the view bar, in lines: 0, 0.5, 1, 1.5 or 2 (1 and 1.0 both work).\n\
             0 draws the view bar on one line. A half line gets the view bar's background color with its\n\
             outer half covered by half blocks (▀ above, ▄ below) in the screen's background color (the\n\
             theme's, or the terminal's, asked once at startup); when that is unknown, without colors, and\n\
             with terminal.ambiguous_wide = true, it is rounded up to an empty line.\n\
             Terminals shorter than 16 lines, and a hidden search bar, always use 0. Other values are\n\
             warned about and the default is used.",
        ja: "ビューの帯の上下の余白の行の数: 0・0.5・1・1.5・2(1 と 1.0 のどちらも書ける)。0 なら帯は余白なしの1行。\
             0.5 の端数は、帯の地の色で塗り外側の半分を画面の地の色の半分の塗りの字(上は ▀、下は ▄)で消した行で描き、\
             帯の行に地の色を敷く。画面の地の色が分からない(テーマが画面の地を塗らず、端末も答えない)ときと、色を\
             使わない表示と terminal.ambiguous_wide = true では、端数を1行の空き行に切り上げる。端末の高さが16行に満たないときと、\
             検索の欄を出さないときは、いつも 0。ほかの値は警告して既定にする。",
    },
    Item {
        path: "display.top_padding",
        ty: "number (0 to 2 in steps of 0.5)",
        ty_ja: "数(0〜2 の 0.5 刻み)",
        default: Some("1"),
        example: "[display]\ntop_padding = 0",
        scope: Scope::Profile,
        en: "Padding above the header, in lines: 0, 0.5, 1, 1.5 or 2. A half line is drawn like\n\
             display.bar_padding (▀ above the header, which then gets the view bar's background color).\n\
             Terminals shorter than 16 lines always use 0.",
        ja: "ヘッダーの上の余白の行の数: 0・0.5・1・1.5・2。端数は display.bar_padding と同じく描く(ヘッダーの上に ▀ の行、\
             ヘッダーの行に帯の地の色)。端末の高さが16行に満たないときは、いつも 0。",
    },
    Item {
        path: "display.row_extra_lines",
        ty: "integer (0 to 9)",
        ty_ja: "整数(0〜9)",
        default: Some("0"),
        example: "[display]\nrow_extra_lines = 2",
        scope: Scope::Profile,
        en: "How many extra lines a table row may grow by: a row holding a long value or a value with\n\
             line breaks wraps it at the column width and grows up to 1 + this many lines (each row only\n\
             as tall as it needs). What does not fit ends with …. 0 keeps every row on one line, with\n\
             line breaks shown as …⏎. Other values are warned about and the default is used.",
        ja: "表の行を伸ばす行の数: 長い値や改行のある値は列の幅で折り返し、行を最大で「1 + この数」の行の高さまで\
             伸ばす(行ごとに要るだけの高さ)。入りきらない分は最後に …。0 ならどの行も1行で、改行は …⏎ で示す。\
             ほかの値は警告して既定にする。",
    },
    Item {
        path: "display.note_path",
        ty: "string (\"common\", \"name\", \"full\" or \"off\")",
        ty_ja: "文字列(\"common\"・\"name\"・\"full\"・\"off\")",
        default: Some("\"common\""),
        example: "[display]\nnote_path = \"name\"",
        scope: Scope::Profile,
        en: "How the note column on the left shows each note: \"common\" drops the folder shared by every row,\n\
             \"name\" shows only the name, \"full\" the path from the root, and \"off\" no name (only the row marks).\n\
             Other values are warned about and the default is used.",
        ja: "左のノートの欄の出し方: \"common\" は全部の行に共通のフォルダを除き、\"name\" は名前だけ、\"full\" は根からのパス、\
             \"off\" は名前を出さない(行の印だけ)。ほかの値は警告して既定にする。",
    },
    Item {
        path: "display.tab_wrap",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("false"),
        example: "[display]\ntab_wrap = true",
        scope: Scope::Profile,
        en: "When the view tabs do not fit on the row of the view bar, wrap them onto extra rows above it so every\n\
             tab stays visible (the table moves down by those rows). false keeps one row and leaves out other tabs\n\
             when the screen is narrow.",
        ja: "ビューのタブがビューの帯の行に入らないとき、帯の上に行を足して折り返し、全部のタブを見せる(足した行のぶん\
             表は下がる)。false なら1行のままで、狭い画面では今のでないタブを省く。",
    },
    Item {
        path: "display.note_name",
        ty: "string (a regular expression)",
        ty_ja: "文字列(正規表現)",
        default: Some("\"\""),
        example: "[display]\nnote_name = '^\\d{8}_'",
        scope: Scope::Profile,
        en: "A regular expression applied to the name in the note column (after note_path; with folders, to the last\n\
             part only). When it matches, show the group named name, or else the first group, or else the name with\n\
             the match removed. A name that does not match, or would become empty, is shown as it is. \"\" sets\n\
             nothing. An expression that cannot be read is warned about and not used.",
        ja: "ノートの欄の名前に当てる正規表現(note_path のあと。フォルダがあれば最後の名前の部分だけ)。一致したら、\
             名前の付いた組 name、無ければ1つ目の組、組が無ければ一致した部分を除いた残りを出す。一致しない名前と、\
             空になる名前はそのまま。\"\" は何もしない。読めない正規表現は警告して使わない。",
    },
    Item {
        path: "dates.format",
        ty: "string",
        ty_ja: "文字列",
        default: Some("\"YYYY-MM-DD\""),
        example: "[dates]\nformat = \"YYYY/MM/DD (ddd)\"",
        scope: Scope::Profile,
        en: "How dates are shown and typed in the table. Parts: YYYY, YY, MM, M, DD, D, ddd\n\
             (weekday) and non-alphanumeric separators. Notes are always written as YYYY-MM-DD.",
        ja: "表の日付の見せ方と打ち込みの形。部品は YYYY・YY・MM・M・DD・D・ddd(曜日)と英数字でない区切り。\
             ノートにはいつも YYYY-MM-DD で書く。",
    },
    Item {
        path: "dates.week_start",
        ty: "string (\"sun\" or \"mon\")",
        ty_ja: "文字列(\"sun\" か \"mon\")",
        default: Some("\"sun\""),
        example: "[dates]\nweek_start = \"mon\"",
        scope: Scope::Profile,
        en: "First day of the week in the date calendar.",
        ja: "日付のカレンダーの週の始まり。",
    },
    Item {
        path: "edit.candidates",
        ty: "integer (0 or more)",
        ty_ja: "整数(0 以上)",
        default: Some("20"),
        example: "[edit]\ncandidates = 30",
        scope: Scope::Profile,
        en: "Maximum number of value candidates offered when editing a text cell.\n\
             Columns with more distinct values offer no candidates.",
        ja: "テキストのセルの編集で出す値の候補の上限。異なる値がこれより多い列は候補を出さない。",
    },
    Item {
        path: "edit.add_frontmatter",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("true"),
        example: "[edit]\nadd_frontmatter = false",
        scope: Scope::Profile,
        en: "Allow writing to notes with no front matter or an empty one (a front matter or a key\n\
             line is added). false makes those notes read-only.",
        ja: "フロントマターの無いノートと空のフロントマターのノートにも書く(フロントマターかキーの行を足す)。\
             false ならこの2つは読むだけ。",
    },
    Item {
        path: "new_note",
        ty: "table (mode, folder, name, ask, required, hidden, body and [new_note.set])",
        ty_ja: "表(mode・folder・name・ask・required・hidden・body と [new_note.set])",
        default: None,
        example: "[new_note]\nfolder = \"inbox\"\nname = \"{date} \"\nask = [\"priority\", \"due\"]\nrequired = [\"due\"]\nhidden = [\"created\"]\nbody = \"templates/note.md\"\n\n[new_note.set]\ntags = [\"inbox\"]\ndue = \"{date+7}\"\ncreated = \"{now}\"",
        scope: Scope::Profile,
        en: "How a new note is made from the table: mode (\"form\": a form with every field, or\n\
             \"editor\": create and open in the editor), folder (relative to the opened folder), name (name\n\
             template), ask (the form's columns; default: the visible columns), required (columns that must\n\
             be filled), hidden (columns written without a field), body (a body template file) and\n\
             [new_note.set] (column = value). Templates may use {date}, {date+7}, {date:YYYY/MM/DD},\n\
             {time}, {now}, {weekday}, {name} and {folder}. Values from the view's filter come first.\n\
             The table is one item: a narrower place that writes it replaces it whole.",
        ja: "表から新しいノートを作るときの決まり。mode(\"form\" は全部の欄を並べた窓、\"editor\" は作ってすぐエディタで開く)・\
             folder(開いたフォルダからの相対)・name(名前の雛形)・ask(窓に出す列。既定は見えている列)・\
             required(空では作らない列)・hidden(欄を出さずに値だけ入れる列)・body(本文の雛形のファイル)・\
             [new_note.set](列 = 値)。雛形には {date}・{date+7}・{date:YYYY/MM/DD}・{time}・{now}・{weekday}・\
             {name}・{folder} を書ける。ビューの絞り込みの値が先。この表は1つの項目で、狭い範囲に書けば丸ごと代わる。",
    },
    Item {
        path: "keys",
        ty: "table of tables ([keys.<mode>] with key = action)",
        ty_ja: "表の表([keys.<モード>] の下に キー = 動作)",
        default: None,
        example: "[keys.table]\n\"ctrl+f\" = \"search\"\n\"x\" = \"none\"",
        scope: Scope::Global,
        en: "Key rebinding per mode: under [keys.<mode>], write key = action name.\n\
             \"none\" removes the key. Modes: table, edit, review, quit, help, palette, search,\n\
             filter, detail, settings, settings_input, chips, list_select, menu, freq, sorts, relations, drawer, filter_window, filter_input, screen_menu, column_type. Action\n\
             names are shown in the command palette (:).",
        ja: "モードごとのキーの割り当て直し。[keys.<モード>] の下に キー = 動作の名前。\
             \"none\" でそのキーを外す。モードは table・edit・review・quit・help・palette・search・\
             filter・detail・settings・settings_input・chips・list_select・menu・freq・sorts・relations・drawer・filter_window・filter_input・screen_menu・column_type。動作の名前はコマンドのパレット(:)に出る。",
    },
    Item {
        path: "templates",
        ty: "table of tables ([templates.<name>] with profile items)",
        ty_ja: "表の表([templates.<名前>] の下にプロファイルの項目)",
        default: None,
        example: "[templates.night.look]\ntheme = \"dracula\"\npreset = \"dozy-pink\"",
        scope: Scope::Global,
        en: "Named pieces of a profile ([look], [display], [dates], [edit], [new_note]). use = \"<name>\" in\n\
             any place lays one under that place's values. ui.toml's template wins over config.toml's of\n\
             the same name.",
        ja: "名前を付けたプロファイルの断片([look]・[display]・[dates]・[edit]・[new_note])。どの範囲でも\
             use = \"<名前>\" で、その範囲の値の下に敷く。同じ名前は ui.toml のものが config.toml より先。",
    },
];

/// 実装が読む設定の項目の道筋の一覧(`ITEMS` の道筋。CLI-12)。
pub const KEYS: &[&str] = &{
    let mut names = [""; ITEMS.len()];
    let mut i = 0;
    while i < ITEMS.len() {
        names[i] = ITEMS[i].path;
        i += 1;
    }
    names
};

/// 道筋の項目。
pub fn item(path: &str) -> Option<&'static Item> {
    ITEMS.iter().find(|i| i.path == path)
}

/// 区画の名前(最上位の区画の表。`[look]` の下の `style`・`columns`・`colors` は look の中)。
pub const SECTIONS: &[&str] = &[
    "terminal",
    "workspace",
    "look",
    "display",
    "dates",
    "edit",
    "new_note",
    "keys",
    "templates",
];

/// 表のプロファイルの最上位の名前(どの範囲でも書ける)。
pub const PROFILE_KEYS: &[&str] = &[
    "use", "look", "display", "dates", "edit", "new_note", "types", "rules",
];

/// アプリ全体の最上位の名前(全体にだけ書ける)。
pub const GLOBAL_KEYS: &[&str] = &[
    "language",
    "editor",
    "poll_ms",
    "terminal",
    "workspace",
    "keys",
    "templates",
];

/// 旧い名前(0.3.0 まで)と移った先(CLI-20)。左は旧い道筋、右は新しい道筋。docs の旧い名前の表と突き合わせる。
pub const LEGACY: &[(&str, &str)] = &[
    ("color", "terminal.color"),
    ("ambiguous_wide", "terminal.ambiguous_wide"),
    ("nerd_font", "terminal.nerd_font"),
    ("workspace_detect", "workspace.detect"),
    ("theme", "look.theme"),
    ("theme_light", "look.theme"),
    ("theme_dark", "look.theme"),
    ("look", "look.mode"),
    ("borders", "look.style.frames"),
    ("style", "look.style"),
    ("style.preset", "look.preset"),
    ("cells", "look.cells"),
    ("cells.style", "look.cells"),
    ("cells.checkbox", "look.style.check"),
    ("cells.chips", "look.style.tags"),
    ("cells.select", "look.style.status"),
    ("cells.links", "look.style.links"),
    ("cells.icons", "look.style.icons"),
    ("cells.columns", "look.columns"),
    ("colors", "look.colors"),
    ("view_tabs", "display.tabs"),
    ("search_bar", "display.search_bar"),
    ("date_format", "dates.format"),
    ("week_start", "dates.week_start"),
    ("candidates", "edit.candidates"),
    ("add_frontmatter", "edit.add_frontmatter"),
];
