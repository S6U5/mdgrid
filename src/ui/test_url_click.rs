//! 開ける URL のクリック(screen.url-click-open・screen.open-url-cell-first・screen.url-click-hint・
//! tree.url-click-open・navigation.detail-click-select・navigation.detail-url-open)。値は作ったもの。期待値は要求の
//! 受け入れの例から取った。本物のブラウザは開かない(開く口を、渡された URL を覚えるものに替える)。

use super::app::{App, ColorMode};
use super::external::OpenLog;
use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, col_named, text, Tmp};
use super::view;
use super::width::width;
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

/// 開く口に渡った URL。
type Seen = std::rc::Rc<std::cell::RefCell<Vec<String>>>;

/// url の列の幅 24 桁で `…` に切って見せる長い URL。
const LONG: &str =
    "https://docs.example.com/guides/workspace-setup/advanced-configuration/a-very-long-page-name";
const EXE: &str = "https://example.com/files/report.pdf.exe";
const HINT_O: &str = "URL: もう一度クリックか O で開く";

fn note(title: &str, url: &str) -> String {
    format!("---\ntitle: {title}\nurl: {url}\n---\n")
}

/// 印の [[table]] に `table` を書き(空なら印を置かない)、設定 `config` で `notes` を開いて、開く口を記録に替える。
fn boot(tag: &str, notes: &[(&str, &str)], config: &str, table: &str) -> (Tmp, App, Seen) {
    let tmp = Tmp::new(tag);
    for (n, t) in notes {
        tmp.write(n, t);
    }
    if !table.is_empty() {
        std::fs::create_dir_all(tmp.0.join(".mdgrid")).unwrap();
        std::fs::write(
            tmp.0.join(".mdgrid/workspace.toml"),
            format!("[[table]]\nname = \"BM\"\npath = \"notes\"\n{table}\n"),
        )
        .unwrap();
    }
    let toml = format!("[display]\ntop_padding = 0\nbar_padding = 0\n{config}");
    let (cfg, warnings) = mdgrid::config::parse(&toml).unwrap();
    assert!(warnings.is_empty(), "設定の警告: {warnings:?}");
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(100, 30);
    a.start(Startup {
        config: cfg,
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: tmp.notes(),
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    a.widths.insert("url".into(), 24);
    let log = OpenLog::default();
    let seen = log.0.clone();
    a.opener = Box::new(log);
    a.message = None;
    (tmp, a, seen)
}

/// App の大きさで描いた画面の文字。
fn shot(a: &App) -> String {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    text(t.backend().buffer())
}

/// 描いた画面のメッセージ行(いちばん下の行)。
fn message_line(a: &App) -> String {
    shot(a)
        .lines()
        .rfind(|l| !l.is_empty())
        .unwrap_or_default()
        .to_string()
}

/// 表の `name` のノートの行の添字(slots)。
fn slot_of(a: &App, name: &str) -> usize {
    (0..a.slots.len())
        .find(|&i| {
            matches!(a.slots[i], super::grid::Slot::Row(r)
            if a.src.label(&a.rows[r]).trim_end_matches(".md") == name)
        })
        .unwrap_or_else(|| panic!("{name} の行が表に無い"))
}

/// 表の `name` のノートの行の、列 `col` のセルの左の桁と行(見えている列)。
fn cell_xy(a: &App, name: &str, col: &str) -> (u16, u16) {
    let i = slot_of(a, name);
    let (lay, cols) = view::visible_layout(a);
    let j = a.cols.iter().position(|c| c == col).unwrap();
    let mut x = lay.data_x();
    for &(k, w) in &cols {
        if k == j {
            return ((x + 1) as u16, (view::data_y(a) + i - a.top) as u16);
        }
        x += w + 1;
    }
    panic!("列 {col} が見えていない");
}

/// 画面の中で `needle` が出る最初の所(桁と行)。
fn find(s: &str, needle: &str) -> Option<(u16, u16)> {
    s.lines()
        .enumerate()
        .find_map(|(y, l)| l.find(needle).map(|i| (width(&l[..i]) as u16, y as u16)))
}

fn cur_col(a: &App) -> String {
    a.cols[a.col].clone()
}

fn cur_name(a: &App) -> String {
    a.src
        .label(&a.cur_row().unwrap())
        .trim_end_matches(".md")
        .to_string()
}

// ---- 表の画面(screen.url-click-open) ----

// [utest -> req~screen.url-click-open~1]
/// 別の行を選んでいる表で、長い URL のセルを1回クリック → そのセルを選ぶだけ(開かず、編集も始めない)。
#[test]
fn test_first_click_on_url_cell_of_other_row_selects_only() {
    let (_t, mut a, seen) = boot(
        "url_click_other_row",
        &[("a.md", &note("a", "x")), ("b.md", &note("b", LONG))],
        "",
        "",
    );
    a.row = slot_of(&a, "a");
    col_named(&mut a, "title");
    assert!(
        shot(&a).contains("https://docs.example.co…"),
        "{}",
        shot(&a)
    );
    let (x, y) = cell_xy(&a, "b", "url");
    a.click(x, y);
    assert_eq!((cur_name(&a), cur_col(&a)), ("b".into(), "url".into()));
    assert_eq!(a.mode, Mode::Table);
    assert!(seen.borrow().is_empty(), "{:?}", seen.borrow());
}

// [utest -> req~screen.url-click-open~1]
/// 同じ行の title を選んでいる表で、url のセルを1回クリック → url のセルを選ぶだけ(編集は始まらない)。
#[test]
fn test_first_click_on_url_cell_of_same_row_selects_only() {
    let (_t, mut a, seen) = boot("url_click_same_row", &[("b.md", &note("b", LONG))], "", "");
    a.row = slot_of(&a, "b");
    col_named(&mut a, "title");
    let (x, y) = cell_xy(&a, "b", "url");
    a.click(x, y);
    assert_eq!(cur_col(&a), "url");
    assert_eq!(a.mode, Mode::Table, "編集が始まった");
    assert!(seen.borrow().is_empty(), "{:?}", seen.borrow());
}

// [utest -> req~screen.url-click-open~1]
/// url のセルを選んでいる表で、そのセルをもう一度クリック → メッセージ行に URL、開く口に切る前の全体が1回、編集なし。
#[test]
fn test_reclick_on_selected_url_cell_opens_full_url() {
    let (_t, mut a, seen) = boot("url_click_open", &[("b.md", &note("b", LONG))], "", "");
    a.row = slot_of(&a, "b");
    col_named(&mut a, "title");
    let (x, y) = cell_xy(&a, "b", "url");
    a.click(x, y);
    a.click(x, y);
    assert_eq!(*seen.borrow(), vec![LONG.to_string()]);
    assert_eq!(a.mode, Mode::Table, "編集が始まった");
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains(LONG), "{m}");
}

// [utest -> req~screen.url-click-open~1]
/// 実行ファイルを指す URL のセルをもう一度クリック → 開かず、理由を出す。
#[test]
fn test_reclick_on_disguised_url_cell_shows_reason() {
    let (_t, mut a, seen) = boot("url_click_exe", &[("b.md", &note("b", EXE))], "", "");
    a.row = slot_of(&a, "b");
    col_named(&mut a, "url");
    let (x, y) = cell_xy(&a, "b", "url");
    a.click(x, y);
    assert!(seen.borrow().is_empty(), "{:?}", seen.borrow());
    assert_eq!(a.mode, Mode::Table, "編集が始まった");
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains(".exe"), "{m}");
}

// [utest -> req~screen.url-click-open~1, req~screen.mouse-click-edit~2]
/// `file:///etc/passwd` のセルをもう一度クリック → URL のセルとして扱わず、開かずに編集を始める。
#[test]
fn test_reclick_on_file_url_cell_starts_edit() {
    let (_t, mut a, seen) = boot(
        "url_click_file",
        &[("b.md", &note("b", "file:///etc/passwd"))],
        "",
        "",
    );
    a.row = slot_of(&a, "b");
    col_named(&mut a, "url");
    let (x, y) = cell_xy(&a, "b", "url");
    a.click(x, y);
    assert!(seen.borrow().is_empty(), "{:?}", seen.borrow());
    assert_eq!(a.mode, Mode::Edit);
}

// [utest -> req~screen.url-click-open~1]
/// 2つの表を縦に並べたページで、下の表の url のセルを選んで、もう一度クリック → 開く口にそのセルの URL の全体。
#[test]
fn test_reclick_on_url_cell_in_stacked_table_opens() {
    let (_t1, mut top, seen) = boot("url_click_stack_top", &[("a.md", &note("a", "x"))], "", "");
    let (_t2, mut low, _) = boot("url_click_stack_low", &[("b.md", &note("b", LONG))], "", "");
    low.opener = Box::new(OpenLog(seen.clone()));
    low.place_name = Some("low".into());
    top.stack_push(low);
    top.resize(100, 30);
    assert_eq!(cur_name(&top), "b");
    col_named(&mut top, "title");
    // 下の表の url の文字の所(並べた画面を描いて探す)。
    let s = {
        let mut t = Terminal::new(TestBackend::new(100, 30)).unwrap();
        t.draw(|f| super::draw_screen(f, &mut top)).unwrap();
        text(t.backend().buffer())
    };
    let (x, y) = find(&s, "https://docs.example.co").unwrap_or_else(|| panic!("{s}"));
    assert!(y > 3, "下の表の行でない:\n{s}");
    top.click(x + 2, y);
    assert_eq!(cur_col(&top), "url", "{s}");
    assert!(seen.borrow().is_empty(), "{:?}", seen.borrow());
    top.click(x + 2, y);
    assert_eq!(*seen.borrow(), vec![LONG.to_string()]);
    assert_eq!(top.mode, Mode::Table);
}

// ---- 表の `O`(screen.open-url-cell-first) ----

fn two_urls(tag: &str) -> (Tmp, App, Seen) {
    let (t, mut a, seen) = boot(
        tag,
        &[(
            "b.md",
            "---\ntitle: b\nurl: https://example.com/a\nmirror: https://example.com/b\n---\n",
        )],
        "",
        "",
    );
    assert!(
        a.cols.iter().position(|c| c == "url") < a.cols.iter().position(|c| c == "mirror"),
        "{:?}",
        a.cols
    );
    a.row = slot_of(&a, "b");
    (t, a, seen)
}

// [utest -> req~screen.open-url-cell-first~1]
/// mirror のセルを選んで `O` → mirror の URL。
#[test]
fn test_shift_o_opens_the_selected_url_cell_first() {
    let (_t, mut a, seen) = two_urls("url_o_cell");
    col_named(&mut a, "mirror");
    ch(&mut a, 'O');
    assert_eq!(*seen.borrow(), vec!["https://example.com/b".to_string()]);
}

// [utest -> req~screen.open-url-cell-first~1]
/// URL でない title のセルを選んで `O` → 行の最初の URL。
#[test]
fn test_shift_o_on_other_cell_opens_the_first_url_of_the_row() {
    let (_t, mut a, seen) = two_urls("url_o_row");
    col_named(&mut a, "title");
    ch(&mut a, 'O');
    assert_eq!(*seen.borrow(), vec!["https://example.com/a".to_string()]);
}

// ---- 案内(screen.url-click-hint) ----

// [utest -> req~screen.url-click-hint~1]
/// 幅 100 の表で url のセルを選ぶ → メッセージ行に案内。title のセルでは出ない。
#[test]
fn test_hint_on_url_cell_only() {
    let (_t, mut a, _) = boot(
        "url_hint_table",
        &[("b.md", &note("b", "https://example.com/mail"))],
        "",
        "",
    );
    a.row = slot_of(&a, "b");
    col_named(&mut a, "url");
    assert_eq!(message_line(&a), HINT_O);
    col_named(&mut a, "title");
    assert!(!shot(&a).contains("もう一度クリック"), "{}", shot(&a));
}

// [utest -> req~screen.url-click-hint~1]
/// 実行ファイルを指す URL のセルでは、案内を出さない。
#[test]
fn test_no_hint_on_disguised_url() {
    let (_t, mut a, _) = boot("url_hint_exe", &[("b.md", &note("b", EXE))], "", "");
    a.row = slot_of(&a, "b");
    col_named(&mut a, "url");
    assert!(!shot(&a).contains("もう一度クリック"), "{}", shot(&a));
}

// [utest -> req~screen.url-click-hint~1]
/// 幅 20 の表では、案内の切れ端も出さない。
#[test]
fn test_no_hint_when_too_narrow() {
    let (_t, mut a, _) = boot(
        "url_hint_narrow",
        &[("b.md", &note("b", "https://example.com/mail"))],
        "",
        "",
    );
    a.resize(20, 30);
    a.row = slot_of(&a, "b");
    col_named(&mut a, "url");
    let s = shot(&a);
    assert!(!s.contains("URL:") && !s.contains("もう一度"), "{s}");
}

// [utest -> req~screen.url-click-hint~1]
/// 表の `O` を外したら、クリックだけを言う。
#[test]
fn test_hint_without_key() {
    let (_t, mut a, _) = boot(
        "url_hint_nokey",
        &[("b.md", &note("b", "https://example.com/mail"))],
        "[keys.table]\n\"O\" = \"none\"\n",
        "",
    );
    a.row = slot_of(&a, "b");
    col_named(&mut a, "url");
    assert_eq!(message_line(&a), "URL: もう一度クリックで開く");
}

// ---- 木の画面(tree.url-click-open・screen.url-click-hint) ----

const TREE_NOTES: &[(&str, &str)] = &[
    (
        "alpha.md",
        "---\nfolder: bar\norder: 10\nurl: https://example.com/alpha\n---\n",
    ),
    (
        "beta.md",
        "---\nfolder: bar\norder: 20\nurl: file:///etc/passwd\n---\n",
    ),
];

fn tree(tag: &str) -> (Tmp, App, Seen) {
    let (t, mut a, seen) = boot(
        tag,
        TREE_NOTES,
        "",
        "tree = { path = \"folder\", order = \"order\" }",
    );
    ch(&mut a, 'T');
    assert_eq!(a.mode, Mode::Tree);
    a.folder_tree.as_mut().unwrap().focus = super::folder_tree::Pane::Rows;
    (t, a, seen)
}

fn tree_row(a: &App) -> String {
    a.src
        .label(a.folder_tree.as_ref().unwrap().row().unwrap())
        .trim_end_matches(".md")
        .to_string()
}

/// 木の画面の右の欄で、`text` が出る所(右の欄は画面の右の側にある)。
fn right_xy(a: &App, text: &str) -> (u16, u16) {
    let s = shot(a);
    find(&s, text).unwrap_or_else(|| panic!("{text} が無い:\n{s}"))
}

// [utest -> req~tree.url-click-open~1, req~tree.mouse-row-reclick~2]
/// 別の行を選んでいて alpha の url の値をクリック → alpha を選ぶだけ。もう一度 → 開く(木のまま)。
#[test]
fn test_tree_url_click_selects_then_opens() {
    let (_t, mut a, seen) = tree("url_click_tree");
    a.folder_tree.as_mut().unwrap().item = 1;
    assert_eq!(tree_row(&a), "beta");
    let (x, y) = right_xy(&a, "https://example.com/alp…");
    a.click(x + 2, y);
    assert_eq!(a.mode, Mode::Tree);
    assert_eq!(tree_row(&a), "alpha");
    assert!(seen.borrow().is_empty(), "{:?}", seen.borrow());
    a.click(x + 2, y);
    assert_eq!(a.mode, Mode::Tree);
    assert_eq!(
        *seen.borrow(),
        vec!["https://example.com/alpha".to_string()]
    );
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains("https://example.com/alpha"), "{m}");
}

// [utest -> req~tree.url-click-open~1, req~tree.mouse-row-reclick~2]
/// alpha を選んでいて、名前をもう一度クリック → 表の alpha の行へ(開かない)。
#[test]
fn test_tree_name_reclick_still_goes_to_table() {
    let (_t, mut a, seen) = tree("url_click_tree_name");
    assert_eq!(tree_row(&a), "alpha");
    let (x, y) = right_xy(&a, ">alpha");
    a.click(x + 2, y);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(cur_name(&a), "alpha");
    assert!(seen.borrow().is_empty(), "{:?}", seen.borrow());
}

// [utest -> req~tree.url-click-open~1]
/// beta(file:///)の url の値をもう一度クリック → 開かず、表の beta の行へ。
#[test]
fn test_tree_file_url_reclick_goes_to_table() {
    let (_t, mut a, seen) = tree("url_click_tree_file");
    a.folder_tree.as_mut().unwrap().item = 1;
    let (x, y) = right_xy(&a, "file:///etc/passwd");
    a.click(x + 2, y);
    assert!(seen.borrow().is_empty(), "{:?}", seen.borrow());
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(cur_name(&a), "beta");
}

// [utest -> req~screen.url-click-hint~1]
/// 木の画面で url のある行を選ぶ → メッセージ行に `o` の案内。
#[test]
fn test_tree_hint() {
    let (_t, a, _) = tree("url_hint_tree");
    assert_eq!(tree_row(&a), "alpha");
    assert_eq!(message_line(&a), "URL: もう一度クリックか o で開く");
}

// ---- 詳細の表示(navigation.detail-click-select・navigation.detail-url-open・screen.url-click-hint) ----

fn detail(tag: &str, url: &str) -> (Tmp, App, Seen) {
    let (t, mut a, seen) = boot(
        tag,
        &[(
            "b.md",
            &format!("---\ntitle: b\nstatus: todo\nurl: {url}\nmemo: m\n---\n"),
        )],
        "",
        "",
    );
    a.row = slot_of(&a, "b");
    col_named(&mut a, "title");
    ch(&mut a, 'K');
    assert_eq!(a.mode, Mode::Detail);
    (t, a, seen)
}

fn detail_sel(a: &App) -> String {
    let d = a.detail.as_ref().unwrap();
    a.detail_props(&d.row)[d.sel].clone()
}

/// 詳細の表示で、項目 `key` の行(キーの欄の名前が出る行)の所。
fn key_xy(a: &App, key: &str) -> (u16, u16) {
    let s = shot(a);
    s.lines()
        .enumerate()
        .find_map(|(y, l)| {
            let t = l.trim_start_matches(['>', ' ']);
            t.starts_with(&format!("{key} "))
                .then(|| (width(&l[..l.len() - t.len()]) as u16, y as u16))
        })
        .unwrap_or_else(|| panic!("{key} の行が無い:\n{s}"))
}

// [utest -> req~navigation.detail-click-select~1]
/// 1つ目の項目を選んでいて、3つ目の項目の行をクリック → 3つ目を選ぶ(編集は始まらない)。
#[test]
fn test_detail_click_selects_property() {
    let (_t, mut a, _) = detail("detail_click", "https://example.com/mail");
    let first = detail_sel(&a);
    let third = a.detail_props(&a.detail.as_ref().unwrap().row)[2].clone();
    assert_ne!(first, third);
    let (x, y) = key_xy(&a, &third);
    a.click(x + 1, y);
    assert_eq!(detail_sel(&a), third);
    assert_eq!(a.mode, Mode::Detail);
}

// [utest -> req~navigation.detail-click-select~1]
/// 2行に折り返した url の値の2行目をクリック → url を選ぶ。
#[test]
fn test_detail_click_on_wrapped_line_selects_property() {
    let (_t, mut a, _) = detail("detail_click_wrap", LONG);
    a.resize(60, 30);
    let s = shot(&a);
    let (_, y) = key_xy(&a, "url");
    let line2 = s.lines().nth(y as usize + 1).unwrap().to_string();
    assert!(
        line2
            .trim_start()
            .starts_with(|c: char| c.is_ascii_alphanumeric() || c == '-' || c == '/'),
        "{s}"
    );
    a.click((width(&line2) - 1) as u16, y + 1);
    assert_eq!(detail_sel(&a), "url");
}

/// 詳細の表示で、url の値の所。
fn url_value_xy(a: &App, url_head: &str) -> (u16, u16) {
    let s = shot(a);
    find(&s, url_head).unwrap_or_else(|| panic!("{url_head} が無い:\n{s}"))
}

// [utest -> req~navigation.detail-url-open~1]
/// url を選んでいない詳細で url の値をクリック → 選ぶだけ。もう一度 → 開く(詳細のまま)。
#[test]
fn test_detail_url_click_selects_then_opens() {
    let (_t, mut a, seen) = detail("detail_url_open", "https://example.com/mail");
    assert_ne!(detail_sel(&a), "url");
    let (x, y) = url_value_xy(&a, "https://example.com/mail");
    a.click(x + 2, y);
    assert_eq!(detail_sel(&a), "url");
    assert!(seen.borrow().is_empty(), "{:?}", seen.borrow());
    a.click(x + 2, y);
    assert_eq!(*seen.borrow(), vec!["https://example.com/mail".to_string()]);
    assert_eq!(a.mode, Mode::Detail);
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains("https://example.com/mail"), "{m}");
}

// [utest -> req~navigation.detail-url-open~1, req~screen.url-click-hint~1]
/// url を選んでいる詳細で、メッセージ行に案内。`O` → 開く。
#[test]
fn test_detail_shift_o_opens_and_hint() {
    let (_t, mut a, seen) = detail("detail_url_o", "https://example.com/mail");
    let (x, y) = url_value_xy(&a, "https://example.com/mail");
    a.click(x + 2, y);
    assert_eq!(detail_sel(&a), "url");
    assert_eq!(message_line(&a), HINT_O);
    ch(&mut a, 'O');
    assert_eq!(*seen.borrow(), vec!["https://example.com/mail".to_string()]);
    assert_eq!(a.mode, Mode::Detail);
}

// [utest -> req~navigation.detail-url-open~1]
/// 実行ファイルを指す URL の項目の値をもう一度クリック → 開かず、理由。
#[test]
fn test_detail_disguised_url_is_refused() {
    let (_t, mut a, seen) = detail("detail_url_exe", EXE);
    let (x, y) = url_value_xy(&a, "https://example.com/files");
    a.click(x + 2, y);
    a.click(x + 2, y);
    assert!(seen.borrow().is_empty(), "{:?}", seen.borrow());
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains(".exe"), "{m}");
}
