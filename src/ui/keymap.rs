//! キーの表(SR-4)。`(モード, キー) → 動作` と、表示名・ヘルプの節・下の帯での順位を1つの表に持つ。
//! 振り分け(`lookup`)と下の帯(`hints`)は、どちらもこの表から作る。
//! 全角の英数字と `、`・`・` は半角のキーに直してから引く(SR-17)。
//! 画面は `App` が持つ表の写し(既定は `BINDINGS`)を引く。設定での割り当て直し(SR-13)は `rebind` で写しを上書きする。

use mdgrid::i18n::{self, Lang, Msg};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// キーの効くモード(SR-16)。表・編集(1行の入力ボックス)・保存の確認(差分)・終了の確認。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    Table,
    Edit,
    Confirm,
    Quit,
    /// ヘルプ(SR-5)。
    Help,
    /// コマンドのパレット(SR-14)。
    Palette,
    /// 検索の語の入力(NV-1)。
    Search,
    /// 簡易の絞り込みの語の入力(NV-2)。
    Filter,
    /// 詳細の表示(NV-6・SR-16)。
    Detail,
    /// ビューの設定の画面(NV-13・NV-18)。
    Settings,
    /// ビューの設定の値の入力(含む・含まない・比較の値。NV-19)。
    SettingsText,
    /// 表の上の帯の項目を選ぶ(NV-16・NV-22)。
    Chips,
    /// リストの選択(CE-16・CE-17・CE-19)。
    ListPick,
    /// その場の操作の一覧(SR-24。menu.rs)。
    Menu,
    /// 列の値の頻度表(NV-9。freq.rs)。
    Freq,
    /// 並べ替えの窓(NV-24。sorts.rs)。
    Sorts,
    /// 関係マップ(REL-7。relmap.rs)。
    Relations,
}

impl Mode {
    /// 表示名の文言(SR-23)。
    pub fn msg(self) -> Msg {
        match self {
            Mode::Table => Msg::ModeTable,
            Mode::Edit => Msg::ModeEdit,
            Mode::Confirm => Msg::ModeConfirm,
            Mode::Quit => Msg::ModeQuit,
            Mode::Help => Msg::ModeHelp,
            Mode::Palette => Msg::ModePalette,
            Mode::Search => Msg::ModeSearch,
            Mode::Filter => Msg::ModeFilter,
            Mode::Detail => Msg::ModeDetail,
            Mode::Settings => Msg::ModeSettings,
            Mode::SettingsText => Msg::ModeSettingsText,
            Mode::Chips => Msg::ModeChips,
            Mode::ListPick => Msg::ModeListPick,
            Mode::Menu => Msg::ModeMenu,
            Mode::Freq => Msg::ModeFreq,
            Mode::Sorts => Msg::ModeSorts,
            Mode::Relations => Msg::ModeRelations,
        }
    }

    /// 今の言語の表示名(下の帯・ヘルプ・警告)。
    pub fn label(self) -> &'static str {
        self.msg().text()
    }

    /// 設定の `[keys.<モード>]` に書く名前(SR-13・SR-16。英語の小文字)。
    pub fn name(self) -> &'static str {
        match self {
            Mode::Table => "table",
            Mode::Edit => "edit",
            Mode::Confirm => "review",
            Mode::Quit => "quit",
            Mode::Help => "help",
            Mode::Palette => "palette",
            Mode::Search => "search",
            Mode::Filter => "filter",
            Mode::Detail => "detail",
            Mode::Settings => "settings",
            Mode::SettingsText => "settings_input",
            Mode::Chips => "chips",
            Mode::ListPick => "list_select",
            Mode::Menu => "menu",
            Mode::Freq => "freq",
            Mode::Sorts => "sorts",
            Mode::Relations => "relations",
        }
    }

    pub const ALL: [Mode; 17] = [
        Mode::Table,
        Mode::Edit,
        Mode::Palette,
        Mode::Confirm,
        Mode::Detail,
        Mode::Quit,
        Mode::Help,
        Mode::Search,
        Mode::Filter,
        Mode::Settings,
        Mode::SettingsText,
        Mode::Chips,
        Mode::ListPick,
        Mode::Menu,
        Mode::Freq,
        Mode::Sorts,
        Mode::Relations,
    ];

    /// 設定のモードの名前を引く。英語の名前(`table`)と、どちらの言語の表示名(`表`・`Table`)も受ける
    /// (設定の書き方が画面の言語で変わらないように)。
    pub fn by_name(name: &str) -> Option<Mode> {
        Mode::ALL
            .into_iter()
            .find(|m| m.name() == name || m.msg().ja() == name || m.msg().en() == name)
    }

    /// 文字を入力に使うモード(前置きのキーを持たない)。
    pub(crate) fn takes_text(self) -> bool {
        matches!(
            self,
            Mode::Edit
                | Mode::Palette
                | Mode::Search
                | Mode::Filter
                | Mode::SettingsText
                | Mode::ListPick
        )
    }
}

/// 振り分け先(意図)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
    FirstColumn,
    LastColumn,
    PageUp,
    PageDown,
    Top,
    Bottom,
    NextCell,
    PrevCell,
    Edit,
    Clear,
    Undo,
    Redo,
    Save,
    Quit,
    CancelLoad,
    // 編集(CE-1・CE-11)
    Commit,
    Cancel,
    Revert,
    ClearInput,
    CommitNext,
    CommitPrev,
    CursorLeft,
    CursorRight,
    CursorHome,
    CursorEnd,
    DeleteBack,
    DeleteForward,
    /// 候補のリストで上・下の候補へ(CE-3)。
    ListUp,
    ListDown,
    /// 日付の入力で今日にする(CE-21)。
    Today,
    /// カレンダーで前の月・次の月・前の年・次の年(CE-23・CE-24)。
    /// カレンダーが出ていなければ ←→(文字のカーソル)・↑↓(候補)と同じ。
    PrevMonth,
    NextMonth,
    PrevYear,
    NextYear,
    /// 日時のカレンダーで、日と時刻の欄のどちらを選ぶかを切り替える(CE-30・CE-31)。
    TimeFocus,
    // 保存の確認(WB-9・WB-16)
    SaveAll,
    Back,
    NextFile,
    PrevFile,
    Overwrite,
    DiscardRow,
    // 終了の確認(WB-11)
    QuitSave,
    QuitDiscard,
    // ヘルプとパレット(SR-5・SR-14)
    Help,
    Palette,
    /// ヘルプ・パレットを閉じる。
    Close,
    /// パレットの選んだ候補を実行する。
    Run,
    // 外へ(SR-8・OUT-1)
    OpenEditor,
    Copy,
    CopyRow,
    // 表の形(BV-13・SR-3・NV-4)
    PrevView,
    NextView,
    Narrower,
    Wider,
    // 移動・検索・絞り込み・選択(NV-1・NV-2・NV-5・NV-7・NV-8・SR-18)
    HalfPageDown,
    HalfPageUp,
    Search,
    SearchNext,
    SearchPrev,
    QuickFilter,
    Mark,
    Visual,
    ExtendUp,
    ExtendDown,
    SelectAll,
    /// Esc: 選択を解く。選択が無ければ簡易の絞り込みを解く(SR-18)。
    Escape,
    HighlightSame,
    FilterSame,
    // 並べ替えと列(NV-3・NV-4)
    Sort,
    HideColumn,
    ShowColumn,
    /// CE-28: どのノートにも無いキーの列を足す。
    AddColumn,
    /// CE-29: 列のキーの名前を、全部のノートで変える。
    RenameKey,
    /// CE-34: 選んだ行のノートの名前を変える。
    RenameNote,
    /// CE-29: 列のキーを、全部のノートから消す。
    DeleteKey,
    MoveColumnLeft,
    MoveColumnRight,
    Freeze,
    // 詳細の表示(NV-6)
    Detail,
    // ビューの設定と帯(NV-13〜NV-22)
    ViewSettings,
    FocusChips,
    NextSection,
    PrevSection,
    Toggle,
    MoveItemUp,
    MoveItemDown,
    RemoveItem,
    // mdgrid のビュー(BV-19)。キーは割り当てず、パレットのコマンド(`COMMANDS`)から。
    ExportBase,
    ImportBase,
    /// OUT-2: 画面の表をファイルに書き出す。
    ExportTable,
    /// CLI-19: 登録した表の一覧を出して開く。
    OpenPlace,
    /// CLI-18: 今の表を登録する。
    RegisterPlace,
    /// REL-4: 今のセルのリンクの行き先を開く。
    OpenLink,
    /// REL-5: 今の行を指しているノート(つながった行)。
    LinkedRows,
    /// REL-7・REL-8: 関係マップと表の画面を切り替える。
    RelationMap,
    /// NV-27: 親子で並べた表で、選んだ行の子孫を畳む・開く。
    ToggleTree,
    /// WS-2: この表でワークスペースを作る・足す・外す・ワークスペースを開く。
    WsNew,
    WsAdd,
    WsRemove,
    WsOpen,
    // 新しいノート(CE-25・CE-27)。ヘッダーの「+ 新規」とパレットからも。
    NewNote,
    /// CE-26: 新しいノートの窓で、どの欄からでも作る(既定 Ctrl+S)。
    CreateNote,
    /// CE-33: 新しいノートの窓で、作ってすぐエディタで開く(既定 Ctrl+E)。
    CreateNoteEdit,
    /// その場の操作の一覧を開く(SR-24)。セルの右クリックからも。
    Menu,
    /// 選んでいる列の値の頻度表を開く(NV-9)。
    Freq,
    /// 並べ替えの窓を開く(NV-24)。
    SortMenu,
    /// 今のビューを既定のビューにする(NV-25)。
    SetDefaultView,
}

impl Action {
    /// パレットで探す動作の名前(SR-14。英語の小文字)。
    pub fn name(self) -> &'static str {
        match self {
            Action::Up => "up",
            Action::Down => "down",
            Action::Left => "left",
            Action::Right => "right",
            Action::FirstColumn => "first_column",
            Action::LastColumn => "last_column",
            Action::PageUp => "page_up",
            Action::PageDown => "page_down",
            Action::Top => "top",
            Action::Bottom => "bottom",
            Action::NextCell => "next_cell",
            Action::PrevCell => "prev_cell",
            Action::Edit => "edit",
            Action::Clear => "clear",
            Action::Undo => "undo",
            Action::Redo => "redo",
            Action::Save => "save",
            Action::Quit => "quit",
            Action::CancelLoad => "cancel_load",
            Action::Commit => "commit",
            Action::Cancel => "cancel",
            Action::Revert => "revert",
            Action::ClearInput => "clear_input",
            Action::CommitNext => "commit_next",
            Action::CommitPrev => "commit_prev",
            Action::CursorLeft => "cursor_left",
            Action::CursorRight => "cursor_right",
            Action::CursorHome => "cursor_home",
            Action::CursorEnd => "cursor_end",
            Action::DeleteBack => "delete_back",
            Action::DeleteForward => "delete_forward",
            Action::ListUp => "list_up",
            Action::ListDown => "list_down",
            Action::Today => "today",
            Action::PrevMonth => "prev_month",
            Action::NextMonth => "next_month",
            Action::PrevYear => "prev_year",
            Action::NextYear => "next_year",
            Action::TimeFocus => "time_focus",
            Action::SaveAll => "save_all",
            Action::Back => "back",
            Action::NextFile => "next_file",
            Action::PrevFile => "prev_file",
            Action::Overwrite => "overwrite",
            Action::DiscardRow => "discard_row",
            Action::QuitSave => "quit_save",
            Action::QuitDiscard => "quit_discard",
            Action::Help => "help",
            Action::Palette => "palette",
            Action::Close => "close",
            Action::Run => "run",
            Action::OpenEditor => "open_editor",
            Action::Copy => "copy",
            Action::CopyRow => "copy_row",
            Action::PrevView => "prev_view",
            Action::NextView => "next_view",
            Action::Narrower => "narrow_column",
            Action::Wider => "widen_column",
            Action::HalfPageDown => "half_page_down",
            Action::HalfPageUp => "half_page_up",
            Action::Search => "search",
            Action::SearchNext => "search_next",
            Action::SearchPrev => "search_prev",
            Action::QuickFilter => "quick_filter",
            Action::Mark => "mark_row",
            Action::Visual => "select_range",
            Action::ExtendUp => "extend_up",
            Action::ExtendDown => "extend_down",
            Action::SelectAll => "select_all",
            Action::Escape => "clear_selection",
            Action::HighlightSame => "highlight_same",
            Action::FilterSame => "filter_same",
            Action::Sort => "sort_column",
            Action::HideColumn => "hide_column",
            Action::ShowColumn => "show_column",
            Action::AddColumn => "add_column",
            Action::RenameKey => "rename_key",
            Action::RenameNote => "rename_note",
            Action::DeleteKey => "delete_key",
            Action::MoveColumnLeft => "move_column_left",
            Action::MoveColumnRight => "move_column_right",
            Action::Freeze => "freeze_columns",
            Action::Detail => "detail",
            Action::ViewSettings => "view_settings",
            Action::FocusChips => "focus_chips",
            Action::NextSection => "next_section",
            Action::PrevSection => "prev_section",
            Action::Toggle => "toggle",
            Action::MoveItemUp => "move_item_up",
            Action::MoveItemDown => "move_item_down",
            Action::RemoveItem => "remove_item",
            Action::ExportBase => "export_base",
            Action::ExportTable => "export_table",
            Action::SortMenu => "sort_menu",
            Action::SetDefaultView => "set_default_view",
            Action::OpenPlace => "open_place",
            Action::RegisterPlace => "register_place",
            Action::OpenLink => "open_link",
            Action::LinkedRows => "linked_rows",
            Action::RelationMap => "relation_map",
            Action::ToggleTree => "toggle_tree",
            Action::WsNew => "workspace_new",
            Action::WsAdd => "workspace_add",
            Action::WsRemove => "workspace_remove",
            Action::WsOpen => "workspace_open",
            Action::ImportBase => "import_base",
            Action::NewNote => "new_note",
            Action::CreateNote => "create_note",
            Action::CreateNoteEdit => "create_note_edit",
            Action::Menu => "action_menu",
            Action::Freq => "frequency",
        }
    }

    /// 動作を名前(`Action::name`)で引く(SR-13 の設定)。表にある動作だけ。
    pub fn by_name(name: &str) -> Option<Action> {
        BINDINGS.iter().map(|b| b.action).find(|a| a.name() == name)
    }
}

/// キーの表の1行。`key` はキーの名前(`key_name` の形)。前置きのあるキーは空白で区切る(`g g`)。
#[derive(Debug, Clone)]
pub struct Binding {
    pub mode: Mode,
    pub key: &'static str,
    pub action: Action,
    /// 表示名の日本語(下の帯・ヘルプ)。既定の表では `msg` の日本語と同じ。
    pub label: &'static str,
    /// ヘルプの節(SR-5)の日本語(節をまとめる鍵にも使う)。既定の表では `section_msg` の日本語と同じ。
    pub section: &'static str,
    /// 下の帯での順位。0 は出さない。小さいほど先。
    pub rank: u8,
    /// 表示名の文言(SR-23)。日本語では `label` を、ほかの言語ではこの文言を出す。
    pub msg: Msg,
    /// 節の文言(SR-23)。日本語では `section` を、ほかの言語ではこの文言を出す。
    pub section_msg: Msg,
}

impl Binding {
    /// 今の言語の表示名(下の帯・ヘルプ・パレット)。
    pub fn text(&self) -> &'static str {
        shown(self.label, self.msg)
    }

    /// 今の言語の節の名前(ヘルプ)。
    pub fn section_text(&self) -> &'static str {
        shown(self.section, self.section_msg)
    }
}

/// 日本語なら表の日本語(`ja`。表の写しで書き換えた名前も含む)、ほかの言語なら文言の表から。
fn shown(ja: &'static str, msg: Msg) -> &'static str {
    match i18n::current() {
        Lang::Ja => ja,
        lang => msg.get(lang),
    }
}

const fn b(key: &'static str, action: Action, msg: Msg, section: Msg, rank: u8) -> Binding {
    m(Mode::Table, key, action, msg, section, rank)
}

const fn m(
    mode: Mode,
    key: &'static str,
    action: Action,
    msg: Msg,
    section: Msg,
    rank: u8,
) -> Binding {
    Binding {
        mode,
        key,
        action,
        label: msg.ja(),
        section: section.ja(),
        rank,
        msg,
        section_msg: section,
    }
}

const MOVE: Msg = Msg::SecMove;
const EDIT: Msg = Msg::SecEdit;
const FILE: Msg = Msg::SecFile;
const INPUT: Msg = Msg::SecInput;
const REVIEW: Msg = Msg::SecReview;
const QUIT: Msg = Msg::SecQuit;
const FIND: Msg = Msg::SecFind;
const OUTSIDE: Msg = Msg::SecOutside;
const IN_HELP: Msg = Msg::SecInHelp;
const IN_PALETTE: Msg = Msg::SecInPalette;
const SHAPE: Msg = Msg::SecShape;
const SEARCH: Msg = Msg::SecSearch;
const SELECT: Msg = Msg::SecSelect;
const IN_PROMPT: Msg = Msg::SecInPrompt;
const IN_DETAIL: Msg = Msg::SecInDetail;
const IN_SETTINGS: Msg = Msg::SecInSettings;
const IN_CHIPS: Msg = Msg::SecInChips;
const IN_PICK: Msg = Msg::SecInPick;
const NATIVE: Msg = Msg::SecNative;
const NOTE: Msg = Msg::SecNote;
const IN_MENU: Msg = Msg::SecInMenu;
const IN_FREQ: Msg = Msg::SecInFreq;
const IN_SORTS: Msg = Msg::SecInSorts;
const IN_RELMAP: Msg = Msg::SecInRelMap;

/// パレットのコマンド(BV-19・CE-25)。キーの無い動作と、キーを外してもパレットから使える動作。
/// パレットの候補とヘルプはこの表からも作る。読むだけ(WB-15)では書くコマンドなので出さない
/// (キーの表にある同じ動作も、ヘルプとパレットに出さない)。
#[derive(Debug, Clone, Copy)]
pub struct Command {
    pub action: Action,
    /// 表示名の日本語(`msg` の日本語)。
    pub label: &'static str,
    /// 節の日本語(`section_msg` の日本語)。
    pub section: &'static str,
    /// 表示名の文言(SR-23)。
    pub msg: Msg,
    /// 節の文言(SR-23)。
    pub section_msg: Msg,
}

impl Command {
    /// 今の言語の表示名(ヘルプ・パレット)。
    pub fn text(&self) -> &'static str {
        shown(self.label, self.msg)
    }

    /// 今の言語の節の名前(ヘルプ)。
    pub fn section_text(&self) -> &'static str {
        shown(self.section, self.section_msg)
    }
}

const fn cmd(action: Action, msg: Msg, section: Msg) -> Command {
    Command {
        action,
        label: msg.ja(),
        section: section.ja(),
        msg,
        section_msg: section,
    }
}

pub const COMMANDS: &[Command] = &[
    cmd(Action::ExportBase, Msg::CmdExportBase, NATIVE),
    cmd(Action::ImportBase, Msg::CmdImportBase, NATIVE),
    // NV-25: 今のビューを既定のビューにする(views.toml に書くので読むだけでは出さない)。
    cmd(Action::SetDefaultView, Msg::CmdSetDefaultView, NATIVE),
    // OUT-2: 画面の表の書き出し(既定のキーなし。パレットから)。
    cmd(Action::ExportTable, Msg::CmdExportTable, OUTSIDE),
    // CE-29: キーの名前の変更と削除(既定のキーなし。パレットと操作の一覧から)。
    cmd(Action::RenameKey, Msg::CmdRenameKey, SHAPE),
    cmd(Action::DeleteKey, Msg::CmdDeleteKey, SHAPE),
    // CE-34: ノートの名前の変更(既定のキーなし。パレットと操作の一覧から)。
    cmd(Action::RenameNote, Msg::CmdRenameNote, NOTE),
    // CE-25: 新しいノートはキー(既定 `a`)もあるが、割り当て直して外してもパレットから始められるように。
    cmd(Action::NewNote, Msg::KeyNewNote, NOTE),
];

/// ノートを書かないパレットのコマンド(既定のキーなし)。`COMMANDS` と違い読むだけ(WB-15)でも出す:
/// 登録した表(CLI-18・CLI-19)と、リレーションをたどる操作(REL-4・REL-5)。
pub const READ_COMMANDS: &[Command] = &[
    cmd(Action::OpenPlace, Msg::PlaceOpen, FILE),
    // NV-24: 並べ替えの窓(キー `S` を外しても、検索の欄を隠しても開けるように)。
    cmd(Action::SortMenu, Msg::CmdSortMenu, SHAPE),
    cmd(Action::RegisterPlace, Msg::PlaceRegister, FILE),
    cmd(Action::OpenLink, Msg::CmdOpenLink, FIND),
    cmd(Action::LinkedRows, Msg::CmdLinkedRows, FIND),
    // WS-2: ワークスペース(ノートを書かない。workspaces.toml だけ)。
    cmd(Action::WsNew, Msg::CmdWsNew, FILE),
    cmd(Action::WsAdd, Msg::CmdWsAdd, FILE),
    cmd(Action::WsRemove, Msg::CmdWsRemove, FILE),
    cmd(Action::WsOpen, Msg::CmdWsOpen, FILE),
];

/// パレットとヘルプに出すキーの無いコマンド(読むだけなら書くコマンドを除く)。
pub fn commands(readonly: bool) -> impl Iterator<Item = &'static Command> {
    let writes: &'static [Command] = if readonly { &[] } else { COMMANDS };
    writes.iter().chain(READ_COMMANDS)
}

/// 既定のキー(SR-13・SR-16・SR-18・WB-10・BV-16)。
pub const BINDINGS: &[Binding] = &[
    b("k", Action::Up, Msg::KeyUp, MOVE, 0),
    b("Up", Action::Up, Msg::KeyUp, MOVE, 0),
    b("j", Action::Down, Msg::KeyDown, MOVE, 0),
    b("Down", Action::Down, Msg::KeyDown, MOVE, 0),
    b("h", Action::Left, Msg::KeyLeft, MOVE, 0),
    b("Left", Action::Left, Msg::KeyLeft, MOVE, 0),
    b("l", Action::Right, Msg::KeyRight, MOVE, 0),
    b("Right", Action::Right, Msg::KeyRight, MOVE, 0),
    b("Home", Action::FirstColumn, Msg::KeyFirstColumn, MOVE, 0),
    b("0", Action::FirstColumn, Msg::KeyFirstColumn, MOVE, 0),
    b("End", Action::LastColumn, Msg::KeyLastColumn, MOVE, 0),
    b("$", Action::LastColumn, Msg::KeyLastColumn, MOVE, 0),
    b("PageUp", Action::PageUp, Msg::KeyPageUp, MOVE, 0),
    b("PageDown", Action::PageDown, Msg::KeyPageDown, MOVE, 0),
    b("Ctrl+u", Action::HalfPageUp, Msg::KeyHalfPageUp, MOVE, 0),
    b(
        "Ctrl+d",
        Action::HalfPageDown,
        Msg::KeyHalfPageDown,
        MOVE,
        0,
    ),
    b("g g", Action::Top, Msg::KeyFirstRow, MOVE, 0),
    b("G", Action::Bottom, Msg::KeyLastRow, MOVE, 0),
    b("Tab", Action::NextCell, Msg::KeyNextCell, MOVE, 0),
    b("Shift+Tab", Action::PrevCell, Msg::KeyPrevCell, MOVE, 0),
    b("Enter", Action::Edit, Msg::KeyEdit, EDIT, 1),
    b("Backspace", Action::Clear, Msg::KeyClear, EDIT, 2),
    b("Delete", Action::Clear, Msg::KeyClear, EDIT, 2),
    b("u", Action::Undo, Msg::KeyUndo, EDIT, 3),
    b("Ctrl+z", Action::Undo, Msg::KeyUndo, EDIT, 3),
    b("Ctrl+r", Action::Redo, Msg::KeyRedo, EDIT, 6),
    b("U", Action::Redo, Msg::KeyRedo, EDIT, 6),
    b("Ctrl+Shift+z", Action::Redo, Msg::KeyRedo, EDIT, 6),
    // CE-25・CE-27: 新しいノート(`n` は検索の次の一致なので既定は `a`。割り当て直せる)。
    b("a", Action::NewNote, Msg::KeyNewNote, EDIT, 0),
    b("Ctrl+s", Action::Save, Msg::KeySave, FILE, 5),
    b("q", Action::Quit, Msg::KeyQuit, FILE, 4),
    b("Ctrl+g", Action::CancelLoad, Msg::KeyCancelLoad, FILE, 0),
    // ヘルプとパレット(SR-5・SR-14・SR-16)
    b("?", Action::Help, Msg::KeyHelp, FIND, 7),
    b(":", Action::Palette, Msg::KeyCommands, FIND, 8),
    b("Ctrl+p", Action::Palette, Msg::KeyCommands, FIND, 8),
    // SR-24: その場の操作の一覧(セルの右クリックでも開く)。下の帯には出さない(順位 0)。
    b("x", Action::Menu, Msg::KeyActionMenu, FIND, 0),
    // エディタとコピー(SR-8・OUT-1)
    b("e", Action::OpenEditor, Msg::KeyOpenEditor, OUTSIDE, 0),
    b("y", Action::Copy, Msg::KeyCopyCell, OUTSIDE, 0),
    b("Ctrl+c", Action::Copy, Msg::KeyCopyCell, OUTSIDE, 0),
    b("Y", Action::CopyRow, Msg::KeyCopyRow, OUTSIDE, 0),
    // 表の形(BV-13・SR-3・NV-4)。見出しの行の Enter は開閉(SR-2。Enter の「編集」と同じ動作)。
    b("[", Action::PrevView, Msg::KeyPrevView, SHAPE, 0),
    b("]", Action::NextView, Msg::KeyNextView, SHAPE, 0),
    b("<", Action::Narrower, Msg::KeyNarrower, SHAPE, 0),
    b(">", Action::Wider, Msg::KeyWider, SHAPE, 0),
    b("s", Action::Sort, Msg::KeySort, SHAPE, 0),
    // NV-24: 並べ替えの窓(検索の欄の右の「並べ替え」と同じ)。
    b("S", Action::SortMenu, Msg::CmdSortMenu, SHAPE, 0),
    b("-", Action::HideColumn, Msg::KeyHideColumn, SHAPE, 0),
    b("+", Action::ShowColumn, Msg::KeyShowColumn, SHAPE, 0),
    // REL-7・REL-8: 関係マップ(上の端のタブ・パレットからも)。
    b("R", Action::RelationMap, Msg::KeyRelationMap, FIND, 0),
    b("Z", Action::ToggleTree, Msg::KeyToggleTree, SHAPE, 0),
    b("A", Action::AddColumn, Msg::KeyAddColumn, SHAPE, 0),
    b(
        "H",
        Action::MoveColumnLeft,
        Msg::KeyMoveColumnLeft,
        SHAPE,
        0,
    ),
    b(
        "L",
        Action::MoveColumnRight,
        Msg::KeyMoveColumnRight,
        SHAPE,
        0,
    ),
    b("F", Action::Freeze, Msg::KeyFreeze, SHAPE, 0),
    // 検索と絞り込み(NV-1・NV-2・NV-8・SR-13・SR-18)
    b("/", Action::Search, Msg::KeySearch, SEARCH, 0),
    b("n", Action::SearchNext, Msg::KeySearchNext, SEARCH, 0),
    b("N", Action::SearchPrev, Msg::KeySearchPrev, SEARCH, 0),
    b("\\", Action::QuickFilter, Msg::KeyQuickFilter, SEARCH, 0),
    b("*", Action::HighlightSame, Msg::KeyHighlightSame, SEARCH, 0),
    b(",", Action::FilterSame, Msg::KeyFilterSame, SEARCH, 0),
    // NV-9: 選んでいる列の値の頻度表(選んだ値で絞る)。
    b("%", Action::Freq, Msg::KeyFrequency, SEARCH, 0),
    // 選択と詳細(NV-5・NV-6・SR-16・SR-18)
    b("Space", Action::Mark, Msg::KeyMark, SELECT, 0),
    b("v", Action::Visual, Msg::KeyVisual, SELECT, 0),
    b("Shift+Up", Action::ExtendUp, Msg::KeyExtendUp, SELECT, 0),
    b(
        "Shift+Down",
        Action::ExtendDown,
        Msg::KeyExtendDown,
        SELECT,
        0,
    ),
    b("Ctrl+a", Action::SelectAll, Msg::KeySelectAll, SELECT, 0),
    b("Esc", Action::Escape, Msg::KeyEscape, SELECT, 0),
    b("K", Action::Detail, Msg::KeyDetail, SELECT, 0),
    // ビューの設定と帯(NV-13・NV-16・NV-22)
    b("o", Action::ViewSettings, Msg::KeyViewSettings, SHAPE, 0),
    b("f", Action::FocusChips, Msg::KeyFocusChips, SHAPE, 0),
    // 編集(CE-1・CE-11・SR-16)。文字のキーは割り当てない(入力に使う)。
    m(
        Mode::Edit,
        "Enter",
        Action::Commit,
        Msg::KeyCommit,
        INPUT,
        1,
    ),
    m(Mode::Edit, "Esc", Action::Cancel, Msg::KeyCancel, INPUT, 2),
    m(
        Mode::Edit,
        "Tab",
        Action::CommitNext,
        Msg::KeyCommitNext,
        INPUT,
        3,
    ),
    m(
        Mode::Edit,
        "Shift+Tab",
        Action::CommitPrev,
        Msg::KeyCommitPrev,
        INPUT,
        4,
    ),
    m(
        Mode::Edit,
        "Ctrl+r",
        Action::Revert,
        Msg::KeyRevert,
        INPUT,
        5,
    ),
    m(
        Mode::Edit,
        "Ctrl+u",
        Action::ClearInput,
        Msg::KeyClearInput,
        INPUT,
        0,
    ),
    // 候補のリスト(CE-3)。文字・BS・←→ を打つと自由入力に切り替わる。
    // 日付のカレンダー(CE-21)が出ているときは、↑↓ で1週、←→ で1日を動かす。
    m(
        Mode::Edit,
        "Up",
        Action::ListUp,
        Msg::KeyListUpWeek,
        INPUT,
        0,
    ),
    m(
        Mode::Edit,
        "Down",
        Action::ListDown,
        Msg::KeyListDownWeek,
        INPUT,
        0,
    ),
    m(
        Mode::Edit,
        "Ctrl+p",
        Action::ListUp,
        Msg::KeyListUpWeek,
        INPUT,
        0,
    ),
    m(
        Mode::Edit,
        "Ctrl+n",
        Action::ListDown,
        Msg::KeyListDownWeek,
        INPUT,
        0,
    ),
    m(
        Mode::Edit,
        "Left",
        Action::CursorLeft,
        Msg::KeyLeftDay,
        INPUT,
        0,
    ),
    m(
        Mode::Edit,
        "Right",
        Action::CursorRight,
        Msg::KeyRightDay,
        INPUT,
        0,
    ),
    // 日付の入力(CE-21)。文字1つのキーは入力に使うので Ctrl と PageUp・PageDown。
    m(
        Mode::Edit,
        "PageUp",
        Action::PageUp,
        Msg::KeyDatePrevMonth,
        INPUT,
        0,
    ),
    m(
        Mode::Edit,
        "PageDown",
        Action::PageDown,
        Msg::KeyDateNextMonth,
        INPUT,
        0,
    ),
    // CE-23・CE-24: PageUp・PageDown の無いキーボードでも月と年を飛ばせるように Shift+矢印。
    // カレンダーが出ていなければ ←→・↑↓ と同じなので、表示名も ←→・↑↓ の形(「左・前の日」)にそろえる。
    m(
        Mode::Edit,
        "Shift+Left",
        Action::PrevMonth,
        Msg::KeyLeftMonth,
        INPUT,
        0,
    ),
    m(
        Mode::Edit,
        "Shift+Right",
        Action::NextMonth,
        Msg::KeyRightMonth,
        INPUT,
        0,
    ),
    m(
        Mode::Edit,
        "Shift+Up",
        Action::PrevYear,
        Msg::KeyListUpYear,
        INPUT,
        0,
    ),
    m(
        Mode::Edit,
        "Shift+Down",
        Action::NextYear,
        Msg::KeyListDownYear,
        INPUT,
        0,
    ),
    m(
        Mode::Edit,
        "Ctrl+t",
        Action::Today,
        Msg::KeyDateToday,
        INPUT,
        0,
    ),
    // CE-30・CE-31: 日時のカレンダーの時刻の欄(Tab は確定して右へ、Ctrl+t は今日なので Ctrl+o)。
    m(
        Mode::Edit,
        "Ctrl+o",
        Action::TimeFocus,
        Msg::KeyTimeFocus,
        INPUT,
        0,
    ),
    m(
        Mode::Edit,
        "Ctrl+d",
        Action::Clear,
        Msg::KeyDateClear,
        INPUT,
        0,
    ),
    m(
        Mode::Edit,
        "Home",
        Action::CursorHome,
        Msg::KeyCursorHome,
        INPUT,
        0,
    ),
    m(
        Mode::Edit,
        "End",
        Action::CursorEnd,
        Msg::KeyCursorEnd,
        INPUT,
        0,
    ),
    m(
        Mode::Edit,
        "Backspace",
        Action::DeleteBack,
        Msg::KeyDeleteBack,
        INPUT,
        0,
    ),
    m(
        Mode::Edit,
        "Delete",
        Action::DeleteForward,
        Msg::KeyDeleteForward,
        INPUT,
        0,
    ),
    // 検索・絞り込みの入力(NV-1・NV-2)。文字のキーは割り当てない(語に使う)。
    m(
        Mode::Search,
        "Enter",
        Action::Commit,
        Msg::KeyCommit,
        IN_PROMPT,
        1,
    ),
    m(
        Mode::Search,
        "Esc",
        Action::Cancel,
        Msg::KeyCancel,
        IN_PROMPT,
        2,
    ),
    m(
        Mode::Search,
        "Backspace",
        Action::DeleteBack,
        Msg::KeyDeleteBack,
        IN_PROMPT,
        0,
    ),
    m(
        Mode::Filter,
        "Enter",
        Action::Commit,
        Msg::KeyCommit,
        IN_PROMPT,
        1,
    ),
    m(
        Mode::Filter,
        "Esc",
        Action::Cancel,
        Msg::KeyFilterClear,
        IN_PROMPT,
        2,
    ),
    m(
        Mode::Filter,
        "Backspace",
        Action::DeleteBack,
        Msg::KeyDeleteBack,
        IN_PROMPT,
        0,
    ),
    // 詳細の表示(NV-6・SR-16)
    m(
        Mode::Detail,
        "Enter",
        Action::Edit,
        Msg::KeyEdit,
        IN_DETAIL,
        1,
    ),
    m(
        Mode::Detail,
        "Esc",
        Action::Close,
        Msg::KeyClose,
        IN_DETAIL,
        2,
    ),
    m(
        Mode::Detail,
        "K",
        Action::Close,
        Msg::KeyClose,
        IN_DETAIL,
        2,
    ),
    m(Mode::Detail, "j", Action::Down, Msg::KeyDown, IN_DETAIL, 3),
    m(
        Mode::Detail,
        "Down",
        Action::Down,
        Msg::KeyDown,
        IN_DETAIL,
        3,
    ),
    m(Mode::Detail, "k", Action::Up, Msg::KeyUp, IN_DETAIL, 4),
    m(Mode::Detail, "Up", Action::Up, Msg::KeyUp, IN_DETAIL, 4),
    m(
        Mode::Detail,
        "PageDown",
        Action::PageDown,
        Msg::KeyPageDown,
        IN_DETAIL,
        0,
    ),
    m(
        Mode::Detail,
        "PageUp",
        Action::PageUp,
        Msg::KeyPageUp,
        IN_DETAIL,
        0,
    ),
    m(Mode::Detail, "g g", Action::Top, Msg::KeyTop, IN_DETAIL, 0),
    m(
        Mode::Detail,
        "G",
        Action::Bottom,
        Msg::KeyBottom,
        IN_DETAIL,
        0,
    ),
    m(Mode::Detail, "?", Action::Help, Msg::KeyHelp, IN_DETAIL, 5),
    // ビューの設定の中(NV-13・NV-18・NV-19・NV-21・NV-22)
    m(
        Mode::Settings,
        "Enter",
        Action::Run,
        Msg::KeyDecide,
        IN_SETTINGS,
        1,
    ),
    m(
        Mode::Settings,
        "Space",
        Action::Toggle,
        Msg::KeyToggle,
        IN_SETTINGS,
        2,
    ),
    m(
        Mode::Settings,
        "Esc",
        Action::Cancel,
        Msg::KeyBackCancel,
        IN_SETTINGS,
        3,
    ),
    m(
        Mode::Settings,
        "Tab",
        Action::NextSection,
        Msg::KeyNextSection,
        IN_SETTINGS,
        4,
    ),
    m(
        Mode::Settings,
        "Shift+Tab",
        Action::PrevSection,
        Msg::KeyPrevSection,
        IN_SETTINGS,
        0,
    ),
    m(
        Mode::Settings,
        "j",
        Action::Down,
        Msg::KeyDown,
        IN_SETTINGS,
        0,
    ),
    m(
        Mode::Settings,
        "Down",
        Action::Down,
        Msg::KeyDown,
        IN_SETTINGS,
        0,
    ),
    m(Mode::Settings, "k", Action::Up, Msg::KeyUp, IN_SETTINGS, 0),
    m(Mode::Settings, "Up", Action::Up, Msg::KeyUp, IN_SETTINGS, 0),
    m(
        Mode::Settings,
        "h",
        Action::Left,
        Msg::KeyLeft,
        IN_SETTINGS,
        0,
    ),
    m(
        Mode::Settings,
        "Left",
        Action::Left,
        Msg::KeyLeft,
        IN_SETTINGS,
        0,
    ),
    m(
        Mode::Settings,
        "l",
        Action::Right,
        Msg::KeyRight,
        IN_SETTINGS,
        0,
    ),
    m(
        Mode::Settings,
        "Right",
        Action::Right,
        Msg::KeyRight,
        IN_SETTINGS,
        0,
    ),
    m(
        Mode::Settings,
        "K",
        Action::MoveItemUp,
        Msg::KeyMoveItemUp,
        IN_SETTINGS,
        5,
    ),
    m(
        Mode::Settings,
        "Shift+Up",
        Action::MoveItemUp,
        Msg::KeyMoveItemUp,
        IN_SETTINGS,
        5,
    ),
    m(
        Mode::Settings,
        "J",
        Action::MoveItemDown,
        Msg::KeyMoveItemDown,
        IN_SETTINGS,
        6,
    ),
    m(
        Mode::Settings,
        "Shift+Down",
        Action::MoveItemDown,
        Msg::KeyMoveItemDown,
        IN_SETTINGS,
        6,
    ),
    m(
        Mode::Settings,
        "d",
        Action::RemoveItem,
        Msg::KeyDelete,
        IN_SETTINGS,
        7,
    ),
    m(
        Mode::Settings,
        "Delete",
        Action::RemoveItem,
        Msg::KeyDelete,
        IN_SETTINGS,
        7,
    ),
    m(
        Mode::Settings,
        "Backspace",
        Action::RemoveItem,
        Msg::KeyDelete,
        IN_SETTINGS,
        7,
    ),
    m(
        Mode::Settings,
        "?",
        Action::Help,
        Msg::KeyHelp,
        IN_SETTINGS,
        0,
    ),
    m(
        Mode::SettingsText,
        "Enter",
        Action::Commit,
        Msg::KeyDecide,
        IN_SETTINGS,
        1,
    ),
    m(
        Mode::SettingsText,
        "Esc",
        Action::Cancel,
        Msg::KeyBack,
        IN_SETTINGS,
        2,
    ),
    m(
        Mode::SettingsText,
        "Backspace",
        Action::DeleteBack,
        Msg::KeyDeleteBack,
        IN_SETTINGS,
        0,
    ),
    // 設定の帯の中(NV-16・NV-22)
    m(
        Mode::Chips,
        "Backspace",
        Action::RemoveItem,
        Msg::KeyRemove,
        IN_CHIPS,
        1,
    ),
    m(
        Mode::Chips,
        "Delete",
        Action::RemoveItem,
        Msg::KeyRemove,
        IN_CHIPS,
        1,
    ),
    m(
        Mode::Chips,
        "h",
        Action::Left,
        Msg::KeyLeftItem,
        IN_CHIPS,
        2,
    ),
    m(
        Mode::Chips,
        "Left",
        Action::Left,
        Msg::KeyLeftItem,
        IN_CHIPS,
        2,
    ),
    m(
        Mode::Chips,
        "l",
        Action::Right,
        Msg::KeyRightItem,
        IN_CHIPS,
        3,
    ),
    m(
        Mode::Chips,
        "Right",
        Action::Right,
        Msg::KeyRightItem,
        IN_CHIPS,
        3,
    ),
    m(
        Mode::Chips,
        "Enter",
        Action::ViewSettings,
        Msg::KeyViewSettings,
        IN_CHIPS,
        4,
    ),
    m(
        Mode::Chips,
        "Esc",
        Action::Close,
        Msg::KeyBackToTable,
        IN_CHIPS,
        5,
    ),
    m(
        Mode::Chips,
        "f",
        Action::Close,
        Msg::KeyBackToTable,
        IN_CHIPS,
        5,
    ),
    m(Mode::Chips, "?", Action::Help, Msg::KeyHelp, IN_CHIPS, 0),
    // リストの選択の中(CE-16・CE-17・CE-19・CE-11)。文字1つのキーは割り当てない(打つ文字は検索に入る)。
    // 検索が空のとき Space=付け外し・Enter=確定。検索中は Space=空白・Enter=選んだ候補を付けるだけ
    // (既に付いていれば何もしない。新規なら足す)で検索を空に戻す。Tab はいつでも確定。
    // CE-26: 新しいノートの窓では、どの欄からでも Ctrl+S で作る(入力とリストの選択の両方)。
    m(
        Mode::ListPick,
        "Ctrl+s",
        Action::CreateNote,
        Msg::KeyCreateNote,
        INPUT,
        0,
    ),
    m(
        Mode::Edit,
        "Ctrl+s",
        Action::CreateNote,
        Msg::KeyCreateNote,
        INPUT,
        0,
    ),
    // CE-33: 作ってすぐエディタで開く。
    m(
        Mode::ListPick,
        "Ctrl+e",
        Action::CreateNoteEdit,
        Msg::KeyCreateNoteEdit,
        INPUT,
        0,
    ),
    m(
        Mode::Edit,
        "Ctrl+e",
        Action::CreateNoteEdit,
        Msg::KeyCreateNoteEdit,
        INPUT,
        0,
    ),
    m(
        Mode::ListPick,
        "Tab",
        Action::Commit,
        Msg::KeyCommit,
        IN_PICK,
        1,
    ),
    m(
        Mode::ListPick,
        "Enter",
        Action::Run,
        Msg::KeyPickRun,
        IN_PICK,
        2,
    ),
    m(
        Mode::ListPick,
        "Space",
        Action::Toggle,
        Msg::KeyPickToggle,
        IN_PICK,
        3,
    ),
    m(
        Mode::ListPick,
        "Esc",
        Action::Cancel,
        Msg::KeyCancel,
        IN_PICK,
        4,
    ),
    m(
        Mode::ListPick,
        "Ctrl+r",
        Action::Revert,
        Msg::KeyPickRevert,
        IN_PICK,
        5,
    ),
    m(
        Mode::ListPick,
        "Up",
        Action::ListUp,
        Msg::KeyPrevItem,
        IN_PICK,
        0,
    ),
    m(
        Mode::ListPick,
        "Down",
        Action::ListDown,
        Msg::KeyNextItem,
        IN_PICK,
        0,
    ),
    m(
        Mode::ListPick,
        "Ctrl+p",
        Action::ListUp,
        Msg::KeyPrevItem,
        IN_PICK,
        0,
    ),
    m(
        Mode::ListPick,
        "Ctrl+n",
        Action::ListDown,
        Msg::KeyNextItem,
        IN_PICK,
        0,
    ),
    m(
        Mode::ListPick,
        "PageUp",
        Action::PageUp,
        Msg::KeyPageUp,
        IN_PICK,
        0,
    ),
    m(
        Mode::ListPick,
        "PageDown",
        Action::PageDown,
        Msg::KeyPageDown,
        IN_PICK,
        0,
    ),
    m(
        Mode::ListPick,
        "Backspace",
        Action::DeleteBack,
        Msg::KeyDeleteBack,
        IN_PICK,
        0,
    ),
    // 保存の確認(WB-9・WB-16)
    m(
        Mode::Confirm,
        "Enter",
        Action::SaveAll,
        Msg::KeySaveAll,
        REVIEW,
        1,
    ),
    m(Mode::Confirm, "Esc", Action::Back, Msg::KeyBack, REVIEW, 2),
    m(Mode::Confirm, "q", Action::Back, Msg::KeyBack, REVIEW, 2),
    m(
        Mode::Confirm,
        "j",
        Action::NextFile,
        Msg::KeyNextFile,
        REVIEW,
        5,
    ),
    m(
        Mode::Confirm,
        "Down",
        Action::NextFile,
        Msg::KeyNextFile,
        REVIEW,
        5,
    ),
    m(
        Mode::Confirm,
        "k",
        Action::PrevFile,
        Msg::KeyPrevFile,
        REVIEW,
        6,
    ),
    m(
        Mode::Confirm,
        "Up",
        Action::PrevFile,
        Msg::KeyPrevFile,
        REVIEW,
        6,
    ),
    m(
        Mode::Confirm,
        "o",
        Action::Overwrite,
        Msg::KeyOverwrite,
        REVIEW,
        3,
    ),
    m(
        Mode::Confirm,
        "d",
        Action::DiscardRow,
        Msg::KeyDiscardRow,
        REVIEW,
        4,
    ),
    m(Mode::Confirm, "?", Action::Help, Msg::KeyHelp, REVIEW, 0),
    // 終了の確認(WB-11)
    m(Mode::Quit, "s", Action::QuitSave, Msg::KeyQuitSave, QUIT, 1),
    m(
        Mode::Quit,
        "d",
        Action::QuitDiscard,
        Msg::KeyQuitDiscard,
        QUIT,
        2,
    ),
    m(Mode::Quit, "Esc", Action::Back, Msg::KeyBack, QUIT, 3),
    m(Mode::Quit, "?", Action::Help, Msg::KeyHelp, QUIT, 0),
    // ヘルプの中(SR-5)
    m(Mode::Help, "Esc", Action::Close, Msg::KeyClose, IN_HELP, 1),
    m(Mode::Help, "q", Action::Close, Msg::KeyClose, IN_HELP, 1),
    m(Mode::Help, "?", Action::Close, Msg::KeyClose, IN_HELP, 1),
    m(
        Mode::Help,
        "j",
        Action::Down,
        Msg::KeyScrollDown,
        IN_HELP,
        2,
    ),
    m(
        Mode::Help,
        "Down",
        Action::Down,
        Msg::KeyScrollDown,
        IN_HELP,
        2,
    ),
    m(Mode::Help, "k", Action::Up, Msg::KeyScrollUp, IN_HELP, 3),
    m(Mode::Help, "Up", Action::Up, Msg::KeyScrollUp, IN_HELP, 3),
    m(
        Mode::Help,
        "PageDown",
        Action::PageDown,
        Msg::KeyPageDown,
        IN_HELP,
        0,
    ),
    m(
        Mode::Help,
        "PageUp",
        Action::PageUp,
        Msg::KeyPageUp,
        IN_HELP,
        0,
    ),
    m(Mode::Help, "g g", Action::Top, Msg::KeyTop, IN_HELP, 0),
    m(Mode::Help, "G", Action::Bottom, Msg::KeyBottom, IN_HELP, 0),
    // パレットの中(SR-14)。文字のキーは割り当てない(探す文字に使う)。
    m(
        Mode::Palette,
        "Enter",
        Action::Run,
        Msg::KeyRun,
        IN_PALETTE,
        1,
    ),
    m(
        Mode::Palette,
        "Esc",
        Action::Close,
        Msg::KeyClose,
        IN_PALETTE,
        2,
    ),
    m(
        Mode::Palette,
        "Down",
        Action::Down,
        Msg::KeyNextItem,
        IN_PALETTE,
        3,
    ),
    m(
        Mode::Palette,
        "Ctrl+n",
        Action::Down,
        Msg::KeyNextItem,
        IN_PALETTE,
        3,
    ),
    m(
        Mode::Palette,
        "Up",
        Action::Up,
        Msg::KeyPrevItem,
        IN_PALETTE,
        4,
    ),
    m(
        Mode::Palette,
        "Ctrl+p",
        Action::Up,
        Msg::KeyPrevItem,
        IN_PALETTE,
        4,
    ),
    m(
        Mode::Palette,
        "Backspace",
        Action::DeleteBack,
        Msg::KeyDeleteBack,
        IN_PALETTE,
        0,
    ),
    // その場の操作の一覧の中(SR-24)。ほかのキーは表のモードのキーとして読み、一覧の項目なら実行する(menu.rs)。
    m(Mode::Menu, "Enter", Action::Run, Msg::KeyRun, IN_MENU, 1),
    m(Mode::Menu, "Esc", Action::Close, Msg::KeyClose, IN_MENU, 2),
    m(Mode::Menu, "j", Action::Down, Msg::KeyNextItem, IN_MENU, 3),
    m(
        Mode::Menu,
        "Down",
        Action::Down,
        Msg::KeyNextItem,
        IN_MENU,
        3,
    ),
    m(Mode::Menu, "k", Action::Up, Msg::KeyPrevItem, IN_MENU, 4),
    m(Mode::Menu, "Up", Action::Up, Msg::KeyPrevItem, IN_MENU, 4),
    m(Mode::Menu, "g g", Action::Top, Msg::KeyTop, IN_MENU, 0),
    m(Mode::Menu, "G", Action::Bottom, Msg::KeyBottom, IN_MENU, 0),
    // 並べ替えの窓の中(NV-24。sorts.rs)。Enter で向きを変えるか列を足し、d で外し、K・J で順を入れ替える。
    m(
        Mode::Sorts,
        "Enter",
        Action::Run,
        Msg::KeySortsRun,
        IN_SORTS,
        1,
    ),
    m(
        Mode::Sorts,
        "Esc",
        Action::Close,
        Msg::KeyClose,
        IN_SORTS,
        2,
    ),
    m(Mode::Sorts, "q", Action::Close, Msg::KeyClose, IN_SORTS, 0),
    m(
        Mode::Sorts,
        "j",
        Action::Down,
        Msg::KeyNextItem,
        IN_SORTS,
        0,
    ),
    m(
        Mode::Sorts,
        "Down",
        Action::Down,
        Msg::KeyNextItem,
        IN_SORTS,
        0,
    ),
    m(Mode::Sorts, "k", Action::Up, Msg::KeyPrevItem, IN_SORTS, 0),
    m(Mode::Sorts, "Up", Action::Up, Msg::KeyPrevItem, IN_SORTS, 0),
    m(
        Mode::Sorts,
        "d",
        Action::RemoveItem,
        Msg::KeyRemove,
        IN_SORTS,
        3,
    ),
    m(
        Mode::Sorts,
        "Delete",
        Action::RemoveItem,
        Msg::KeyRemove,
        IN_SORTS,
        0,
    ),
    m(
        Mode::Sorts,
        "Backspace",
        Action::RemoveItem,
        Msg::KeyRemove,
        IN_SORTS,
        0,
    ),
    m(
        Mode::Sorts,
        "K",
        Action::MoveItemUp,
        Msg::KeyMoveItemUp,
        IN_SORTS,
        4,
    ),
    m(
        Mode::Sorts,
        "Shift+Up",
        Action::MoveItemUp,
        Msg::KeyMoveItemUp,
        IN_SORTS,
        0,
    ),
    m(
        Mode::Sorts,
        "J",
        Action::MoveItemDown,
        Msg::KeyMoveItemDown,
        IN_SORTS,
        5,
    ),
    m(
        Mode::Sorts,
        "Shift+Down",
        Action::MoveItemDown,
        Msg::KeyMoveItemDown,
        IN_SORTS,
        0,
    ),
    // 列の値の頻度表の中(NV-9。freq.rs)。Enter で選んだ値の行だけに絞り、Esc で何もせず閉じる。
    m(
        Mode::Freq,
        "Enter",
        Action::Run,
        Msg::KeyFreqRun,
        IN_FREQ,
        1,
    ),
    m(Mode::Freq, "Esc", Action::Close, Msg::KeyClose, IN_FREQ, 2),
    m(Mode::Freq, "j", Action::Down, Msg::KeyNextItem, IN_FREQ, 3),
    m(
        Mode::Freq,
        "Down",
        Action::Down,
        Msg::KeyNextItem,
        IN_FREQ,
        3,
    ),
    m(Mode::Freq, "k", Action::Up, Msg::KeyPrevItem, IN_FREQ, 4),
    m(Mode::Freq, "Up", Action::Up, Msg::KeyPrevItem, IN_FREQ, 4),
    m(Mode::Freq, "g g", Action::Top, Msg::KeyTop, IN_FREQ, 0),
    m(Mode::Freq, "G", Action::Bottom, Msg::KeyBottom, IN_FREQ, 0),
    // REL-7: 関係マップ。↑↓ で表、←→ でその表のつながり、Enter で表を開く。
    m(
        Mode::Relations,
        "Enter",
        Action::Run,
        Msg::KeyRelOpen,
        IN_RELMAP,
        1,
    ),
    m(
        Mode::Relations,
        "Esc",
        Action::Close,
        Msg::KeyRelBack,
        IN_RELMAP,
        2,
    ),
    m(
        Mode::Relations,
        "R",
        Action::RelationMap,
        Msg::KeyRelBack,
        IN_RELMAP,
        0,
    ),
    m(
        Mode::Relations,
        "q",
        Action::Close,
        Msg::KeyRelBack,
        IN_RELMAP,
        0,
    ),
    m(
        Mode::Relations,
        "j",
        Action::Down,
        Msg::KeyRelNextTable,
        IN_RELMAP,
        3,
    ),
    m(
        Mode::Relations,
        "Down",
        Action::Down,
        Msg::KeyRelNextTable,
        IN_RELMAP,
        3,
    ),
    m(
        Mode::Relations,
        "k",
        Action::Up,
        Msg::KeyRelPrevTable,
        IN_RELMAP,
        0,
    ),
    m(
        Mode::Relations,
        "Up",
        Action::Up,
        Msg::KeyRelPrevTable,
        IN_RELMAP,
        0,
    ),
    m(
        Mode::Relations,
        "l",
        Action::Right,
        Msg::KeyRelNextLink,
        IN_RELMAP,
        4,
    ),
    m(
        Mode::Relations,
        "Right",
        Action::Right,
        Msg::KeyRelNextLink,
        IN_RELMAP,
        4,
    ),
    m(
        Mode::Relations,
        "h",
        Action::Left,
        Msg::KeyRelPrevLink,
        IN_RELMAP,
        0,
    ),
    m(
        Mode::Relations,
        "Left",
        Action::Left,
        Msg::KeyRelPrevLink,
        IN_RELMAP,
        0,
    ),
    m(
        Mode::Relations,
        "a",
        Action::NewNote,
        Msg::KeyNewNote,
        IN_RELMAP,
        5,
    ),
    m(
        Mode::Relations,
        "?",
        Action::Help,
        Msg::KeyHelp,
        IN_RELMAP,
        0,
    ),
];

/// 全角の英数字・記号と `、`・`・` を、同じキーの位置の半角に直す(SR-17)。
pub fn halfwidth(c: char) -> char {
    match c {
        '\u{ff01}'..='\u{ff5e}' => char::from_u32(c as u32 - 0xfee0).unwrap_or(c),
        '\u{3000}' => ' ',
        '、' | '､' => ',',
        '・' | '･' => '/',
        _ => c,
    }
}

/// 端末から来たキーの名前。表の `key` と同じ形にする。
pub fn key_name(ev: &KeyEvent) -> Option<String> {
    let ctrl = ev.modifiers.contains(KeyModifiers::CONTROL);
    let alt = ev.modifiers.contains(KeyModifiers::ALT);
    let shift = ev.modifiers.contains(KeyModifiers::SHIFT);
    let mut pre = String::new();
    if ctrl {
        pre.push_str("Ctrl+");
    }
    if alt {
        pre.push_str("Alt+");
    }
    let name = match ev.code {
        KeyCode::Char(c) => {
            let c = halfwidth(c);
            if ctrl || alt {
                if shift || c.is_ascii_uppercase() {
                    pre.push_str("Shift+");
                }
                return Some(format!("{pre}{}", c.to_ascii_lowercase()));
            }
            if c == ' ' {
                return Some("Space".into());
            }
            return Some(c.to_string());
        }
        KeyCode::Enter => "Enter",
        KeyCode::Esc => "Esc",
        KeyCode::Backspace => "Backspace",
        KeyCode::Delete => "Delete",
        KeyCode::Tab if shift => "Shift+Tab",
        KeyCode::Tab => "Tab",
        KeyCode::BackTab => "Shift+Tab",
        // Shift+矢印(NV-5 の範囲の選択)。
        KeyCode::Up if shift => "Shift+Up",
        KeyCode::Down if shift => "Shift+Down",
        // Shift+←→(CE-23 のカレンダーの月)。表に無いモードでは ←→ として引き直す(`unshifted`)。
        KeyCode::Left if shift => "Shift+Left",
        KeyCode::Right if shift => "Shift+Right",
        KeyCode::Up => "Up",
        KeyCode::Down => "Down",
        KeyCode::Left => "Left",
        KeyCode::Right => "Right",
        KeyCode::Home => "Home",
        KeyCode::End => "End",
        KeyCode::PageUp => "PageUp",
        KeyCode::PageDown => "PageDown",
        _ => return None,
    };
    Some(format!("{pre}{name}"))
}

/// Shift+←→ を ←→ に戻した名前(Shift+←→ を割り当てていないモードでは、Shift の無い ←→ と同じに扱う)。
/// Shift+←→ でなければ None。
pub fn unshifted(key: &str) -> Option<String> {
    ["Left", "Right"].iter().find_map(|k| {
        key.strip_suffix(&format!("Shift+{k}"))
            .map(|pre| format!("{pre}{k}"))
    })
}

/// キーの表を引く。
pub fn lookup(table: &[Binding], mode: Mode, key: &str) -> Option<Action> {
    table
        .iter()
        .find(|b| b.mode == mode && b.key == key)
        .map(|b| b.action)
}

/// `key` が前置き(`g g` の `g`)か。
pub fn is_prefix(table: &[Binding], mode: Mode, key: &str) -> bool {
    table.iter().any(|b| {
        b.mode == mode
            && b.key
                .strip_prefix(key)
                .is_some_and(|rest| rest.starts_with(' '))
    })
}

/// キーの見せ方(下の帯・ヘルプ)。
pub fn display(key: &str) -> String {
    key.split(' ')
        .map(|k| match k {
            "Up" => "↑".to_string(),
            "Down" => "↓".to_string(),
            "Left" => "←".to_string(),
            "Right" => "→".to_string(),
            "Backspace" => "BS".to_string(),
            "Delete" => "Del".to_string(),
            // 修飾の付いた矢印(`Shift+Left` → `Shift+←`)。
            k => arrow(k).unwrap_or_else(|| match k.strip_prefix("Ctrl+") {
                Some(c) if c.len() == 1 => format!("^{}", c.to_ascii_uppercase()),
                Some(c) => match c.strip_prefix("Shift+") {
                    Some(c) if c.len() == 1 => format!("^Shift+{}", c.to_ascii_uppercase()),
                    _ => k.to_string(),
                },
                _ => k.to_string(),
            }),
        })
        .collect()
}

/// 修飾の付いた矢印のキーの見せ方(`Shift+Left` → `Shift+←`)。修飾の付いた矢印でなければ None。
fn arrow(key: &str) -> Option<String> {
    let (pre, base) = key.rsplit_once('+')?;
    let sym = match base {
        "Up" => "↑",
        "Down" => "↓",
        "Left" => "←",
        "Right" => "→",
        _ => return None,
    };
    Some(format!("{pre}+{sym}"))
}

/// 動作の最初のキーの見せ方(表から引く)。
pub fn key_for(table: &[Binding], mode: Mode, action: Action) -> Option<String> {
    table
        .iter()
        .find(|b| b.mode == mode && b.action == action)
        .map(|b| display(b.key))
}

/// 動作の全部のキーの見せ方(ヘルプ・パレット)。表の順。
pub fn keys_of(table: &[Binding], mode: Mode, action: Action) -> Vec<String> {
    table
        .iter()
        .filter(|b| b.mode == mode && b.action == action)
        .map(|b| display(b.key))
        .collect()
}

/// 下の帯に出す「今押せるキー」(SR-1): 順位の順に、動作ごとに表の最初のキーと表示名。
/// 全部のキーはヘルプ(SR-5)に出す。
pub fn hints(table: &[Binding], mode: Mode) -> Vec<String> {
    let mut shown: Vec<(u8, Action, String)> = Vec::new();
    for b in table.iter().filter(|b| b.mode == mode && b.rank > 0) {
        if !shown.iter().any(|(_, a, _)| *a == b.action) {
            shown.push((b.rank, b.action, format!("{} {}", display(b.key), b.text())));
        }
    }
    shown.sort_by_key(|(r, _, _)| *r);
    shown.into_iter().map(|(_, _, s)| s).collect()
}

/// 前置きのキー `prefix` のあとに続けて押せるキーと表示名(SR-25)。表の順に、動作ごとに最初の並び。
/// 続きのキーは前置きを除いた残り(`g g` の前置き `g` なら `g`)。
pub fn continuations(table: &[Binding], mode: Mode, prefix: &str) -> Vec<(Action, String)> {
    let mut seen: Vec<Action> = Vec::new();
    let mut out = Vec::new();
    for b in table.iter().filter(|b| b.mode == mode) {
        let Some(rest) = b.key.strip_prefix(prefix).and_then(|r| r.strip_prefix(' ')) else {
            continue;
        };
        if seen.contains(&b.action) {
            continue;
        }
        seen.push(b.action);
        out.push((b.action, format!("{} {}", display(rest), b.text())));
    }
    out
}

/// 設定のキーの表記(`j`・`ctrl+s`・`shift+tab`・`g g`)を、表のキーの名前(`key_name` の形)に直す(SR-13)。
/// 修飾は `ctrl`・`alt`・`shift`(大文字小文字は問わない)、名前のキーは `enter`・`esc`・`tab`・`space`・
/// `backspace`・`delete`・`up`・`down`・`left`・`right`・`home`・`end`・`pageup`・`pagedown`。
/// 空白で区切ると前置きのキーの並び。読めなければ理由。
pub fn parse_key(text: &str) -> Result<String, String> {
    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.is_empty() {
        return Err(Msg::KeyErrEmpty.text().into());
    }
    let keys: Result<Vec<String>, String> = parts.into_iter().map(parse_one_key).collect();
    Ok(keys?.join(" "))
}

fn parse_one_key(tok: &str) -> Result<String, String> {
    // `+` そのもの、`ctrl++` の末尾の `+` はキー。
    let (mods, base) = if tok == "+" {
        ("", "+")
    } else if let Some(m) = tok.strip_suffix("++") {
        (m, "+")
    } else {
        match tok.rfind('+') {
            Some(i) => (&tok[..i], &tok[i + 1..]),
            None => ("", tok),
        }
    };
    let (mut ctrl, mut alt, mut shift) = (false, false, false);
    for m in mods.split('+').filter(|m| !m.is_empty()) {
        match m.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => ctrl = true,
            "alt" | "meta" => alt = true,
            "shift" => shift = true,
            _ => return Err(Msg::KeyErrModifier.fill(&[&m])),
        }
    }
    let mut pre = String::new();
    if ctrl {
        pre.push_str("Ctrl+");
    }
    if alt {
        pre.push_str("Alt+");
    }
    let mut chars = base.chars();
    if let (Some(c), None) = (chars.next(), chars.clone().next()) {
        let c = halfwidth(c);
        if c == ' ' {
            return Err(Msg::KeyErrSpace.text().into());
        }
        if ctrl || alt {
            if shift || c.is_ascii_uppercase() {
                pre.push_str("Shift+");
            }
            return Ok(format!("{pre}{}", c.to_ascii_lowercase()));
        }
        if shift {
            if c.is_ascii_alphabetic() {
                return Ok(c.to_ascii_uppercase().to_string());
            }
            return Err(Msg::KeyErrShiftChar.fill(&[&c]));
        }
        return Ok(c.to_string());
    }
    let name = match base.to_ascii_lowercase().as_str() {
        "enter" | "return" => "Enter",
        "esc" | "escape" => "Esc",
        "tab" => "Tab",
        "backtab" => {
            shift = true;
            "Tab"
        }
        "space" | "spc" => "Space",
        "backspace" | "bs" => "Backspace",
        "delete" | "del" => "Delete",
        "up" => "Up",
        "down" => "Down",
        "left" => "Left",
        "right" => "Right",
        "home" => "Home",
        "end" => "End",
        "pageup" | "pgup" => "PageUp",
        "pagedown" | "pgdn" => "PageDown",
        "" => return Err(Msg::KeyErrNoKey.fill(&[&tok])),
        _ => return Err(Msg::KeyErrUnknown.fill(&[&base])),
    };
    if name == "Space" {
        if ctrl || alt || shift {
            return Err(Msg::KeyErrSpaceModifier.text().into());
        }
        return Ok("Space".into());
    }
    if shift {
        if !matches!(name, "Tab" | "Up" | "Down" | "Left" | "Right") {
            return Err(Msg::KeyErrShiftSame.fill(&[&base.to_ascii_lowercase()]));
        }
        pre.push_str("Shift+");
    }
    Ok(format!("{pre}{name}"))
}

/// 設定のキーの名前を表の `&'static str` にする。既定の表にある名前はそれを使い、無ければ1回だけ確保する
/// (起動のときに設定の行の数だけ。表は App が終わるまで使う)。
fn intern(key: String) -> &'static str {
    match BINDINGS.iter().find(|b| b.key == key) {
        Some(b) => b.key,
        None => Box::leak(key.into_boxed_str()),
    }
}

/// 各モードの抜ける動作(閉じ込められないために、どのモードにも、どれか1つの動作に1本はキーを残す)。
/// 表は終了か、パレット(`q` で終了できる)のどちらか。
const EXITS: &[(Mode, &[Action])] = &[
    (Mode::Table, &[Action::Quit, Action::Palette]),
    (Mode::Help, &[Action::Close]),
    (Mode::Palette, &[Action::Close]),
    (Mode::Detail, &[Action::Close]),
    (Mode::Confirm, &[Action::Back]),
    (Mode::Quit, &[Action::Back]),
    (Mode::Edit, &[Action::Cancel]),
    (Mode::Search, &[Action::Cancel]),
    (Mode::Filter, &[Action::Cancel]),
    (Mode::Settings, &[Action::Cancel]),
    (Mode::SettingsText, &[Action::Cancel]),
    (Mode::Chips, &[Action::Close]),
    (Mode::ListPick, &[Action::Cancel]),
    (Mode::Menu, &[Action::Close]),
    (Mode::Freq, &[Action::Close]),
    (Mode::Sorts, &[Action::Close]),
    (Mode::Relations, &[Action::Close]),
];

/// 割り当て直しで抜ける動作のキーが0本になったら、その動作の既定のキーを戻して警告する
/// (外す・置き換える設定を当てない)。戻すキーに当てた別の動作と、戻すキーを前置きにする並びは外す。
fn keep_exits(table: &mut Vec<Binding>, warnings: &mut Vec<String>) {
    for &(m, actions) in EXITS {
        if table
            .iter()
            .any(|b| b.mode == m && actions.contains(&b.action))
        {
            continue;
        }
        let mut back = Vec::new();
        for d in BINDINGS
            .iter()
            .filter(|b| b.mode == m && actions.contains(&b.action))
        {
            table.retain(|b| {
                !(b.mode == m
                    && (b.key == d.key
                        || b.key
                            .strip_prefix(d.key)
                            .is_some_and(|r| r.starts_with(' '))))
            });
            table.push(d.clone());
            back.push(format!("`{}`", display(d.key)));
        }
        let sep = Msg::ListSep.text();
        let names = actions
            .iter()
            .map(|a| a.name())
            .collect::<Vec<_>>()
            .join(&format!("`{sep}`"));
        warnings.push(Msg::RebindExitKept.fill(&[&m.label(), &names, &back.join(sep)]));
    }
}

/// 設定の `(モード, キーの表記, 動作の名前か "none")` で表を上書きする(SR-13)。
/// 知らないモード・キー・動作、そのモードで使えない動作、同じモードで重なる割り当て(SR-16)は
/// 警告の文にして当てない。`"none"` を先に当て(外してから足せる)、残りを当てる。
/// 同じキーに別の動作を書けば、そのキーの既定の動作を置き換える。各モードの抜ける動作(`EXITS`)の
/// キーが0本になる設定は当てず、既定のキーを残して警告する。設定の中で同じモードの同じキー
/// (表記の違う `ctrl+n` と `control+n` など)に2つ以上の動作を書いたら、どれも当てない(TOML の表は順を持たない)。
pub fn rebind(table: &mut Vec<Binding>, binds: &[(String, String, String)]) -> Vec<String> {
    let mut warnings = Vec::new();
    let mut parsed: Vec<(Mode, String, &str, String)> = Vec::new();
    for (mode, key, action) in binds {
        let at = format!("keys.{mode}.{key}");
        let Some(m) = Mode::by_name(mode) else {
            let names: Vec<&str> = Mode::ALL.iter().map(|m| m.name()).collect();
            warnings.push(Msg::RebindUnknownMode.fill(&[mode, &names.join(", ")]));
            continue;
        };
        let k = match parse_key(key) {
            Ok(k) => k,
            Err(e) => {
                warnings.push(Msg::RebindBadKey.fill(&[&at, &e]));
                continue;
            }
        };
        if m.takes_text() && k.contains(' ') {
            warnings.push(Msg::RebindNoPrefix.fill(&[&at, &m.label()]));
            continue;
        }
        parsed.push((m, k, action.as_str(), at));
    }
    // 外す
    for (m, k, a, _) in &parsed {
        if *a == "none" {
            table.retain(|b| !(b.mode == *m && b.key == k));
        }
    }
    // 足す・置き換える
    let assigns: Vec<(Mode, String, &str, String)> =
        parsed.into_iter().filter(|p| p.2 != "none").collect();
    let mut warned: Vec<(Mode, String)> = Vec::new();
    for (m, k, a, at) in &assigns {
        let (m, k, a) = (*m, k.clone(), *a);
        let Some(action) = Action::by_name(a) else {
            warnings.push(Msg::RebindUnknownAction.fill(&[at, &a]));
            continue;
        };
        let Some(def) = BINDINGS.iter().find(|b| b.mode == m && b.action == action) else {
            warnings.push(Msg::RebindNotInMode.fill(&[at, &a, &m.label()]));
            continue;
        };
        let same: Vec<&str> = assigns
            .iter()
            .filter(|o| o.0 == m && o.1 == k)
            .map(|o| o.3.as_str())
            .collect();
        if same.len() > 1 {
            if !warned.iter().any(|(wm, wk)| *wm == m && *wk == k) {
                warned.push((m, k.clone()));
                let places = same
                    .iter()
                    .map(|s| format!("`{s}`"))
                    .collect::<Vec<_>>()
                    .join(Msg::ListSep.text());
                warnings.push(Msg::RebindSameKey.fill(&[&places, &m.label(), &display(&k)]));
            }
            continue;
        }
        // 前置きとぶつかる(`g` と `g g`): 1つのキーが動作と前置きを兼ねるので当てない(SR-16)。
        let clash = table.iter().find(|b| {
            b.mode == m
                && b.key != k
                && (b
                    .key
                    .strip_prefix(k.as_str())
                    .is_some_and(|r| r.starts_with(' '))
                    || k.strip_prefix(b.key).is_some_and(|r| r.starts_with(' ')))
        });
        if let Some(c) = clash {
            warnings.push(Msg::RebindPrefixClash.fill(&[
                at,
                &m.label(),
                &display(c.key),
                &c.action.name(),
            ]));
            continue;
        }
        let nb = Binding {
            mode: m,
            key: intern(k),
            action,
            label: def.label,
            section: def.section,
            rank: def.rank,
            msg: def.msg,
            section_msg: def.section_msg,
        };
        match table.iter_mut().find(|b| b.mode == m && b.key == nb.key) {
            Some(b) => *b = nb,
            None => table.push(nb),
        }
    }
    keep_exits(table, &mut warnings);
    warnings
}

#[cfg(test)]
#[path = "test_keymap.rs"]
mod tests;
