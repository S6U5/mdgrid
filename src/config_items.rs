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
        ty: "string (\"default\", \"nord\", \"solarized-light\", \"dracula\", \"gruvbox\", \"pink-monster\" or \"dozy-pink\")",
        ty_ja: "文字列(\"default\"・\"nord\"・\"solarized-light\"・\"dracula\"・\"gruvbox\"・\"pink-monster\"・\"dozy-pink\")",
        default: Some("\"default\""),
        example: "theme = \"nord\"",
        en: "Color theme of the screen: \"default\", \"nord\", \"solarized-light\", \"dracula\",\n\
             \"gruvbox\", \"pink-monster\" or \"dozy-pink\". \"default\" keeps the terminal's own colors.\n\
             Ignored without colors (NO_COLOR, --no-color, TERM=dumb, color = false); 256-color\n\
             terminals get the nearest colors.",
        ja: "画面の色のテーマ。\"default\"・\"nord\"・\"solarized-light\"・\"dracula\"・\"gruvbox\"・\"pink-monster\"・\
             \"dozy-pink\" のどれか。\"default\" は端末の色のまま。色なし(NO_COLOR・--no-color・TERM=dumb・\
             color = false)では効かない。256 色の端末では近い色で塗る。",
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
             filter, detail, settings, settings_input, chips, list_select, menu, freq. Action names\n\
             are shown in the command palette (:).",
        ja: "モードごとのキーの割り当て直し。[keys.<モード>] の下に キー = 動作の名前。\
             \"none\" でそのキーを外す。モードは table・edit・review・quit・help・palette・search・\
             filter・detail・settings・settings_input・chips・list_select・menu・freq。動作の名前はコマンドのパレット(:)に出る。",
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
