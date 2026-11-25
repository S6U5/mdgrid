//! 値の名前と字(`[rules.<列>] labels`・`icons`)を、値を見せる所(表のセル・詳細・木の画面の右の欄)で「字 名前」で
//! 見せる(navigation.value-labels-cells)試験と、値を選ぶ窓(リストの値の付け外し・絞り込みの窓・ビューの設定の
//! 値の一覧・値の件数の窓)に名前と値を並べる(navigation.value-labels-picker)試験。値は作ったもの。
//!
//! 期待値は要求の受け入れの例から取った。見せ方は描いた画面の文字で、書く値はためた値とノートのファイルで確かめる。

use super::app::{App, ColorMode};
use super::keymap::{Action, Mode};
use super::startup::Startup;
use super::test_screen::{ch, col_named, ctrl, press, text, Tmp};
use mdgrid::config::Config;
use mdgrid::source::markdown::Markdown;
use mdgrid::writeback::NewValue;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;
use std::path::PathBuf;

/// 受け入れの例の設定: 候補 todo・doing・done、todo に字 ○、todo と doing に名前。
const STATUS: &str = "[rules.status]\nvalues = [\"todo\", \"doing\", \"done\"]\nicons = { todo = \"○\" }\nlabels = { todo = \"未着手\", doing = \"進行中\" }\n";

/// status が todo・doing・done・blocked の行。
const NOTES: &[(&str, &str)] = &[
    ("alpha.md", "---\nstatus: todo\n---\n"),
    ("beta.md", "---\nstatus: doing\n---\n"),
    ("gamma.md", "---\nstatus: done\n---\n"),
    ("delta.md", "---\nstatus: blocked\n---\n"),
];

/// 印(根 = 一時フォルダ)の先頭に `top`、[[table]] に `line` を書いて、`notes` を表として開く。
fn boot(tag: &str, notes: &[(&str, &str)], top: &str, line: &str, color: ColorMode) -> (Tmp, App) {
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
    let mut a = App::new(Box::new(src), color);
    a.resize(120, 30);
    a.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: color == ColorMode::None,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: dir,
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    (tmp, a)
}

/// App の大きさで描いたバッファ。
fn buffer(a: &App) -> ratatui::buffer::Buffer {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    t.backend().buffer().clone()
}

fn screen(a: &App) -> String {
    text(&buffer(a))
}

/// 画面の中で `name` を含む最初の行。
fn line_with(s: &str, name: &str) -> String {
    s.lines()
        .find(|l| l.contains(name))
        .unwrap_or_else(|| panic!("「{name}」の行が無い:\n{s}"))
        .to_string()
}

/// ノートの名前 `name` の行を選ぶ。
fn select_row(a: &mut App, name: &str) {
    let i = (0..a.slots.len())
        .find(|&i| {
            matches!(a.slots[i], super::grid::Slot::Row(k) if a.src.label(&a.rows[k]).contains(name))
        })
        .unwrap_or_else(|| panic!("{name} の行が無い"));
    a.row = i;
}

#[test]
fn test_value_labels_cells_text_cells_show_glyph_and_name() {
    // [utest -> req~navigation.value-labels-cells~1] 色を使わない表示で、status のセルは「○ 未着手」・「進行中」・done・!blocked
    // (values に無い blocked は名前を当てずに、決まりに合わない印)。
    let (_t, a) = boot("vlc_text", NOTES, STATUS, "", ColorMode::None);
    let s = screen(&a);
    assert!(line_with(&s, "alpha").contains("○ 未着手"), "{s}");
    assert!(line_with(&s, "beta").contains("進行中"), "{s}");
    let g = line_with(&s, "gamma");
    assert!(g.contains("done"), "{s}");
    let d = line_with(&s, "delta");
    assert!(d.contains("!blocked"), "{s}");
    assert!(!s.contains(" todo") && !s.contains("doing"), "{s}");
}

#[test]
fn test_value_labels_cells_chip_text_is_name_and_color_is_value() {
    // [utest -> req~navigation.value-labels-cells~1] 色のある表示で status の列を札にすると、todo の行の札の文字は「○ 未着手」で、
    // 札の色は値 todo の札の色(名前で色を変えない)。
    let top =
        format!("{STATUS}\n[look.columns]\nstatus = \"chip\"\n\n[look.style]\nstatus = \"chip\"\n");
    let (_t, a) = boot("vlc_chip", NOTES, &top, "", ColorMode::Rgb);
    let b = buffer(&a);
    let s = text(&b);
    let row = line_with(&s, "alpha");
    assert!(row.contains("○ 未着手"), "{s}");
    let y = s.lines().position(|l| l.contains("alpha")).unwrap() as u16;
    let x = (0..b.area.width)
        .find(|&x| b[(x, y)].symbol() == "未")
        .expect("未の字");
    let want = super::chips::style(&a, "todo").and_then(|st| st.bg);
    assert!(want.is_some(), "前提: 札の色");
    assert_eq!(Some(b[(x, y)].bg), want, "札の地は値 todo の色");
}

#[test]
fn test_value_labels_cells_list_elements() {
    // [utest -> req~navigation.value-labels-cells~1] values = [a, b]・labels = { a = 甲 } の tags が [a, b] の行(色なし)→ tags のセルは [甲, b]。
    let top = "[types]\ntags = \"list\"\n\n[rules.tags]\nvalues = [\"a\", \"b\"]\nlabels = { a = \"甲\" }\n";
    let notes = [("alpha.md", "---\ntags: [a, b]\n---\n")];
    let (_t, a) = boot("vlc_list", &notes, top, "", ColorMode::None);
    let s = screen(&a);
    assert!(line_with(&s, "alpha").contains("[甲, b]"), "{s}");
}

#[test]
fn test_value_labels_cells_number_values() {
    // [utest -> req~navigation.value-labels-cells~1] rating を数の型にし、values = [1, 2, 3]・labels = { 3 = ★★★ } → rating が 3 のセルは「★★★」。
    let top = "[types]\nrating = \"number\"\n\n[rules.rating]\nvalues = [\"1\", \"2\", \"3\"]\nlabels = { \"3\" = \"★★★\" }\n";
    let notes = [
        ("alpha.md", "---\nrating: 3\n---\n"),
        ("beta.md", "---\nrating: 1\n---\n"),
    ];
    let (_t, a) = boot("vlc_num", &notes, top, "", ColorMode::None);
    let s = screen(&a);
    assert!(line_with(&s, "alpha").contains("★★★"), "{s}");
    assert!(line_with(&s, "beta").contains('1'), "{s}");
}

#[test]
fn test_value_labels_cells_detail_shows_name() {
    // [utest -> req~navigation.value-labels-cells~1] todo の行の詳細の表示(K)で、status の値は「○ 未着手」。
    let (_t, mut a) = boot("vlc_detail", NOTES, STATUS, "", ColorMode::None);
    select_row(&mut a, "alpha");
    ch(&mut a, 'K');
    assert_eq!(a.mode, Mode::Detail, "{:?}", a.message);
    let s = screen(&a);
    let l = line_with(&s, "status");
    assert!(l.contains("○ 未着手"), "{s}");
    assert!(!l.contains("todo"), "{s}");
}

#[test]
fn test_value_labels_cells_tree_right_pane() {
    // [utest -> req~navigation.value-labels-cells~1] 右の欄に status の列を出す木の画面で、todo の行の status は「○ 未着手」。
    let notes = [
        ("alpha.md", "---\nstatus: todo\nshelf: 仕事\n---\n"),
        ("beta.md", "---\nstatus: doing\nshelf: 仕事\n---\n"),
    ];
    let (_t, mut a) = boot(
        "vlc_tree",
        &notes,
        STATUS,
        "tree = { path = \"shelf\" }",
        ColorMode::None,
    );
    ch(&mut a, 'T');
    assert_eq!(a.mode, Mode::Tree, "木が開かない: {:?}", a.message);
    // 右の欄(フォルダの中の並び)へ移る。
    press(&mut a, KeyCode::Tab);
    let s = screen(&a);
    let l = line_with(&s, "alpha");
    assert!(l.contains("○ 未着手"), "{s}");
    assert!(line_with(&s, "beta").contains("進行中"), "{s}");
}

#[test]
fn test_value_labels_cells_pending_value_and_written_value() {
    // [utest -> req~navigation.value-labels-cells~1] todo の行の status のセルを候補「進行中 doing」で直すと、セルは *進行中 で、
    // 保存するとノートには status: doing と書く。
    let (t, mut a) = boot("vlc_pending", NOTES, STATUS, "", ColorMode::None);
    select_row(&mut a, "alpha");
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Enter);
    let items: Vec<String> = a
        .input
        .as_ref()
        .and_then(|i| i.list.as_ref())
        .map(|l| l.items.iter().map(|i| i.text.clone()).collect())
        .unwrap_or_default();
    let at = items
        .iter()
        .position(|x| x == "進行中 doing")
        .unwrap_or_else(|| panic!("候補に「進行中 doing」が無い: {items:?}"));
    let sel = a.input.as_ref().unwrap().list.as_ref().unwrap().sel;
    for _ in sel..at {
        press(&mut a, KeyCode::Down);
    }
    for _ in at..sel {
        press(&mut a, KeyCode::Up);
    }
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table, "{:?}", a.message);
    let s = screen(&a);
    assert!(line_with(&s, "alpha").contains("*進行中"), "{s}");
    ctrl(&mut a, 's');
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.changes.count(), 0, "{:?}", a.message);
    let disk = std::fs::read_to_string(t.notes().join("alpha.md")).unwrap();
    assert!(disk.contains("status: doing"), "{disk}");
}

#[test]
fn test_value_labels_cells_kanban_move_message() {
    // [utest -> req~navigation.value-labels-cells~1] カンバンで todo の札を doing のレーンへ動かすと、知らせは「○ 未着手 → 進行中」の形で、
    // ためた値は doing。
    let notes = [
        ("alpha.md", "---\nstatus: todo\n---\n"),
        ("beta.md", "---\nstatus: doing\n---\n"),
    ];
    let (_t, mut a) = boot(
        "vlc_kanban",
        &notes,
        STATUS,
        "kanban = { column = \"status\" }",
        ColorMode::None,
    );
    ch(&mut a, 'B');
    assert_eq!(a.mode, Mode::Kanban, "{:?}", a.message);
    let row = a
        .rows
        .iter()
        .find(|r| a.src.label(r).contains("alpha"))
        .unwrap()
        .clone();
    // 札は todo のレーン(左端)の alpha。右のレーンへ動かす。
    a.kanban_action(Action::MoveCardRight);
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains("○ 未着手 → 進行中"), "{m}");
    assert_eq!(
        a.changes.pending(&row, "status"),
        Some(&NewValue::Str("doing".into()))
    );
}

#[test]
fn test_value_labels_quick_filter_and_search_match_name() {
    // [no-req: 未確定事項 tbd~navigation.value-labels-match の仮決めを確かめる(人が決めたら要求に書いて印を付ける)]
    // 仮決め: `\` の簡易の絞り込みと `/` の検索は、値でも見せる名前でも当たる。
    let (_t, mut a) = boot("vlc_match", NOTES, STATUS, "", ColorMode::None);
    a.filter = Some("未着手".into());
    a.refresh();
    let names: Vec<String> = a.rows.iter().map(|r| a.src.label(r)).collect();
    assert_eq!(names.len(), 1, "{names:?}");
    assert!(names[0].contains("alpha"), "{names:?}");
    a.filter = Some("todo".into());
    a.refresh();
    assert_eq!(a.rows.len(), 1, "値でも当たる");
    a.filter = None;
    a.refresh();
    a.search = Some("進行中".into());
    let row = a
        .rows
        .iter()
        .find(|r| a.src.label(r).contains("beta"))
        .unwrap()
        .clone();
    assert!(a.search_hit(&row, Some("status")));
}

// ---- 値を選ぶ窓(navigation.value-labels-picker) ----

#[test]
fn test_value_labels_picker_list_toggle_window() {
    // [utest -> req~navigation.value-labels-picker~2] values = [a, b]・labels = { a = 甲 } の tags が [a] の行で付け外しの窓を開くと、
    // 候補の行は「甲 a」と「b」で、「甲 a」の行の印は [x]。
    let top = "[types]\ntags = \"list\"\n\n[rules.tags]\nvalues = [\"a\", \"b\"]\nlabels = { a = \"甲\" }\n";
    let notes = [("alpha.md", "---\ntags: [a]\n---\n")];
    let (_t, mut a) = boot("vlp_list", &notes, top, "", ColorMode::None);
    select_row(&mut a, "alpha");
    col_named(&mut a, "tags");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::ListPick, "{:?}", a.message);
    let s = screen(&a);
    assert!(line_with(&s, "甲 a").contains("[x]"), "{s}");
    assert!(line_with(&s, "[ ] b").contains("[ ]"), "{s}");
}

#[test]
fn test_value_labels_picker_filter_window_values() {
    // [utest -> req~navigation.value-labels-picker~2] 絞り込みの窓で status の値の一覧を出すと、一覧の行は「未着手 todo」と「doing」
    // (この試験の設定は名前が todo だけ)。
    let top = "[rules.status]\nvalues = [\"todo\", \"doing\"]\nlabels = { todo = \"未着手\" }\n";
    let notes = [
        ("alpha.md", "---\nstatus: todo\n---\n"),
        ("beta.md", "---\nstatus: doing\n---\n"),
    ];
    let (_t, mut a) = boot("vlp_filter", &notes, top, "", ColorMode::None);
    a.apply(Action::FilterMenu);
    assert_eq!(a.mode, Mode::FilterWin, "{:?}", a.message);
    // 「+ 絞り込みを足す」→ 列 → 種類(値の一覧)。
    press(&mut a, KeyCode::Enter);
    let i = match &a.filter_win.as_ref().unwrap().step {
        Some(super::filters::Step::Column { cols, .. }) => cols
            .iter()
            .position(|c| c == "status")
            .expect("status の列"),
        other => panic!("列の段でない: {other:?}"),
    };
    for _ in 0..i {
        press(&mut a, KeyCode::Down);
    }
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Enter);
    let s = screen(&a);
    assert!(
        matches!(
            a.filter_win.as_ref().and_then(|w| w.step.as_ref()),
            Some(super::filters::Step::Values { .. })
        ),
        "値の一覧の段でない:\n{s}"
    );
    assert!(s.contains("未着手 todo"), "{s}");
    assert!(line_with(&s, "doing").contains("doing"), "{s}");
}

#[test]
fn test_value_labels_picker_freq_window() {
    // [utest -> req~navigation.value-labels-picker~2] status の列で値の件数の窓を開くと、項目は「未着手 todo」と「doing」で、
    // 「未着手 todo」で Enter を押すと status が todo の行だけが残る。
    let top = "[rules.status]\nvalues = [\"todo\", \"doing\"]\nlabels = { todo = \"未着手\" }\n";
    let notes = [
        ("alpha.md", "---\nstatus: todo\n---\n"),
        ("beta.md", "---\nstatus: doing\n---\n"),
        ("gamma.md", "---\nstatus: doing\n---\n"),
    ];
    let (_t, mut a) = boot("vlp_freq", &notes, top, "", ColorMode::None);
    col_named(&mut a, "status");
    ch(&mut a, '%');
    assert_eq!(a.mode, Mode::Freq, "{:?}", a.message);
    let labels = a.freq.as_ref().unwrap().labels.clone();
    assert!(labels.iter().any(|l| l == "未着手 todo"), "{labels:?}");
    assert!(labels.iter().any(|l| l == "doing"), "{labels:?}");
    let at = labels.iter().position(|l| l == "未着手 todo").unwrap();
    for _ in 0..at {
        press(&mut a, KeyCode::Down);
    }
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.rows.len(), 1, "{:?}", a.message);
    assert!(a.src.label(&a.rows[0]).contains("alpha"));
    assert_eq!(
        a.changes.pending(&a.rows[0].clone(), "status"),
        None::<&NewValue>
    );
}

#[test]
fn test_value_labels_picker_settings_values() {
    // [utest -> req~navigation.value-labels-picker~2] ビューの設定(o)のフィルターで status の値の一覧を出しても、行は「未着手 todo」と「doing」。
    use super::settings::{Pick, Sec};
    use super::test_settings_screen as ts;
    let top = "[rules.status]\nvalues = [\"todo\", \"doing\"]\nlabels = { todo = \"未着手\" }\n";
    let notes = [
        ("alpha.md", "---\nstatus: todo\n---\n"),
        ("beta.md", "---\nstatus: doing\n---\n"),
    ];
    let (_t, mut a) = boot("vlp_settings", &notes, top, "", ColorMode::None);
    ts::open(&mut a);
    ts::section(&mut a, Sec::Filters);
    let last = ts::draft(&a).len(Sec::Filters) - 1;
    ts::sel_to(&mut a, last);
    press(&mut a, KeyCode::Enter);
    ts::pick_column(&mut a, "status");
    ts::sel_to(&mut a, 0);
    press(&mut a, KeyCode::Enter);
    assert!(
        matches!(ts::draft(&a).pick, Some(Pick::Values { .. })),
        "値の一覧が開いていない"
    );
    let s = screen(&a);
    assert!(s.contains("未着手 todo"), "{s}");
}

#[test]
fn test_value_labels_picker_new_note_preset_value() {
    // [utest -> req~navigation.value-labels-picker~2] [new_note.set] status = "todo" の表で新しいノートの窓を開くと、status の欄の値は
    // 「未着手 todo」と出どころ「(設定から)」。作ったノートには status: todo と書く。
    let top = "[rules.status]\nvalues = [\"todo\", \"doing\"]\nlabels = { todo = \"未着手\" }\n\n[new_note.set]\nstatus = \"todo\"\n";
    let notes = [
        ("alpha.md", "---\nstatus: todo\n---\n"),
        ("beta.md", "---\nstatus: doing\n---\n"),
    ];
    let (t, mut a) = boot("vlp_new_note", &notes, top, "", ColorMode::None);
    ch(&mut a, 'a');
    assert!(a.note.flow.is_some(), "{:?}", a.message);
    let s = screen(&a);
    let l = line_with(&s, "(設定から)");
    assert!(l.contains("status") && l.contains("未着手 todo"), "{s}");
    super::test_screen::typing(&mut a, "gamma");
    ctrl(&mut a, 's');
    let disk = std::fs::read_to_string(t.notes().join("gamma.md")).unwrap_or_default();
    assert!(disk.contains("status: todo"), "{disk:?} {:?}", a.message);
}
