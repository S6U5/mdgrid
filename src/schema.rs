//! 設定の項目の表(CLI-3・CLI-11・CLI-12・CLI-20)。形は docs/design.md の「設定の形と範囲ごとの上書き」。
//!
//! 項目の一覧はここ1か所に持ち、読み取りの知らない項目の判定(`config::parse`)・`--print-config`
//! (`config::default_toml`)・文書との突き合わせの試験・カタログの試験がこれを使う。旧い名前(0.3.0 まで)と
//! 移った先の対応も `LEGACY` に持ち、読み取りの写し(`legacy::lift`)と文書の表の試験が使う。

use crate::i18n::Msg;
/// 書ける範囲(CLI-11・SR-44)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// 全体(config.toml)だけ。アプリ全体の項目。
    Global,
    /// 表のプロファイル: 全体・ワークスペース・表・ビュー。
    Profile,
}

impl Scope {
    /// 範囲の名前の文言(screen.locales-all-in-ledger)。
    pub fn msg(self) -> Msg {
        match self {
            Scope::Global => Msg::CfgScopeGlobal,
            Scope::Profile => Msg::CfgScopeProfile,
        }
    }

    /// 文書の「書ける範囲」の行(英語。docs/config.md と突き合わせる)。
    pub fn en(self) -> &'static str {
        self.msg().en()
    }

    /// 文書の「書ける範囲」の行(日本語)。
    pub fn ja(self) -> &'static str {
        self.msg().ja()
    }

    /// `--print-config` の「書ける範囲」の行(今の言語)。
    pub fn text(self) -> &'static str {
        self.msg().text()
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
    /// 英語の説明(台帳の `msg` の英語。文書との突き合わせの試験が使う)。
    pub en: &'static str,
    /// 日本語の説明(台帳の `msg` の日本語)。
    pub ja: &'static str,
    /// 説明の文言(screen.locales-all-in-ledger。`--print-config` は今の言語で出す)。
    pub msg: Msg,
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

    /// 表の項目(`[look.style]` のように、その道筋の見出しで書く項目。表の並び `[[notices]]` も)か。
    pub fn is_table(&self) -> bool {
        self.example.starts_with(&format!("[{}", self.path))
            || self.example.starts_with(&format!("[[{}", self.path))
    }
}

/// 設定の項目の表。`--print-config` はこの順で出す(最上位 → 区画の順)。
pub const ITEMS: &[Item] = &[
    Item {
        path: "language",
        ty: "string (\"auto\" or a language code such as \"en\" or \"ja\")",
        ty_ja: "文字列(\"auto\" か、\"en\"・\"ja\" などの言語の符号)",
        default: Some("\"auto\""),
        example: "language = \"en\"",
        scope: Scope::Global,
        en: Msg::CfgLanguage.en(),
        ja: Msg::CfgLanguage.ja(),
        msg: Msg::CfgLanguage,
    },
    Item {
        path: "editor",
        ty: "string",
        ty_ja: "文字列",
        default: None,
        example: "editor = \"nvim\"",
        scope: Scope::Global,
        en: Msg::CfgEditor.en(),
        ja: Msg::CfgEditor.ja(),
        msg: Msg::CfgEditor,
    },
    Item {
        path: "poll_ms",
        ty: "integer (1 or more)",
        ty_ja: "整数(1 以上)",
        default: Some("1000"),
        example: "poll_ms = 2000",
        scope: Scope::Global,
        en: Msg::CfgPollMs.en(),
        ja: Msg::CfgPollMs.ja(),
        msg: Msg::CfgPollMs,
    },
    Item {
        path: "use",
        ty: "string (a template name)",
        ty_ja: "文字列(テンプレートの名前)",
        default: None,
        example: "use = \"night\"",
        scope: Scope::Profile,
        en: Msg::CfgUse.en(),
        ja: Msg::CfgUse.ja(),
        msg: Msg::CfgUse,
    },
    Item {
        path: "types",
        ty: "table (column = \"text\", \"number\", \"checkbox\", \"date\", \"datetime\", \"list\" or \"auto\")",
        ty_ja: "表(列 = \"text\"・\"number\"・\"checkbox\"・\"date\"・\"datetime\"・\"list\"・\"auto\")",
        default: Some("{}"),
        example: "[types]\ndue = \"date\"\nestimate = \"number\"",
        scope: Scope::Profile,
        en: Msg::CfgTypes.en(),
        ja: Msg::CfgTypes.ja(),
        msg: Msg::CfgTypes,
    },
    Item {
        path: "rules",
        ty: "table of tables (column = { required = boolean, values = [string, …], then = { value = { column = string } } })",
        ty_ja: "表の表(列 = { required = 真偽値, values = [文字列, …], then = { 値 = { 列 = 文字列 } } })",
        default: Some("{}"),
        example: "[rules.status]\nrequired = true\nvalues = [\"todo\", \"doing\", \"done\"]\n\n[rules.status.then]\ndone = { done_at = \"{date}\" }",
        scope: Scope::Profile,
        en: Msg::CfgRules.en(),
        ja: Msg::CfgRules.ja(),
        msg: Msg::CfgRules,
    },
    Item {
        path: "terminal.color",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("true"),
        example: "[terminal]\ncolor = false",
        scope: Scope::Global,
        en: Msg::CfgTerminalColor.en(),
        ja: Msg::CfgTerminalColor.ja(),
        msg: Msg::CfgTerminalColor,
    },
    Item {
        path: "terminal.ambiguous_wide",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("false"),
        example: "[terminal]\nambiguous_wide = true",
        scope: Scope::Global,
        en: Msg::CfgTerminalAmbiguousWide.en(),
        ja: Msg::CfgTerminalAmbiguousWide.ja(),
        msg: Msg::CfgTerminalAmbiguousWide,
    },
    Item {
        path: "terminal.nerd_font",
        ty: "boolean or \"auto\"",
        ty_ja: "真偽か \"auto\"",
        default: Some("\"auto\""),
        example: "[terminal]\nnerd_font = true",
        scope: Scope::Global,
        en: Msg::CfgTerminalNerdFont.en(),
        ja: Msg::CfgTerminalNerdFont.ja(),
        msg: Msg::CfgTerminalNerdFont,
    },
    Item {
        path: "terminal.nerd_icons",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("false"),
        example: "[terminal]\nnerd_icons = true",
        scope: Scope::Global,
        en: Msg::CfgTerminalNerdIcons.en(),
        ja: Msg::CfgTerminalNerdIcons.ja(),
        msg: Msg::CfgTerminalNerdIcons,
    },
    Item {
        path: "terminal.herdr",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("true"),
        example: "[terminal]\nherdr = false",
        scope: Scope::Global,
        en: Msg::CfgTerminalHerdr.en(),
        ja: Msg::CfgTerminalHerdr.ja(),
        msg: Msg::CfgTerminalHerdr,
    },
    Item {
        path: "workspace.detect",
        ty: "array of strings",
        ty_ja: "文字列の並び",
        default: Some("[\".obsidian\"]"),
        example: "[workspace]\ndetect = [\".obsidian\", \".git\"]",
        scope: Scope::Global,
        en: Msg::CfgWorkspaceDetect.en(),
        ja: Msg::CfgWorkspaceDetect.ja(),
        msg: Msg::CfgWorkspaceDetect,
    },
    Item {
        path: "git.auto",
        ty: "\"off\", \"add\", \"commit\" or \"push\"",
        ty_ja: "\"off\"・\"add\"・\"commit\"・\"push\"",
        default: Some("\"off\""),
        example: "[git]\nauto = \"commit\"",
        scope: Scope::Global,
        en: Msg::CfgGitAuto.en(),
        ja: Msg::CfgGitAuto.ja(),
        msg: Msg::CfgGitAuto,
    },
    Item {
        path: "git.when",
        ty: "\"save\" or \"quit\"",
        ty_ja: "\"save\"・\"quit\"",
        default: Some("\"save\""),
        example: "[git]\nwhen = \"quit\"",
        scope: Scope::Global,
        en: Msg::CfgGitWhen.en(),
        ja: Msg::CfgGitWhen.ja(),
        msg: Msg::CfgGitWhen,
    },
    Item {
        path: "git.message",
        ty: "\"auto\" or \"ask\"",
        ty_ja: "\"auto\"・\"ask\"",
        default: Some("\"auto\""),
        example: "[git]\nmessage = \"ask\"",
        scope: Scope::Global,
        en: Msg::CfgGitMessage.en(),
        ja: Msg::CfgGitMessage.ja(),
        msg: Msg::CfgGitMessage,
    },
    Item {
        path: "look.theme",
        ty: "string (\"auto\", \"default\", \"nord\", \"solarized-light\", \"dracula\", \"gruvbox\", \"pink-monster\", \"dozy-pink\", \"sumi\", \"slate\", \"saas\", \"saas-dark\" or \"paper\") or table ({ light, dark })",
        ty_ja: "文字列(\"auto\"・\"default\"・\"nord\"・\"solarized-light\"・\"dracula\"・\"gruvbox\"・\"pink-monster\"・\"dozy-pink\"・\"sumi\"・\"slate\"・\"saas\"・\"saas-dark\"・\"paper\")か表({ light, dark })",
        default: Some("\"default\""),
        example: "[look]\ntheme = { light = \"paper\", dark = \"sumi\" }",
        scope: Scope::Profile,
        en: Msg::CfgLookTheme.en(),
        ja: Msg::CfgLookTheme.ja(),
        msg: Msg::CfgLookTheme,
    },
    Item {
        path: "look.preset",
        ty: "string (\"sumi\", \"slate\", \"saas\", \"paper\", \"grid\", \"classic\" or \"dozy-pink\")",
        ty_ja: "文字列(\"sumi\"・\"slate\"・\"saas\"・\"paper\"・\"grid\"・\"classic\"・\"dozy-pink\")",
        default: Some("\"sumi\""),
        example: "[look]\npreset = \"saas\"",
        scope: Scope::Profile,
        en: Msg::CfgLookPreset.en(),
        ja: Msg::CfgLookPreset.ja(),
        msg: Msg::CfgLookPreset,
    },
    Item {
        path: "look.mode",
        ty: "string (\"modern\" or \"classic\")",
        ty_ja: "文字列(\"modern\" か \"classic\")",
        default: Some("\"modern\""),
        example: "[look]\nmode = \"classic\"",
        scope: Scope::Profile,
        en: Msg::CfgLookMode.en(),
        ja: Msg::CfgLookMode.ja(),
        msg: Msg::CfgLookMode,
    },
    Item {
        path: "look.cells",
        ty: "string (\"rich\" or \"plain\")",
        ty_ja: "文字列(\"rich\" か \"plain\")",
        default: Some("\"rich\""),
        example: "[look]\ncells = \"plain\"",
        scope: Scope::Profile,
        en: Msg::CfgLookCells.en(),
        ja: Msg::CfgLookCells.ja(),
        msg: Msg::CfgLookCells,
    },
    Item {
        path: "look.style",
        ty: "table (status, tags, check, select, rules, tabs, frames, band, links and icons)",
        ty_ja: "表(status・tags・check・select・rules・tabs・frames・band・links・icons)",
        default: None,
        example: "[look.style]\nstatus = \"chip\"\nselect = \"cross\"",
        scope: Scope::Profile,
        en: Msg::CfgLookStyle.en(),
        ja: Msg::CfgLookStyle.ja(),
        msg: Msg::CfgLookStyle,
    },
    Item {
        path: "look.columns",
        ty: "table (column = \"rich\", \"plain\" or \"chip\")",
        ty_ja: "表(列 = \"rich\"・\"plain\"・\"chip\")",
        default: None,
        example: "[look.columns]\nstatus = \"plain\"\nowner = \"chip\"",
        scope: Scope::Profile,
        en: Msg::CfgLookColumns.en(),
        ja: Msg::CfgLookColumns.ja(),
        msg: Msg::CfgLookColumns,
    },
    Item {
        path: "look.colors",
        ty: "table (color roles, and [look.colors.values] for value colors)",
        ty_ja: "表(色の役割と、[look.colors.values] に値の色)",
        default: None,
        example: "[look.colors]\naccent = \"#e0a458\"\n\n[look.colors.values]\ndone = \"green\"",
        scope: Scope::Profile,
        en: Msg::CfgLookColors.en(),
        ja: Msg::CfgLookColors.ja(),
        msg: Msg::CfgLookColors,
    },
    Item {
        path: "display.row_numbers",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("false"),
        example: "[display]\nrow_numbers = true",
        scope: Scope::Profile,
        en: Msg::CfgDisplayRowNumbers.en(),
        ja: Msg::CfgDisplayRowNumbers.ja(),
        msg: Msg::CfgDisplayRowNumbers,
    },
    Item {
        path: "display.zebra",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("false"),
        example: "[display]\nzebra = true",
        scope: Scope::Profile,
        en: Msg::CfgDisplayZebra.en(),
        ja: Msg::CfgDisplayZebra.ja(),
        msg: Msg::CfgDisplayZebra,
    },
    Item {
        path: "display.column_lines",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("false"),
        example: "[display]\ncolumn_lines = true",
        scope: Scope::Profile,
        en: Msg::CfgDisplayColumnLines.en(),
        ja: Msg::CfgDisplayColumnLines.ja(),
        msg: Msg::CfgDisplayColumnLines,
    },
    Item {
        path: "display.group_gap",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("false"),
        example: "[display]\ngroup_gap = true",
        scope: Scope::Profile,
        en: Msg::CfgDisplayGroupGap.en(),
        ja: Msg::CfgDisplayGroupGap.ja(),
        msg: Msg::CfgDisplayGroupGap,
    },
    Item {
        path: "display.tabs",
        ty: "string (\"always\", \"auto\" or \"never\")",
        ty_ja: "文字列(\"always\"・\"auto\"・\"never\")",
        default: Some("\"always\""),
        example: "[display]\ntabs = \"auto\"",
        scope: Scope::Profile,
        en: Msg::CfgDisplayTabs.en(),
        ja: Msg::CfgDisplayTabs.ja(),
        msg: Msg::CfgDisplayTabs,
    },
    Item {
        path: "display.search_bar",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("true"),
        example: "[display]\nsearch_bar = false",
        scope: Scope::Profile,
        en: Msg::CfgDisplaySearchBar.en(),
        ja: Msg::CfgDisplaySearchBar.ja(),
        msg: Msg::CfgDisplaySearchBar,
    },
    Item {
        path: "display.chips",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("true"),
        example: "[display]\nchips = false",
        scope: Scope::Profile,
        en: Msg::CfgDisplayChips.en(),
        ja: Msg::CfgDisplayChips.ja(),
        msg: Msg::CfgDisplayChips,
    },
    Item {
        path: "display.buttons",
        ty: "string (\"plain\" or \"box\")",
        ty_ja: "文字列(\"plain\" か \"box\")",
        default: Some("\"plain\""),
        example: "[display]\nbuttons = \"box\"",
        scope: Scope::Profile,
        en: Msg::CfgDisplayButtons.en(),
        ja: Msg::CfgDisplayButtons.ja(),
        msg: Msg::CfgDisplayButtons,
    },
    Item {
        path: "display.bar_padding",
        ty: "number (0 to 2 in steps of 0.5)",
        ty_ja: "数(0〜2 の 0.5 刻み)",
        default: Some("1"),
        example: "[display]\nbar_padding = 0.5",
        scope: Scope::Profile,
        en: Msg::CfgDisplayBarPadding.en(),
        ja: Msg::CfgDisplayBarPadding.ja(),
        msg: Msg::CfgDisplayBarPadding,
    },
    Item {
        path: "display.top_padding",
        ty: "number (0 to 2 in steps of 0.5)",
        ty_ja: "数(0〜2 の 0.5 刻み)",
        default: Some("1"),
        example: "[display]\ntop_padding = 0",
        scope: Scope::Profile,
        en: Msg::CfgDisplayTopPadding.en(),
        ja: Msg::CfgDisplayTopPadding.ja(),
        msg: Msg::CfgDisplayTopPadding,
    },
    Item {
        path: "display.row_extra_lines",
        ty: "integer (0 to 9)",
        ty_ja: "整数(0〜9)",
        default: Some("0"),
        example: "[display]\nrow_extra_lines = 2",
        scope: Scope::Profile,
        en: Msg::CfgDisplayRowExtraLines.en(),
        ja: Msg::CfgDisplayRowExtraLines.ja(),
        msg: Msg::CfgDisplayRowExtraLines,
    },
    Item {
        path: "display.note_path",
        ty: "string (\"common\", \"name\", \"full\" or \"off\")",
        ty_ja: "文字列(\"common\"・\"name\"・\"full\"・\"off\")",
        default: Some("\"common\""),
        example: "[display]\nnote_path = \"name\"",
        scope: Scope::Profile,
        en: Msg::CfgDisplayNotePath.en(),
        ja: Msg::CfgDisplayNotePath.ja(),
        msg: Msg::CfgDisplayNotePath,
    },
    Item {
        path: "display.tab_wrap",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("false"),
        example: "[display]\ntab_wrap = true",
        scope: Scope::Profile,
        en: Msg::CfgDisplayTabWrap.en(),
        ja: Msg::CfgDisplayTabWrap.ja(),
        msg: Msg::CfgDisplayTabWrap,
    },
    Item {
        path: "display.tab_wrap_gap",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("false"),
        example: "[display]\ntab_wrap = true\ntab_wrap_gap = true",
        scope: Scope::Profile,
        en: Msg::CfgDisplayTabWrapGap.en(),
        ja: Msg::CfgDisplayTabWrapGap.ja(),
        msg: Msg::CfgDisplayTabWrapGap,
    },
    Item {
        path: "display.note_name",
        ty: "string (a regular expression)",
        ty_ja: "文字列(正規表現)",
        default: Some("\"\""),
        example: "[display]\nnote_name = '^\\d{8}_'",
        scope: Scope::Profile,
        en: Msg::CfgDisplayNoteName.en(),
        ja: Msg::CfgDisplayNoteName.ja(),
        msg: Msg::CfgDisplayNoteName,
    },
    Item {
        path: "dates.format",
        ty: "string",
        ty_ja: "文字列",
        default: Some("\"YYYY-MM-DD\""),
        example: "[dates]\nformat = \"YYYY/MM/DD (ddd)\"",
        scope: Scope::Profile,
        en: Msg::CfgDatesFormat.en(),
        ja: Msg::CfgDatesFormat.ja(),
        msg: Msg::CfgDatesFormat,
    },
    Item {
        path: "dates.week_start",
        ty: "string (\"sun\" or \"mon\")",
        ty_ja: "文字列(\"sun\" か \"mon\")",
        default: Some("\"sun\""),
        example: "[dates]\nweek_start = \"mon\"",
        scope: Scope::Profile,
        en: Msg::CfgDatesWeekStart.en(),
        ja: Msg::CfgDatesWeekStart.ja(),
        msg: Msg::CfgDatesWeekStart,
    },
    Item {
        path: "edit.candidates",
        ty: "integer (0 or more)",
        ty_ja: "整数(0 以上)",
        default: Some("20"),
        example: "[edit]\ncandidates = 30",
        scope: Scope::Profile,
        en: Msg::CfgEditCandidates.en(),
        ja: Msg::CfgEditCandidates.ja(),
        msg: Msg::CfgEditCandidates,
    },
    Item {
        path: "edit.add_frontmatter",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("true"),
        example: "[edit]\nadd_frontmatter = false",
        scope: Scope::Profile,
        en: Msg::CfgEditAddFrontmatter.en(),
        ja: Msg::CfgEditAddFrontmatter.ja(),
        msg: Msg::CfgEditAddFrontmatter,
    },
    Item {
        path: "new_note",
        ty: "table (mode, folder, name, ask, required, hidden, body and [new_note.set])",
        ty_ja: "表(mode・folder・name・ask・required・hidden・body と [new_note.set])",
        default: None,
        example: "[new_note]\nfolder = \"inbox\"\nname = \"{date} \"\nask = [\"priority\", \"due\"]\nrequired = [\"due\"]\nhidden = [\"created\"]\nbody = \"templates/note.md\"\n\n[new_note.set]\ntags = [\"inbox\"]\ndue = \"{date+7}\"\ncreated = \"{now}\"",
        scope: Scope::Profile,
        en: Msg::CfgNewNote.en(),
        ja: Msg::CfgNewNote.ja(),
        msg: Msg::CfgNewNote,
    },
    Item {
        path: "keys",
        ty: "table of tables ([keys.<mode>] with key = action)",
        ty_ja: "表の表([keys.<モード>] の下に キー = 動作)",
        default: None,
        example: "[keys.table]\n\"ctrl+f\" = \"search\"\n\"x\" = \"none\"",
        scope: Scope::Global,
        en: Msg::CfgKeys.en(),
        ja: Msg::CfgKeys.ja(),
        msg: Msg::CfgKeys,
    },
    Item {
        path: "keys.vim",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("false"),
        example: "[keys]\nvim = true",
        scope: Scope::Global,
        en: Msg::CfgKeysVim.en(),
        ja: Msg::CfgKeysVim.ja(),
        msg: Msg::CfgKeysVim,
    },
    Item {
        path: "search.across",
        ty: "boolean",
        ty_ja: "真偽値",
        default: Some("false"),
        example: "[search]\nacross = true",
        scope: Scope::Global,
        en: Msg::CfgSearchAcross.en(),
        ja: Msg::CfgSearchAcross.ja(),
        msg: Msg::CfgSearchAcross,
    },
    Item {
        path: "templates",
        ty: "table of tables ([templates.<name>] with profile items)",
        ty_ja: "表の表([templates.<名前>] の下にプロファイルの項目)",
        default: None,
        example: "[templates.night.look]\ntheme = \"dracula\"\npreset = \"dozy-pink\"",
        scope: Scope::Global,
        en: Msg::CfgTemplates.en(),
        ja: Msg::CfgTemplates.ja(),
        msg: Msg::CfgTemplates,
    },
    Item {
        path: "conditions",
        ty: "table of tables ([conditions.<name>] with expr, label, key, hide)",
        ty_ja: "表の表([conditions.<名前>] の下に expr・label・key・hide)",
        default: None,
        example: "[conditions.soon]\nexpr = 'due <= today() + \"7d\"'\nkey = 1\n\n[conditions.far]\nexpr = 'start > today() + \"30d\" && !(due <= today() + \"7d\")'\nhide = true\nkey = 9",
        scope: Scope::Global,
        en: Msg::CfgConditions.en(),
        ja: Msg::CfgConditions.ja(),
        msg: Msg::CfgConditions,
    },
    Item {
        path: "notices",
        ty: "array of tables ([[notices]] with cond, text, scope, show_zero, or note, lines)",
        ty_ja: "表の並び([[notices]] の下に cond・text・scope・show_zero か note・lines)",
        default: None,
        example: "[[notices]]\ncond = \"overdue\"\ntext = \"{count} overdue\"\nscope = \"workspace\"\n\n[[notices]]\nnote = \"~/notes/inbox.md\"\nlines = 2\n\n[conditions.overdue]\nexpr = 'due < today() && status != \"done\"'",
        scope: Scope::Global,
        en: Msg::CfgNotices.en(),
        ja: Msg::CfgNotices.ja(),
        msg: Msg::CfgNotices,
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
    "git",
    "look",
    "display",
    "dates",
    "edit",
    "new_note",
    "keys",
    "search",
    "templates",
    "conditions",
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
    "git",
    "keys",
    "templates",
    "conditions",
    "notices",
    "search",
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
