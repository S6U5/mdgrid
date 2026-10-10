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
    /// 書き方の例(TOML の行。区画の下の項目は区画の見出しを除いた行、表の項目は見出しから)。
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

    /// 表の項目(`[look.style]` のように見出しで書く項目)か。
    pub fn is_table(&self) -> bool {
        self.example.starts_with('[')
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
             a shell). Unset or blank: $VISUAL, then $EDITOR, then vi.",
        ja: "選んだノートを開くエディタ(引数つきでよい。例 \"code -w\"。シェルは通さない)。\
             無いか空(空白だけも)なら $VISUAL、$EDITOR、vi の順。",
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
        path: "terminal.color",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("true"),
        example: "color = false",
        scope: Scope::Global,
        en: "Use colors. false draws without colors (same as NO_COLOR or --no-color).",
        ja: "色を使う。false で色なし(NO_COLOR・--no-color と同じ)。",
    },
    Item {
        path: "terminal.ambiguous_wide",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("false"),
        example: "ambiguous_wide = true",
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
        example: "nerd_font = true",
        scope: Scope::Global,
        en: "Whether round pill ends (Nerd Font glyphs) can be drawn. true: your terminal font is a Nerd\n\
             Font. \"auto\": only in terminals that draw these glyphs themselves (Ghostty, WezTerm), so\n\
             nothing breaks elsewhere. Without round ends, \"pill\" is drawn as \"soft\".",
        ja: "丸い札の端(Nerd Font の字)を描いてよいか。true は端末の字形が Nerd Font。\"auto\" は、その字を\
             字体に頼らず自分で描く端末(Ghostty・WezTerm)のときだけ(ほかの端末で字が化けないように)。\
             描けなければ \"pill\" は \"soft\" で描く。",
    },
    Item {
        path: "workspace.detect",
        ty: "array of strings",
        ty_ja: "文字列の並び",
        default: Some("[\"vault\"]"),
        example: "detect = [\"vault\", \"git\"]",
        scope: Scope::Global,
        en: "When the table you open is in no written workspace, treat the root of the Obsidian vault\n\
             (\"vault\", a folder with .obsidian/) or the git repository (\"git\", a folder with .git) above it\n\
             as a workspace: its folders with notes become the tables of the relation map.\n\
             [] turns detection off.",
        ja: "書いたワークスペースに入らない表を開いたとき、上にある Obsidian の保管庫(\"vault\"。.obsidian/ のあるフォルダ)か\
             git のリポ(\"git\"。.git のあるフォルダ)の根をワークスペースとみなす。直下の、ノートのあるフォルダが\
             関係マップの表になる。[] で検知しない。",
    },
    Item {
        path: "look.theme",
        ty: "string (\"auto\", \"default\", \"nord\", \"solarized-light\", \"dracula\", \"gruvbox\", \"pink-monster\", \"dozy-pink\", \"sumi\", \"slate\", \"saas\", \"saas-dark\" or \"paper\") or table ({ light, dark })",
        ty_ja: "文字列(\"auto\"・\"default\"・\"nord\"・\"solarized-light\"・\"dracula\"・\"gruvbox\"・\"pink-monster\"・\"dozy-pink\"・\"sumi\"・\"slate\"・\"saas\"・\"saas-dark\"・\"paper\")か表({ light, dark })",
        default: Some("\"default\""),
        example: "theme = { light = \"paper\", dark = \"sumi\" }",
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
        example: "preset = \"saas\"",
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
        example: "mode = \"classic\"",
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
        example: "cells = \"plain\"",
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
        example: "row_numbers = true",
        scope: Scope::Profile,
        en: "Numbers 1, 2, 3... on the left of each row, in the shown order.",
        ja: "各行の左に、今の表示の並びの 1・2・3… の行番号。",
    },
    Item {
        path: "display.zebra",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("false"),
        example: "zebra = true",
        scope: Scope::Profile,
        en: "Shade every other row (not without colors).",
        ja: "一行おきに背景の色を付ける(色を使わない表示では付けない)。",
    },
    Item {
        path: "display.column_lines",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("false"),
        example: "column_lines = true",
        scope: Scope::Profile,
        en: "Draw │ between columns.",
        ja: "列の間に │ を引く。",
    },
    Item {
        path: "display.group_gap",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("false"),
        example: "group_gap = true",
        scope: Scope::Profile,
        en: "An empty line above every group heading but the first.",
        ja: "2つ目からのまとまりの見出しの上に空きの行を1つ入れる。",
    },
    Item {
        path: "display.tabs",
        ty: "string (\"always\", \"auto\" or \"never\")",
        ty_ja: "文字列(\"always\"・\"auto\"・\"never\")",
        default: Some("\"always\""),
        example: "tabs = \"auto\"",
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
        example: "search_bar = false",
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
        example: "chips = false",
        scope: Scope::Profile,
        en: "Show the band of active view settings. f still picks its items when hidden.",
        ja: "効いているビューの設定の帯を出す。隠しても f で帯の項目を選べる。",
    },
    Item {
        path: "dates.format",
        ty: "string",
        ty_ja: "文字列",
        default: Some("\"YYYY-MM-DD\""),
        example: "format = \"YYYY/MM/DD (ddd)\"",
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
        example: "week_start = \"mon\"",
        scope: Scope::Profile,
        en: "First day of the week in the date calendar.",
        ja: "日付のカレンダーの週の始まり。",
    },
    Item {
        path: "edit.candidates",
        ty: "integer (0 or more)",
        ty_ja: "整数(0 以上)",
        default: Some("20"),
        example: "candidates = 30",
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
        example: "add_frontmatter = false",
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
             filter, detail, settings, settings_input, chips, list_select, menu, freq, sorts, relations. Action\n\
             names are shown in the command palette (:).",
        ja: "モードごとのキーの割り当て直し。[keys.<モード>] の下に キー = 動作の名前。\
             \"none\" でそのキーを外す。モードは table・edit・review・quit・help・palette・search・\
             filter・detail・settings・settings_input・chips・list_select・menu・freq・sorts・relations。動作の名前はコマンドのパレット(:)に出る。",
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
pub const PROFILE_KEYS: &[&str] = &["use", "look", "display", "dates", "edit", "new_note"];

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
