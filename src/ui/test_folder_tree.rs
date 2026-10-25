//! 木の画面(TR-1〜TR-7)。値は作ったもの。

use super::app::{App, ColorMode};
use super::external::OpenLog;
use super::keymap::{Action, Mode};
use super::startup::Startup;
use super::test_screen::{ch, press, text, Tmp};
use super::width::width;
use mdgrid::config::Config;
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;
use std::path::PathBuf;

/// TR-3 の例の行: `bar > 開発`(2行)・`bar > 開発 > Web`(1行)・`other`(1行)・空(1行)。
const NOTES: &[(&str, &str)] = &[
    (
        "alpha.md",
        "---\nfolder: bar > 開発\norder: 20\nurl: https://example.com/alpha\nstatus: active\n---\n",
    ),
    (
        "beta.md",
        "---\nfolder: bar > 開発\norder: 10\nurl: file:///etc/passwd\nstatus: active\n---\n",
    ),
    (
        "gamma.md",
        "---\nfolder: bar > 開発 > Web\norder: 1\nstatus: archived\n---\n",
    ),
    (
        "delta.md",
        "---\nfolder: other\norder: 5\nurl: javascript:alert(1)\nstatus: active\n---\n",
    ),
    ("eps.md", "---\nfolder: \"\"\nstatus: active\n---\n"),
];

const TREE: &str = "tree = { path = \"folder\", order = \"order\" }";

/// 印(根 = 一時フォルダ)の [[table]] に `tree_line` を書いて、`notes` を表として開く。
fn boot_with(tag: &str, notes: &[(&str, &str)], tree_line: &str) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    for (n, t) in notes {
        tmp.write(n, t);
    }
    std::fs::create_dir_all(tmp.0.join(".mdgrid")).unwrap();
    std::fs::write(
        tmp.0.join(".mdgrid/workspace.toml"),
        format!("[[table]]\nname = \"BM\"\npath = \"notes\"\n{tree_line}\n"),
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
        no_color: false,
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

fn boot(tag: &str) -> (Tmp, App) {
    boot_with(tag, NOTES, TREE)
}

/// App の大きさで描いた画面の文字。
fn screen(a: &App) -> String {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    text(t.backend().buffer())
}

/// 画面の行と桁で、`what` の最初の字の位置。
fn find(s: &str, what: &str) -> Option<(u16, u16)> {
    s.lines()
        .enumerate()
        .find_map(|(y, l)| l.find(what).map(|i| (width(&l[..i]) as u16, y as u16)))
}

/// 行のノートの名前(拡張子なし)。
fn lab(a: &App, r: &mdgrid::source::RowId) -> String {
    a.src.label(r).trim_end_matches(".md").to_string()
}

/// 選んでいるフォルダの名前。
fn sel_folder(a: &App) -> String {
    let ft = a.folder_tree.as_ref().unwrap();
    ft.tree.folders[ft.sel].name.clone()
}

/// 右の欄に並ぶ行のノートの名前(順のまま)。
fn right_names(a: &App) -> Vec<String> {
    let ft = a.folder_tree.as_ref().unwrap();
    ft.tree.folders[ft.sel]
        .rows
        .iter()
        .map(|&i| lab(a, &ft.rows[i]))
        .collect()
}

/// フォルダの欄で ↓ を押して `name` のフォルダを選ぶ。
fn goto_folder(a: &mut App, name: &str) {
    for _ in 0..20 {
        if sel_folder(a) == name {
            return;
        }
        press(a, KeyCode::Down);
    }
    panic!("フォルダ {name} が選べない");
}

/// 表の画面で `name` の行を選ぶ。
fn table_row(a: &mut App, name: &str) {
    let at = a.rows.iter().position(|r| lab(a, r) == name).unwrap();
    a.row = a
        .slots
        .iter()
        .position(|s| *s == super::grid::Slot::Row(at))
        .unwrap();
}

#[test]
fn test_tr_1_menu_offers_tree_and_switches() {
    // [TR-1] 印に階層の列を書いた表で「▦ 表 ▾」を押す → 窓に「木」。選ぶ → 木の画面。
    let (_t, mut a) = boot("tr1menu");
    assert_eq!(a.message, None, "警告が無い");
    let s = screen(&a);
    let (x, y) = find(&s, "▦ 表 ▾").expect("ヘッダーのメニュー");
    a.click(x + 1, y);
    assert_eq!(a.mode, Mode::ScreenMenu);
    let s = screen(&a);
    let (x, y) = s
        .lines()
        .enumerate()
        .skip(y as usize + 1)
        .find_map(|(yy, l)| {
            l.find(" 木")
                .filter(|_| {
                    l.trim_end()
                        .trim_end_matches(['│', '|'])
                        .trim_end()
                        .ends_with('T')
                })
                .map(|i| (width(&l[..i]) as u16 + 1, yy as u16))
        })
        .unwrap_or_else(|| panic!("窓に「木」と切り替えのキー T:\n{s}"));
    a.click(x, y);
    assert_eq!(a.mode, Mode::Tree, "{:?}", a.message);
    let s = screen(&a);
    assert!(s.contains("▤ 木 ▾"), "{s}");
    // T で表に戻り、もう一度 T で木。
    ch(&mut a, 'T');
    assert_eq!(a.mode, Mode::Table);
    assert!(a.folder_tree.is_none());
    ch(&mut a, 'T');
    assert_eq!(a.mode, Mode::Tree);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
}

#[test]
fn test_tr_1_no_tree_without_setting() {
    // [TR-1] 設定の無い表 → 窓に「木」が無く、T は理由を出して表のまま。
    let (_t, mut a) = boot_with("tr1none", NOTES, "");
    a.apply(Action::Escape);
    a.open_screen_menu();
    assert_eq!(a.mode, Mode::ScreenMenu);
    let s = screen(&a);
    assert!(!s.contains('木'), "{s}");
    a.close_screen_menu();
    ch(&mut a, 'T');
    assert_eq!(a.mode, Mode::Table);
    assert!(
        a.message.as_deref().unwrap_or("").contains("tree"),
        "{:?}",
        a.message
    );
}

#[test]
fn test_tr_1_tree_uses_view_rows() {
    // [TR-1] 今の表のビューを通った行だけ: status が active の行だけのビュー → archived の行(gamma)は木に出ない。
    let tmp = Tmp::new("tr1view");
    for (n, t) in NOTES {
        tmp.write(n, t);
    }
    let base = tmp.notes().join("bm.base");
    std::fs::write(
        &base,
        "views:\n  - type: table\n    name: 使う\n    filters:\n      and:\n        - status == \"active\"\n    order:\n      - folder\n      - order\n      - url\n",
    )
    .unwrap();
    std::fs::create_dir_all(tmp.0.join(".mdgrid")).unwrap();
    std::fs::write(
        tmp.0.join(".mdgrid/workspace.toml"),
        format!("[[table]]\nname = \"BM\"\npath = \"notes/bm.base\"\n{TREE}\n"),
    )
    .unwrap();
    let t = crate::open_target(std::slice::from_ref(&base), None).unwrap();
    let mut a = App::new(Box::new(t.src), ColorMode::None);
    a.resize(120, 30);
    a.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: base.clone(),
        base: t.base,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    ch(&mut a, 'T');
    assert_eq!(a.mode, Mode::Tree, "{:?}", a.message);
    let ft = a.folder_tree.as_ref().unwrap();
    let names: Vec<String> = ft.rows.iter().map(|r| lab(&a, r)).collect();
    assert!(!names.iter().any(|n| n == "gamma"), "{names:?}");
    assert!(names.iter().any(|n| n == "alpha"), "{names:?}");
    // Web のフォルダも出ない(行が無い)。
    assert!(!screen(&a).contains("Web"), "{}", screen(&a));
}

#[test]
fn test_tr_2_empty_separator_warns_and_no_tree() {
    // [TR-2] separator = "" → 警告1行で、メニューに「木」が出ない。
    let (_t, mut a) = boot_with(
        "tr2sep",
        NOTES,
        "tree = { path = \"folder\", separator = \"\" }",
    );
    let m = a.message.clone().unwrap_or_default();
    assert!(
        m.contains(".mdgrid/workspace.toml: table[1].tree.separator: "),
        "{m}"
    );
    assert!(!a.tree_ready());
    ch(&mut a, 'T');
    assert_eq!(a.mode, Mode::Table);
}

#[test]
fn test_tr_2_missing_column_warns_and_no_tree() {
    // [TR-2] path の列が表に無い → 読み終えたときに CLI-3 の形の警告で、木を出さない。
    let (_t, mut a) = boot_with("tr2col", NOTES, "tree = { path = \"place\" }");
    let m = a.message.clone().unwrap_or_default();
    assert!(
        m.contains(".mdgrid/workspace.toml: table[1].tree.path: ") && m.contains("place"),
        "{m}"
    );
    assert!(!a.tree_ready());
    ch(&mut a, 'T');
    assert_eq!(a.mode, Mode::Table);
}

#[test]
fn test_tr_2_custom_separator() {
    // [TR-2] separator を書けばその区切りで切る。
    let notes: &[(&str, &str)] = &[
        ("a.md", "---\nfolder: x/y\n---\n"),
        ("b.md", "---\nfolder: x\n---\n"),
    ];
    let (_t, mut a) = boot_with(
        "tr2custom",
        notes,
        "tree = { path = \"folder\", separator = \"/\" }",
    );
    ch(&mut a, 'T');
    let ft = a.folder_tree.as_ref().unwrap();
    let names: Vec<(String, usize)> = ft
        .shown()
        .into_iter()
        .map(|f| (ft.tree.folders[f].name.clone(), ft.tree.folders[f].count))
        .collect();
    assert_eq!(names, vec![("x".into(), 2), ("y".into(), 1)]);
}

#[test]
fn test_tr_3_tree_counts_fold_and_none() {
    // [TR-3] 左の木: bar(3)・開発(3)・Web(1)・other(1)・(なし)(1)。開発を閉じる → Web が隠れ、数は3のまま。
    let (_t, mut a) = boot("tr3");
    ch(&mut a, 'T');
    let s = screen(&a);
    let line = |s: &str, name: &str| -> String {
        s.lines()
            .find(|l| l.contains(name))
            .unwrap_or_else(|| panic!("{name}:\n{s}"))
            .to_string()
    };
    for (name, n) in [
        ("▾ bar", "3"),
        ("▾ 開発", "3"),
        ("Web", "1"),
        ("other", "1"),
        ("(なし)", "1"),
    ] {
        let l = line(&s, name);
        let after = &l[l.find(name).unwrap() + name.len()..];
        let first: String = after
            .trim_start()
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        assert_eq!(first, n, "{name}: {l}");
    }
    // 並び: bar → 開発 → Web → other → (なし)。
    let ys: Vec<u16> = ["▾ bar", "▾ 開発", "Web", "other", "(なし)"]
        .iter()
        .map(|n| find(&s, n).unwrap().1)
        .collect();
    assert!(ys.windows(2).all(|w| w[0] < w[1]), "{ys:?}\n{s}");
    goto_folder(&mut a, "開発");
    press(&mut a, KeyCode::Left);
    let s = screen(&a);
    assert!(!s.contains("Web"), "{s}");
    let l = line(&s, "▸ 開発");
    assert!(l.contains(" 3"), "{l}");
    press(&mut a, KeyCode::Char(' '));
    assert!(screen(&a).contains("Web"));
}

#[test]
fn test_tr_4_rows_by_order_and_enter_jumps() {
    // [TR-4] 開発 を選ぶ → order の小さい順(数でない・空は後ろ)。行で Enter → 表の画面でその行。
    let mut notes = NOTES.to_vec();
    notes.push(("zeta.md", "---\nfolder: bar > 開発\norder: abc\n---\n"));
    notes.push(("omega.md", "---\nfolder: bar > 開発\n---\n"));
    let (_t, mut a) = boot_with("tr4", &notes, TREE);
    ch(&mut a, 'T');
    goto_folder(&mut a, "開発");
    assert_eq!(right_names(&a), vec!["beta", "alpha", "omega", "zeta"]);
    let s = screen(&a);
    let ys: Vec<u16> = ["beta.md", "alpha.md", "omega.md", "zeta.md"]
        .iter()
        .map(|n| find(&s, n).unwrap_or_else(|| panic!("{n}:\n{s}")).1)
        .collect();
    assert!(ys.windows(2).all(|w| w[0] < w[1]), "{ys:?}\n{s}");
    // 今のビューの列(folder・order など)も出る。
    assert!(
        s.lines().any(|l| l.contains("beta") && l.contains("10")),
        "{s}"
    );
    // 右の欄へ移って2行目(alpha)で Enter。
    press(&mut a, KeyCode::Tab);
    press(&mut a, KeyCode::Up);
    press(&mut a, KeyCode::Up);
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    let cur = a.cur_row().unwrap();
    assert_eq!(lab(&a, &cur), "alpha");
}

#[test]
fn test_tr_4_opens_at_current_row() {
    // [TR-4] 表で選んでいる行があれば、木はそのフォルダとその行を選んで開く。
    let (_t, mut a) = boot("tr4cur");
    table_row(&mut a, "gamma");
    ch(&mut a, 'T');
    assert_eq!(sel_folder(&a), "Web");
    assert_eq!(right_names(&a), vec!["gamma"]);
}

#[test]
fn test_tr_5_open_url_shows_and_passes_value() {
    // [TR-5] url が https の行で「URL を開く」→ 帯に URL、開く口に渡る。file・javascript の行は開かない。
    let (_t, mut a) = boot("tr5");
    let log = OpenLog::default();
    let seen = log.0.clone();
    a.opener = Box::new(log);
    table_row(&mut a, "alpha");
    ch(&mut a, 'T');
    a.folder_tree.as_mut().unwrap().focus = super::folder_tree::Pane::Rows;
    // 操作の一覧(x)に「URL を開く」がある。
    ch(&mut a, 'x');
    let s = screen(&a);
    assert!(in_menu(&s, "URL を開く"), "{s}");
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        *seen.borrow(),
        vec!["https://example.com/alpha".to_string()]
    );
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains("https://example.com/alpha"), "{m}");
    assert!(screen(&a).contains("https://example.com/alpha"));
    // file:/// の行(beta)は開かない: 一覧に出ず、o は理由を出す。
    press(&mut a, KeyCode::Up);
    assert_eq!(
        lab(&a, a.folder_tree.as_ref().unwrap().row().unwrap()),
        "beta"
    );
    ch(&mut a, 'x');
    assert!(in_menu(&screen(&a), "表でこの行を見る"), "{}", screen(&a));
    assert!(!in_menu(&screen(&a), "URL を開く"), "{}", screen(&a));
    press(&mut a, KeyCode::Esc);
    ch(&mut a, 'o');
    assert_eq!(seen.borrow().len(), 1);
    assert!(a.message.is_some());
    // javascript: の行(delta)も開かない。
    goto_folder_any(&mut a, "other");
    ch(&mut a, 'o');
    assert_eq!(seen.borrow().len(), 1);
    // パレットの open_url(木から開いて木に戻る)。
    goto_folder_any(&mut a, "開発");
    a.folder_tree.as_mut().unwrap().focus = super::folder_tree::Pane::Rows;
    a.folder_tree.as_mut().unwrap().item = 1;
    ch(&mut a, ':');
    for c in "open_url".chars() {
        ch(&mut a, c);
    }
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Tree);
    assert_eq!(seen.borrow().len(), 2);
    // 表の画面の操作の一覧(SR-24)の行の節にも出る。
    press(&mut a, KeyCode::Esc);
    table_row(&mut a, "alpha");
    let items: Vec<Action> = super::menu::items(&a)
        .into_iter()
        .map(|i| i.action)
        .collect();
    assert!(items.contains(&Action::OpenUrl), "{items:?}");
    table_row(&mut a, "beta");
    let items: Vec<Action> = super::menu::items(&a)
        .into_iter()
        .map(|i| i.action)
        .collect();
    assert!(!items.contains(&Action::OpenUrl), "{items:?}");
}

/// 行の操作の窓に `label` の項目があるか(窓の縁の縦の線の内側の「> 」か「  」の後ろ)。
fn in_menu(s: &str, label: &str) -> bool {
    s.lines()
        .any(|l| l.contains(&format!("│> {label}")) || l.contains(&format!("│  {label}")))
}

/// フォルダの欄に移って `name` を選ぶ。
fn goto_folder_any(a: &mut App, name: &str) {
    a.folder_tree.as_mut().unwrap().focus = super::folder_tree::Pane::Folders;
    for _ in 0..20 {
        press(a, KeyCode::Up);
    }
    goto_folder(a, name);
}

#[test]
fn test_tr_6_layout_by_width() {
    // [TR-6] 幅 160・100 は並べ、60 は1つずつ(タブと Tab で切り替え)。どの幅でも行は幅に収まる。
    let (_t, mut a) = boot("tr6");
    ch(&mut a, 'T');
    goto_folder(&mut a, "開発");
    for w in [160u16, 100] {
        a.resize(w, 30);
        let s = screen(&a);
        let both = s
            .lines()
            .any(|l| l.contains("フォルダ") && l.contains("行: bar > 開発"));
        assert!(both, "幅 {w} で並べる:\n{s}");
        fits(&a, w);
    }
    a.resize(60, 30);
    let s = screen(&a);
    assert!(s.contains("フォルダ"), "{s}");
    assert!(!s.contains("alpha"), "1つずつ(フォルダの欄だけ):\n{s}");
    fits(&a, 60);
    press(&mut a, KeyCode::Tab);
    let s = screen(&a);
    assert!(s.contains("alpha") && s.contains("行: bar > 開発"), "{s}");
    assert!(!s.contains("other"), "行の欄だけ:\n{s}");
    fits(&a, 60);
}

/// 描いた行が幅 `w - 1` に収まり、ちょうど埋まる(重ならない)。
fn fits(a: &App, w: u16) {
    for l in super::view::render(a, (w - 1) as usize, (a.size.1 - 1) as usize) {
        let lw: usize = l.spans.iter().map(|s| width(&s.content)).sum();
        assert!(lw <= (w - 1) as usize, "幅 {w} で {lw}: {l:?}");
    }
}

#[test]
fn test_tr_7_mouse() {
    // [TR-7] 開発 の印のクリック → 閉じる。Web をクリック → 選ぶ。右の行のクリック2回 → 表のその行へ。ホイールで上下。
    let (_t, mut a) = boot("tr7");
    ch(&mut a, 'T');
    let s = screen(&a);
    let (x, y) = find(&s, "▾ 開発").unwrap();
    a.click(x, y);
    assert!(!screen(&a).contains("Web"), "閉じる:\n{}", screen(&a));
    let s = screen(&a);
    let (x, y) = find(&s, "▸ 開発").unwrap();
    a.click(x, y);
    let s = screen(&a);
    let (x, y) = find(&s, "Web").unwrap_or_else(|| panic!("開く:\n{s}"));
    a.click(x + 1, y);
    assert_eq!(sel_folder(&a), "Web");
    // ホイールで上下にフォルダを選ぶ。
    a.wheel(false);
    assert_eq!(sel_folder(&a), "開発");
    a.wheel(true);
    assert_eq!(sel_folder(&a), "Web");
    a.wheel(false);
    // 右の行(alpha)のクリック: 1回目は選ぶ、2回目で表のその行へ。
    let s = screen(&a);
    let (x, y) = s
        .lines()
        .enumerate()
        .find_map(|(yy, l)| l.rfind("alpha").map(|i| (width(&l[..i]) as u16, yy as u16)))
        .unwrap();
    a.click(x + 1, y);
    assert_eq!(a.mode, Mode::Tree);
    assert_eq!(
        lab(&a, a.folder_tree.as_ref().unwrap().row().unwrap()),
        "alpha"
    );
    a.click(x + 1, y);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(lab(&a, &a.cur_row().unwrap()), "alpha");
}
