//! 表の上の検索の欄の試験(NV-23・NV-2。関係: SR-9・SR-17・CLI-3)。
use super::keymap::Mode;
use super::test_screen::*;
use super::*;
use mdgrid::config::Config;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;

/// 16 のノートのうち 3 つの title に「会議」を含む(NV-23 の例の「3/16行」)。
fn sixteen(name: &str) -> (Tmp, App) {
    let notes: Vec<(String, String)> = (0..16)
        .map(|i| {
            let t = if i % 5 == 0 && i > 0 {
                format!("定例会議 {i}")
            } else {
                format!("メモ {i}")
            };
            (format!("n{i:02}.md"), format!("---\ntitle: {t}\n---\n"))
        })
        .collect();
    let refs: Vec<(&str, &str)> = notes
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    make(name, &refs)
}

fn names(app: &App) -> Vec<String> {
    app.rows.iter().map(|r| app.src.label(r)).collect()
}

fn line(app: &App, i: usize) -> String {
    screen(app).lines().nth(i).unwrap_or_default().to_string()
}

fn bar_off(app: &mut App) {
    app.configure(
        &mdgrid::config::parse("[display]\nsearch_bar = false\n")
            .unwrap()
            .0,
    );
}

#[test]
fn test_nv_23_bar_is_shown_empty_at_start() {
    // [NV-23] 起動 → タブの下に検索の欄が空で出ている(案内は薄く。キーはキーの表から。SR-4)。
    let (_t, a) = sixteen("nv23start");
    let s = screen(&a);
    let lines: Vec<&str> = s.lines().collect();
    assert!(lines[1].contains("既定の表"), "{s}");
    assert!(lines[2].starts_with(" 検索: (\\ で絞る)"), "{s}");
    assert!(lines[3].contains("ノート"), "列の見出しは欄の下: {s}");
    assert_eq!(view::data_y(&a), 4);
    // 案内は薄い。
    let buf = buffer(&a);
    let x = (0..80u16).find(|&x| buf[(x, 2)].symbol() == "(").unwrap();
    assert!(buf[(x, 2)].modifier.contains(ratatui::style::Modifier::DIM));
    // ヘッダーに絞り込みの印は無い。
    assert!(!lines[0].contains("絞り込み"), "{s}");
}

#[test]
fn test_nv_23_bar_shows_word_and_count() {
    // [NV-23] 「会議」で絞る → 欄に「会議」と「3/16行」。打っている間は語の後ろに `▏`。
    let (_t, mut a) = sixteen("nv23count");
    ch(&mut a, '\\');
    typing(&mut a, "会議");
    assert_eq!(a.mode, Mode::Filter);
    let bar = line(&a, 2);
    assert!(bar.starts_with(" 検索: 会議▏  3/16行"), "{bar}");
    press(&mut a, KeyCode::Enter);
    let s = screen(&a);
    let bar = s.lines().nth(2).unwrap();
    assert!(bar.starts_with(" 検索: 会議  3/16行"), "{s}");
    assert!(!bar.contains('▏'), "抜けたら印は消える: {bar}");
    assert!(!s.lines().next().unwrap().contains("絞り込み"), "{s}");
    assert_eq!(a.rows.len(), 3);
    golden("nv_23", &s);
}

#[test]
fn test_nv_23_config_off_hides_bar_and_uses_bottom_line() {
    // [NV-23] 設定で欄を消す → 欄が無く、表が1行広い。`\` を押せば最下行で打てる。
    let (_t, mut a) = sixteen("nv23off");
    let h_on = a.data_height();
    bar_off(&mut a);
    assert_eq!(a.data_height(), h_on + 1, "表が1行広い");
    assert_eq!(view::data_y(&a), 3);
    let s = screen(&a);
    assert!(!s.contains("検索:"), "{s}");
    assert!(s.lines().nth(2).unwrap().contains("ノート"), "{s}");
    ch(&mut a, '\\');
    typing(&mut a, "会議");
    assert_eq!(a.rows.len(), 3);
    let s = screen(&a);
    assert!(s.lines().nth(22).unwrap().starts_with("\\会議"), "{s}");
    // カーソルは最下行の入力の末尾(SR-17)。
    assert_eq!(view::cursor(&a), Some((5, 22)));
    press(&mut a, KeyCode::Enter);
    // 欄が無いときは、効いている語をヘッダーに出す(気づけるように)。
    assert!(line(&a, 0).contains("絞り込み「会議」"));
    // クリックで欄に入ることも無い(2行目は列の見出し)。
    a.click(5, 2);
    assert_ne!(a.mode, Mode::Filter);
}

#[test]
fn test_nv_2_backslash_enters_bar_and_esc_restores() {
    // [NV-2] `\` で上の欄に入り「会議」と打つ → 「会議」を含む行だけ。Esc で元に戻る。
    let (_t, mut a) = sixteen("nv2bar");
    ch(&mut a, '\\');
    assert_eq!(a.mode, Mode::Filter);
    typing(&mut a, "会");
    assert_eq!(a.rows.len(), 3, "打つたびに絞る");
    typing(&mut a, "議");
    assert_eq!(names(&a), ["n05.md", "n10.md", "n15.md"]);
    let s = screen(&a);
    // 入力は最下行ではなく上の欄に出る。
    assert!(!s.lines().nth(22).unwrap().starts_with('\\'), "{s}");
    assert!(s.lines().nth(2).unwrap().contains("会議"), "{s}");
    // [SR-17] 端末のカーソルは欄の語の末尾(` 検索: 会議` は幅 11)。
    assert_eq!(view::cursor(&a), Some((11, 2)));
    let mut term = Terminal::new(TestBackend::new(80, 24)).unwrap();
    term.draw(|f| draw(f, &a)).unwrap();
    let got = term.get_cursor_position().unwrap();
    assert_eq!((got.x, got.y), (11, 2));
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.rows.len(), 16);
    assert!(a.filter.is_none());
    assert!(line(&a, 2).starts_with(" 検索: (\\ で絞る)"));
}

#[test]
fn test_nv_23_click_bar_enters_and_enter_keeps_filter() {
    // [NV-23] 欄のクリックで欄に入る。Enter で欄を抜けても絞り込みは残り、表のキーが効く。
    let (_t, mut a) = sixteen("nv23click");
    a.click(20, 2);
    assert_eq!(a.mode, Mode::Filter);
    typing(&mut a, "会議");
    // 欄の中のクリックは打つのを続ける。
    a.click(3, 2);
    assert_eq!(a.mode, Mode::Filter);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.filter.as_deref(), Some("会議"));
    assert_eq!(a.rows.len(), 3);
    // 表のキー: j で下、Enter で編集。
    assert_eq!(a.row, 0);
    ch(&mut a, 'j');
    assert_eq!(a.row, 1);
    col_named(&mut a, "title");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    // もう一度入ると、効いている語の続きから打てる。
    ch(&mut a, '\\');
    assert!(line(&a, 2).starts_with(" 検索: 会議▏"));
    // 欄の外(表の行)のクリックは欄を抜けて(絞り込みは残す)、その行を選ぶ。
    let y = (view::data_y(&a) + 2) as u16;
    a.click(0, y);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.filter.as_deref(), Some("会議"));
    assert_eq!(a.row, 2);
}

#[test]
fn test_nv_23_row_click_hits_right_row_below_bar() {
    // [NV-23] [SR-6] 帯が1行増えても、行のクリックは見えている行を選ぶ。設定の帯(NV-16)が重なっても同じ。
    let (_t, mut a) = sixteen("nv23hit");
    let dy = view::data_y(&a) as u16;
    assert_eq!(dy, 4);
    a.click(0, dy + 3);
    assert_eq!(a.row, 3);
    let s = screen(&a);
    assert!(
        s.lines()
            .nth((dy + 3) as usize)
            .unwrap()
            .starts_with(">n03 "),
        "{s}"
    );
    // 列の見出しのクリックは並べ替え(NV-3)。
    let head = s.lines().nth((dy - 1) as usize).unwrap();
    assert!(head.contains("ノート"), "{s}");
    // 設定の帯が出ると、さらに1行下がる。
    a.settings
        .sorts
        .push(("title".into(), mdgrid::settings::Dir::Desc));
    a.refresh();
    assert_eq!(view::data_y(&a), 5);
    let s = screen(&a);
    assert!(s.lines().nth(2).unwrap().contains("検索:"), "{s}");
    assert!(s.lines().nth(3).unwrap().contains("設定:"), "{s}");
    assert!((0..80u16).all(|x| view::chip_at(&a, x, 2).is_none()));
    assert!((0..80u16).any(|x| view::chip_at(&a, x, 3) == Some(0)));
    a.click(0, 6);
    assert_eq!(a.row, 1);
    let want = a.src.label(&a.rows[1]);
    let want = want.trim_end_matches(".md");
    assert!(s.lines().nth(6).unwrap().contains(want), "{s}");
    // 欄を消すと1行上がる。
    bar_off(&mut a);
    assert_eq!(view::data_y(&a), 4);
    a.click(0, 6);
    assert_eq!(a.row, 2);
}

#[test]
fn test_nv_23_bar_fits_narrow_terminals() {
    // [SR-9] 狭い端末でも、欄を含む全行が幅を超えない(打っている間・絞った後・案内・East Asian Ambiguous を幅2)。
    let (_t, mut a) = sixteen("nv23narrow");
    let check = |a: &mut App| {
        for (w, h) in [(80u16, 24u16), (30, 10), (14, 6), (9, 5), (5, 4), (3, 3)] {
            a.resize(w, h);
            for l in view::render(a, (w - 1) as usize, (h - 1) as usize) {
                let lw: usize = l.spans.iter().map(|s| width::width(&s.content)).sum();
                assert!(lw <= (w - 1) as usize, "幅 {w} で {lw}: {l:?}");
            }
            let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
            term.draw(|f| draw(f, a)).unwrap();
            // カーソルは画面の中にだけ置く。
            if let Some((x, y)) = view::cursor(a) {
                assert!(x < w - 1 && y < h - 1, "{w}x{h}: {x},{y}");
            }
        }
        a.resize(80, 24);
    };
    check(&mut a);
    ch(&mut a, '\\');
    typing(&mut a, "会議のとても長い語を打つとどうなるか");
    check(&mut a);
    let mut b = a;
    b.configure(&Config {
        terminal: mdgrid::config::Terminal {
            ambiguous_wide: true,
            ..Default::default()
        },
        ..Config::default()
    });
    check(&mut b);
    press(&mut b, KeyCode::Enter);
    check(&mut b);
}

#[test]
fn test_nv_23_low_terminal_bar_not_drawn_ignores_click() {
    // [NV-23] 高さ5の端末では検索の欄が描かれない(行2は下の帯)。下の帯のクリックで絞り込みの入力に入らない。
    // `\` で打つときは最下行に語を出し、カーソルも最下行に置く(SR-17)。
    let (_t, mut a) = sixteen("nv23low");
    a.resize(80, 5);
    let s = view::render(&a, 79, 4);
    let footer: String = s[2].spans.iter().map(|sp| sp.content.as_ref()).collect();
    assert!(footer.contains("未保存"), "{footer}");
    a.click(5, 2);
    assert_eq!(a.mode, Mode::Table);
    assert!(a.prompt.is_none());
    ch(&mut a, '\\');
    typing(&mut a, "会議");
    assert_eq!(a.mode, Mode::Filter);
    let s = view::render(&a, 79, 4);
    let msg: String = s[3].spans.iter().map(|sp| sp.content.as_ref()).collect();
    assert!(msg.starts_with("\\会議"), "{msg}");
    assert_eq!(view::cursor(&a).map(|c| c.1), Some(3));
}
