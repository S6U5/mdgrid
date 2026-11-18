//! カンバンのレビュー3回目の指摘(#14〜#17)の直しの試験。記録: specs/_history/decidespec/_changes/2026-11-02-kanban.md の
//! 「レビューの指摘」と「実装の気づき」の「3回目の直し:」。値は作ったもの。錠のある試験のファイルは変えず、補助は
//! test_kanban_fix2.rs から写した。
//!
//! 直す前の版で落ち、直した後の版で通る期待にする。実装の前の版(4bc740c)に無い名前は使わず、キーとパレットを
//! 送り、画面の文字と今ある口(`mode`・`message`・`changes`・`poll`・`folder_tree` の欄)で見る。

use super::app::{App, ColorMode};
use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, col_named, ctrl, edit_outside, press, text, typing, Tmp};
use mdgrid::config::Config;
use mdgrid::source::markdown::Markdown;
use mdgrid::source::RowId;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;
use std::path::PathBuf;

/// 状態の列の決まり(CE-36)。レーンの順はこの順。
const RULES: &str = "[rules.status]\nvalues = [\"todo\", \"doing\", \"waiting\", \"done\"]\n";

/// 印の [[table]] のカンバンの設定。
const KANBAN: &str = "kanban = { column = \"status\" }";

/// 印の [[table]] のガントの設定。
const GANTT: &str = "gantt = { start = \"start\", end = \"due\" }";

/// 印の [[table]] の木の設定。
const TREE: &str = "tree = { path = \"folder\" }";

/// 名前の順に alpha・beta・delta・epsilon・gamma。どれも状態と日付を持つ。
const NOTES: &[(&str, &str)] = &[
    (
        "alpha.md",
        "---\nstatus: todo\nstart: 2026-10-01\ndue: 2026-10-03\n---\n",
    ),
    (
        "beta.md",
        "---\nstatus: todo\nstart: 2026-10-02\ndue: 2026-10-04\n---\n",
    ),
    (
        "gamma.md",
        "---\nstatus: doing\nstart: 2026-10-03\ndue: 2026-10-05\n---\n",
    ),
    (
        "delta.md",
        "---\nstatus: done\nstart: 2026-10-04\ndue: 2026-10-06\n---\n",
    ),
    (
        "epsilon.md",
        "---\nstatus: done\nstart: 2026-10-05\ndue: 2026-10-07\n---\n",
    ),
];

/// 縦の線(レーンの間と枠の縦の辺)。
const SEPS: [char; 3] = ['│', '┃', '|'];

/// レーンの見出しの (区画の番号, 値, 数)。
type HeadCell = (usize, String, usize);

/// 印(根 = 一時フォルダ)の先頭に `top`、[[table]] に `line` を書いて `notes` を表として開き、読み終える。
fn boot(tag: &str, notes: &[(&str, &str)], top: &str, line: &str) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    for (n, t) in notes {
        tmp.write(n, t);
    }
    std::fs::create_dir_all(tmp.0.join(".mdgrid")).unwrap();
    std::fs::write(
        tmp.0.join(".mdgrid/workspace.toml"),
        format!("{top}\n[[table]]\nname = \"T\"\npath = \"notes\"\n{line}\n"),
    )
    .unwrap();
    let dir: PathBuf = tmp.notes();
    let src = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(120, 30);
    app.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: dir,
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    (tmp, app)
}

/// App の大きさで描いた画面の文字。
fn screen(a: &App) -> String {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    text(t.backend().buffer())
}

/// 見出しの区画の端の飾り(空白と、隠れたレーンの印と、選んでいるレーンの `>`)。
fn trim_head(t: &str) -> &str {
    t.trim_matches(|c: char| c.is_whitespace() || c == '‹' || c == '›' || c == '>')
}

/// 見出しの区画の文字から (値, 数)。`値 (数)` と `値(数)` のどちらでも。
fn parse_head(seg: &str) -> Option<(String, usize)> {
    let t = trim_head(seg).strip_suffix(')')?;
    let i = t.rfind('(')?;
    let n: usize = t[i + 1..].parse().ok()?;
    let v = t[..i].trim_end();
    (!v.is_empty()).then(|| (v.to_string(), n))
}

/// レーンの見出しの行(区画の中身がどれも「値 (数)」で2つ以上並ぶ最初の行)の見出し。
fn heading_row(s: &str) -> Option<Vec<HeadCell>> {
    for line in s.lines() {
        let filled: Vec<(usize, String)> = line
            .split(&SEPS[..])
            .map(str::to_string)
            .enumerate()
            .filter(|(_, t)| !trim_head(t).is_empty())
            .collect();
        if filled.len() < 2 {
            continue;
        }
        let heads: Option<Vec<HeadCell>> = filled
            .iter()
            .map(|(k, t)| parse_head(t).map(|(v, n)| (*k, v, n)))
            .collect();
        if heads.is_some() {
            return heads;
        }
    }
    None
}

/// 見えているレーンの (値, 数)。左から。
fn heads(s: &str) -> Vec<(String, usize)> {
    heading_row(s)
        .unwrap_or_else(|| panic!("レーンの見出しの行が無い:\n{s}"))
        .into_iter()
        .map(|(_, v, n)| (v, n))
        .collect()
}

/// メッセージ行(空でない最後の行。描画は高さより1行少なく描くので一番下の行はいつも空)より上の画面の文字。
fn above_message(s: &str) -> String {
    let lines: Vec<&str> = s.lines().collect();
    let last = lines
        .iter()
        .rposition(|l| !l.trim().is_empty())
        .unwrap_or(0);
    lines[..last].join("\n")
}

/// 行のノートの名前(拡張子なし)。
fn lab(a: &App, r: &RowId) -> String {
    a.src.label(r).trim_end_matches(".md").to_string()
}

/// 表で選んでいる行の名前。
fn cur(a: &App) -> String {
    lab(a, &a.cur_row().expect("表の行"))
}

/// 名前の行。
fn row_of(a: &App, name: &str) -> RowId {
    a.rows
        .iter()
        .find(|r| lab(a, r) == name)
        .cloned()
        .unwrap_or_else(|| panic!("行 {name} が無い"))
}

/// `B` でカンバンを開き、開いた画面の文字を返す(開かなければ落ちる)。
fn open(a: &mut App) -> String {
    ch(a, 'B');
    let s = screen(a);
    assert_ne!(
        a.mode,
        Mode::Table,
        "B でカンバンが開かない: {:?}\n{s}",
        a.message
    );
    let body = above_message(&s);
    assert!(
        body.contains("カンバン") && heading_row(&body).is_some(),
        "カンバンの題とレーンの見出し:\n{s}"
    );
    s
}

/// パレットで打って選ぶ。
fn palette(a: &mut App, query: &str) {
    ch(a, ':');
    assert_eq!(a.mode, Mode::Palette, "{:?}", a.message);
    typing(a, query);
    press(a, KeyCode::Enter);
}

/// 見本なしの空の台帳を読み終え、外で設定の列の無いノート(first.md)を書いて poll する。
fn empty_then_first_note(tag: &str, line: &str) -> (Tmp, App) {
    let (t, mut a) = boot(tag, &[], RULES, line);
    assert!(a.rows.is_empty(), "前提: 0件");
    t.write("first.md", "---\ntitle: x\n---\n");
    a.poll();
    assert_eq!(a.rows.len(), 1, "前提: 足したノートが行になる");
    (t, a)
}

/// `empty_then_first_note` のあと、外で first.md に `keys` を足して poll する(設定の列ができる)。
fn then_add_columns(t: &Tmp, a: &mut App, keys: &str, col: &str) {
    edit_outside(t, "first.md", &format!("---\ntitle: x\n{keys}---\n"));
    a.poll();
    assert!(
        a.src.columns().iter().any(|c| c == col),
        "前提: 列 {col} ができた"
    );
}

/// 設定はあって列が無いあいだに `key` を押したときの理由(表のまま)。
fn reason_for_missing_column(a: &mut App, key: char) -> String {
    a.message = None;
    ch(a, key);
    assert_eq!(a.mode, Mode::Table, "列が無いので開かない: {:?}", a.message);
    let m = a.message.clone().unwrap_or_default();
    assert!(!m.trim().is_empty() && !m.contains('\n'), "理由1行: {m:?}");
    m
}

// ---- #14: 最初のノートに列が無く、あとで列ができる ----

#[test]
fn test_kb_1_column_added_after_first_note_opens_kanban() {
    // [utest -> req~kanban.config-bad-column~1, req~kanban.config-empty-table~1] #14: 空の台帳に status の無いノートが最初にでき、あとで外で status を足す → B でカンバンが開く
    // (列が表に無いあいだだけ出さない。最初の1件で設定を外したままにしない)。
    let (t, mut a) = empty_then_first_note("kbfx14", KANBAN);
    then_add_columns(&t, &mut a, "status: todo\n", "status");
    let s = open(&mut a);
    assert!(heads(&s).contains(&("todo".to_string(), 1)), "{s}");
}

#[test]
fn test_ga_1_column_added_after_first_note_opens_gantt() {
    // [utest -> req~gantt.config-days~1, req~gantt.config-empty-table~1] #14: 空の台帳に start・due の無いノートが最初にでき、あとで外で足す → C でガントが開く。
    let (t, mut a) = empty_then_first_note("gafx14", GANTT);
    then_add_columns(&t, &mut a, "start: 2026-10-01\ndue: 2026-10-03\n", "start");
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "ガントが開く: {:?}", a.message);
}

#[test]
fn test_tr_2_column_added_after_first_note_opens_tree() {
    // [utest -> req~tree.config-empty-table~1] #14: 空の台帳に folder の無いノートが最初にでき、あとで外で足す → T で木が開く。
    let (t, mut a) = empty_then_first_note("trfx14", TREE);
    then_add_columns(&t, &mut a, "folder: x > y\n", "folder");
    ch(&mut a, 'T');
    assert_eq!(a.mode, Mode::Tree, "木が開く: {:?}", a.message);
}

#[test]
fn test_kb_1_b_names_the_missing_column_not_no_setting() {
    // [utest -> req~kanban.config-guess~1, req~kanban.config-guess-none~1, req~kanban.config-empty-table~1] #14: 設定はあって列が無いあいだ(空の台帳に status の無いノートができた)の B の理由は、
    // 「設定が無い」でなく、無い列の名前 status を言う。
    let (_t, mut a) = empty_then_first_note("kbfx14r", KANBAN);
    let m = reason_for_missing_column(&mut a, 'B');
    assert!(m.contains("status") && !m.contains("設定が無"), "{m}");
}

#[test]
fn test_kb_1_b_names_the_missing_column_after_loading() {
    // [utest -> req~kanban.config-guess~1, req~kanban.config-empty-table~1] #14: 行のある表で設定の列が無い(column = "none")とき、読み終えたあとの B の理由も、
    // 「設定が無い」でなく、無い列の名前 none を言う。
    let (_t, mut a) = boot("kbfx14n", NOTES, RULES, "kanban = { column = \"none\" }");
    let m = reason_for_missing_column(&mut a, 'B');
    assert!(m.contains("none") && !m.contains("設定が無"), "{m}");
}

#[test]
fn test_ga_1_c_names_the_missing_column_not_no_setting() {
    // [utest -> req~gantt.config-infer~1] #14: 設定はあって列が無いあいだの C の理由は、「設定が無い」でなく、無い列の名前 start を言う。
    let (_t, mut a) = empty_then_first_note("gafx14r", GANTT);
    let m = reason_for_missing_column(&mut a, 'C');
    assert!(m.contains("start") && !m.contains("設定が無"), "{m}");
}

#[test]
fn test_tr_2_t_names_the_missing_column_not_no_setting() {
    // [utest -> req~tree.config-empty-table~1] #14: 設定はあって列が無いあいだの T の理由は、「設定が無い」でなく、無い列の名前 folder を言う。
    let (_t, mut a) = empty_then_first_note("trfx14r", TREE);
    let m = reason_for_missing_column(&mut a, 'T');
    assert!(m.contains("folder") && !m.contains("設定が無"), "{m}");
}

// ---- #15: 「+ 新規」で最初のノートを作ったときの警告 ----

#[test]
fn test_kb_1_first_new_note_reports_created_and_missing_column() {
    // [utest -> req~kanban.config-empty-table~1] #15: 見本なしの空の台帳で「+ 新規」(`a`)から最初のノートを作る(status を書かない)→ 作ったあとの
    // メッセージに、「作った」の知らせと一緒に、列 status が無い警告(CLI-3 の形)が出る。
    let (t, mut a) = boot("kbfx15", &[], RULES, KANBAN);
    assert!(a.rows.is_empty(), "前提: 0件");
    ch(&mut a, 'a');
    assert_ne!(a.mode, Mode::Table, "前提: 名前の欄が開く: {:?}", a.message);
    typing(&mut a, "first");
    ctrl(&mut a, 's');
    assert!(t.notes().join("first.md").is_file(), "前提: 作った");
    assert_eq!(a.rows.len(), 1, "前提: 作ったノートが行になる");
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains("作った"), "作ったの知らせ: {m}");
    assert!(
        m.contains("table[1].kanban") && m.contains("status"),
        "列の警告も出る: {m}"
    );
}

// ---- #16: 木の右の欄で選んでいる行に、パレットの動作が効く ----

#[test]
fn test_tr_4_palette_action_applies_to_row_selected_in_tree() {
    // [utest -> req~tree.rows-enter~1] #16: 表で alpha の status を選んでから木を開き、右の欄で beta を選んでパレットから clear →
    // 表に戻って beta の status を空にする(木を開く前の表の行 alpha には効かない)。
    let notes: &[(&str, &str)] = &[
        ("alpha.md", "---\nfolder: x\nstatus: todo\n---\n"),
        ("beta.md", "---\nfolder: x\nstatus: doing\n---\n"),
        ("gamma.md", "---\nfolder: y\nstatus: done\n---\n"),
    ];
    let (_t, mut a) = boot("trfx16", notes, "", TREE);
    col_named(&mut a, "status");
    assert_eq!(cur(&a), "alpha", "前提: 表の今の行");
    ch(&mut a, 'T');
    assert_eq!(a.mode, Mode::Tree, "{:?}", a.message);
    let tree_row = |a: &App| {
        let ft = a.folder_tree.as_ref().expect("木");
        ft.row().map(|r| lab(a, r))
    };
    // 右の欄へ移り、beta を選ぶ。
    ch(&mut a, 'l');
    for _ in 0..4 {
        if tree_row(&a).as_deref() == Some("beta") {
            break;
        }
        ch(&mut a, 'j');
    }
    assert_eq!(
        tree_row(&a).as_deref(),
        Some("beta"),
        "前提: 右の欄で beta を選ぶ"
    );
    palette(&mut a, "clear");
    let (alpha, beta) = (row_of(&a, "alpha"), row_of(&a, "beta"));
    assert_eq!(
        a.changes.pending(&alpha, "status"),
        None,
        "表の別の行(alpha)に効かない: {:?}",
        a.message
    );
    assert!(
        a.changes.pending(&beta, "status").is_some(),
        "木で選んでいる beta に効く: {:?}",
        a.message
    );
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(cur(&a), "beta");
}

// ---- #17: パレットの `:<番号>` ----

#[test]
fn test_nv_7_palette_line_number_from_kanban_selects_table_row() {
    // [utest -> req~navigation.jump~1] #17: カンバンでパレットの `:3` → 表の画面に戻って3行目(delta)が選ばれる。
    let (_t, mut a) = boot("kbfx17", NOTES, RULES, KANBAN);
    open(&mut a);
    palette(&mut a, "3");
    assert_eq!(
        a.mode,
        Mode::Table,
        "表に戻る: {:?}\n{}",
        a.message,
        screen(&a)
    );
    assert_eq!(cur(&a), "delta");
}

#[test]
fn test_nv_7_palette_line_number_from_gantt_selects_table_row() {
    // [utest -> req~navigation.jump~1] #17: ガントでパレットの `:2` → 表の画面に戻って2行目(beta)が選ばれる。
    let (_t, mut a) = boot("gafx17", NOTES, "", GANTT);
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    palette(&mut a, "2");
    assert_eq!(
        a.mode,
        Mode::Table,
        "表に戻る: {:?}\n{}",
        a.message,
        screen(&a)
    );
    assert_eq!(cur(&a), "beta");
}
