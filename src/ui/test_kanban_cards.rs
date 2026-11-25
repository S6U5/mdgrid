//! カンバンの札の値と名前の折り返し(kanban.card-*)の受け入れの試験。値は作ったもの。
//!
//! 実装の前に書いた。行番号や字の位置は決め打ちせず、札の名前の出る所(見出しの行より下で最初に名前が出る行と桁)を
//! 起点に、同じレーンの区画(名前の左の縦の線から右の縦の線まで)の字を下へ読む。札の行の字は、名前の最初の字の桁から
//! 読む(名前の左の2桁は選びの `>` とためた変更の `*` の印)。

use super::app::{App, ColorMode};
use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, text, Tmp};
use super::width::width;
use mdgrid::config::Config;
use mdgrid::i18n::Msg;
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::path::PathBuf;

/// 状態の列の決まり。レーンは todo・doing・done の3つ。
const RULES: &str = "[rules.status]\nvalues = [\"todo\", \"doing\", \"done\"]\n";

/// 縦の線(レーンの間と枠の縦の辺)。
const SEPS: [char; 3] = ['│', '┃', '|'];

/// 枠の下の角(札の区画の終わり)。
const BOTTOM: [char; 6] = ['╰', '└', '┗', '╯', '┘', '┛'];

/// 印の先頭に `top`、[[table]] に `line` を書いて、`notes` を表として開く(色なし・書ける)。
fn boot(tag: &str, notes: &[(&str, &str)], top: &str, line: &str, size: (u16, u16)) -> (Tmp, App) {
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
    app.resize(size.0, size.1);
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

/// `notes/t.base`(式 `formulas`)を、印の [[table]] の path で指して開く。
fn boot_base(tag: &str, notes: &[(&str, &str)], yaml: &str, line: &str) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    for (n, t) in notes {
        tmp.write(n, t);
    }
    let path = tmp.notes().join("t.base");
    std::fs::write(&path, yaml).unwrap();
    std::fs::create_dir_all(tmp.0.join(".mdgrid")).unwrap();
    std::fs::write(
        tmp.0.join(".mdgrid/workspace.toml"),
        format!("{RULES}\n[[table]]\nname = \"T\"\npath = \"notes/t.base\"\n{line}\n"),
    )
    .unwrap();
    let base = mdgrid::base::parse(yaml).unwrap();
    let src = Markdown::open_vault(&path).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(120, 30);
    app.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: path,
        base: Some((base, "t.base".into(), None)),
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

/// `B` でカンバンを開き、画面の文字を返す。
fn open(a: &mut App) -> String {
    ch(a, 'B');
    let s = screen(a);
    assert_eq!(
        a.mode,
        Mode::Kanban,
        "B でカンバンが開かない: {:?}\n{s}",
        a.message
    );
    s
}

/// 行の桁 `from` から、桁 `to` の手前までの字。
fn span(line: &str, from: usize, to: usize) -> String {
    let mut col = 0;
    let mut out = String::new();
    for c in line.chars() {
        let w = width(&c.to_string());
        if col >= from && col + w <= to {
            out.push(c);
        }
        col += w;
    }
    out
}

/// 行の桁 `x` より右で最初の縦の線の桁(無ければ行の幅)。
fn right_sep(line: &str, x: usize) -> usize {
    let mut col = 0;
    for c in line.chars() {
        if col >= x && SEPS.contains(&c) {
            return col;
        }
        col += width(&c.to_string());
    }
    col
}

/// レーンの見出しの行(「値 (数)」の並ぶ行)。
fn heading_y(s: &str) -> usize {
    s.lines()
        .position(|l| l.contains("todo (") && l.contains("doing ("))
        .unwrap_or_else(|| panic!("レーンの見出しの行が無い:\n{s}"))
}

/// 札 `name` の名前の最初の字の (桁, 行)。見出しの行より下で最初に名前が出る所。
fn card_at(s: &str, name: &str) -> Option<(usize, usize)> {
    let hy = heading_y(s);
    s.lines()
        .enumerate()
        .skip(hy + 1)
        .take_while(|(_, l)| !l.contains(&BOTTOM[..]))
        .find_map(|(y, l)| l.find(name).map(|i| (width(&l[..i]), y)))
}

/// 札 `name` の頭の行から下へ `n` 行の、同じレーンの字(名前の最初の字の桁から、右の縦の線の手前まで。末尾の空白を除く)。
fn rows_from(s: &str, name: &str, n: usize) -> Vec<String> {
    let (x, y) = card_at(s, name).unwrap_or_else(|| panic!("札 {name} が無い:\n{s}"));
    let lines: Vec<&str> = s.lines().collect();
    let end = right_sep(lines[y], x);
    lines[y..(y + n).min(lines.len())]
        .iter()
        .map(|l| span(l, x, end).trim_end().to_string())
        .collect()
}

/// 札 `name` の印(名前の左の2桁)。
fn marks(s: &str, name: &str) -> String {
    let (x, y) = card_at(s, name).unwrap_or_else(|| panic!("札 {name} が無い:\n{s}"));
    span(s.lines().nth(y).unwrap(), x.saturating_sub(2), x)
}

/// `>` の付いた札か。
fn is_selected(s: &str, name: &str) -> bool {
    card_at(s, name).is_some() && marks(s, name).contains('>')
}

/// 札の列 author・rating の見本。alpha と beta は todo、gamma は doing。
const NOTES: &[(&str, &str)] = &[
    (
        "alpha.md",
        "---\nstatus: todo\nauthor: yamada\nrating: 4\n---\n",
    ),
    ("beta.md", "---\nstatus: todo\nauthor: sato\n---\n"),
    (
        "gamma.md",
        "---\nstatus: doing\nauthor: kato\nrating: 2\n---\n",
    ),
];

const CARD: &str = "kanban = { column = \"status\", card = [\"author\", \"rating\"] }";

// ---- kanban.card-columns・kanban.card-gap・kanban.card-indent ----

#[test]
fn test_card_values_under_the_name_in_order() {
    // [utest -> req~kanban.card-columns~1, req~kanban.card-indent~1, req~kanban.card-gap~1] card = [author, rating] → alpha の札は
    // alpha・yamada・4 の3行(値は名前より2字下げ)。beta は rating が無いので beta・sato の2行。札の間は1行空く。
    let (_t, mut a) = boot("kcards1", NOTES, RULES, CARD, (120, 30));
    let s = open(&mut a);
    let rows = rows_from(&s, "alpha", 6);
    assert_eq!(
        rows,
        ["alpha", "  yamada", "  4", "", "beta", "  sato"],
        "札は名前・値(2字下げ)・空きの行・次の札:\n{s}"
    );
    let gamma = rows_from(&s, "gamma", 3);
    assert_eq!(gamma, ["gamma", "  kato", "  2"], "{s}");
}

#[test]
fn test_card_none_keeps_one_line_cards_without_gap() {
    // [utest -> req~kanban.card-columns~1, req~kanban.card-gap~1, req~kanban.open-card-label~2] card を書かない → 札は名前の1行だけで、
    // 札の間も空けない(前の札のまま)。
    let (_t, mut a) = boot(
        "kcards2",
        NOTES,
        RULES,
        "kanban = { column = \"status\" }",
        (120, 30),
    );
    let s = open(&mut a);
    assert_eq!(rows_from(&s, "alpha", 2), ["alpha", "beta"], "{s}");
    assert!(!s.contains("yamada"), "値の行は無い:\n{s}");
}

// ---- kanban.card-values-shown ----

#[test]
fn test_card_value_uses_value_label() {
    // [utest -> req~kanban.card-values-shown~1] 決まりの名前のある値は名前で(book でなく 本)。
    let top = format!("{RULES}\n[rules.kind]\nvalues = [\"book\"]\nlabels = {{ book = \"本\" }}\n");
    let notes: &[(&str, &str)] = &[("alpha.md", "---\nstatus: todo\nkind: book\n---\n")];
    let (_t, mut a) = boot(
        "kcards3",
        notes,
        &top,
        "kanban = { column = \"status\", card = [\"kind\"] }",
        (120, 30),
    );
    let s = open(&mut a);
    assert_eq!(rows_from(&s, "alpha", 2), ["alpha", "  本"], "{s}");
}

#[test]
fn test_card_value_uses_date_format() {
    // [utest -> req~kanban.card-values-shown~1] 日付の列は設定の日付の形で(表のセルと同じ)。ビューに出していない列でも。
    let top = format!("{RULES}\n[dates]\nformat = \"YYYY/MM/DD\"\n\n[types]\ndue = \"date\"\n");
    let notes: &[(&str, &str)] = &[("alpha.md", "---\nstatus: todo\ndue: 2026-12-01\n---\n")];
    let (_t, mut a) = boot(
        "kcards4",
        notes,
        &top,
        "kanban = { column = \"status\", card = [\"due\"] }",
        (120, 30),
    );
    let s = open(&mut a);
    assert_eq!(rows_from(&s, "alpha", 2), ["alpha", "  2026/12/01"], "{s}");
}

#[test]
fn test_card_value_shows_link_target_name() {
    // [utest -> req~kanban.card-values-shown~1] リンクは行き先の名前で([[ ]] を見せない)。
    let notes: &[(&str, &str)] = &[
        (
            "alpha.md",
            "---\nstatus: todo\nowner: \"[[hanako]]\"\n---\n",
        ),
        ("hanako.md", "---\nstatus: done\n---\n"),
    ];
    let (_t, mut a) = boot(
        "kcards5",
        notes,
        RULES,
        "kanban = { column = \"status\", card = [\"owner\"] }",
        (120, 30),
    );
    let s = open(&mut a);
    assert_eq!(rows_from(&s, "alpha", 2), ["alpha", "  hanako"], "{s}");
}

#[test]
fn test_card_value_shows_formula() {
    // [utest -> req~kanban.card-values-shown~1] 計算の列は式の値で。
    let yaml = "formulas:\n  twice: \"n * 2\"\nviews:\n  - type: table\n    name: v\n";
    let notes: &[(&str, &str)] = &[("alpha.md", "---\nstatus: todo\nn: 21\n---\n")];
    let (_t, mut a) = boot_base(
        "kcards6",
        notes,
        yaml,
        "kanban = { column = \"status\", card = [\"formula.twice\"] }",
    );
    let s = open(&mut a);
    assert_eq!(rows_from(&s, "alpha", 2), ["alpha", "  42"], "{s}");
}

// ---- kanban.card-config-bad・kanban.card-missing-column ----

#[test]
fn test_card_not_a_list_warns_and_opens_names_only() {
    // [utest -> req~kanban.card-config-bad~1] card が文字列 → table[1].kanban.card の警告1行。カンバンは開き、札は名前だけ。
    let (_t, mut a) = boot(
        "kcards7",
        NOTES,
        RULES,
        "kanban = { column = \"status\", card = \"author\" }",
        (120, 30),
    );
    let m = a.message.clone().unwrap_or_default();
    assert!(
        m.contains(".mdgrid/workspace.toml: table[1].kanban.card"),
        "{m}"
    );
    assert!(!m.contains('\n'), "警告は1行: {m:?}");
    assert!(
        !m.contains(Msg::RsnUnknown.text()),
        "card は知っている項目: {m}"
    );
    let s = open(&mut a);
    assert_eq!(rows_from(&s, "alpha", 2), ["alpha", "beta"], "{s}");
}

#[test]
fn test_card_with_non_string_item_warns() {
    // [utest -> req~kanban.card-config-bad~1] card = ["author", 3] → table[1].kanban.card の警告1行。
    let (_t, a) = boot(
        "kcards8",
        NOTES,
        RULES,
        "kanban = { column = \"status\", card = [\"author\", 3] }",
        (120, 30),
    );
    let m = a.message.clone().unwrap_or_default();
    assert!(
        m.contains(".mdgrid/workspace.toml: table[1].kanban.card"),
        "{m}"
    );
    assert!(!m.contains('\n'), "警告は1行: {m:?}");
    assert!(
        !m.contains(Msg::RsnUnknown.text()),
        "card は知っている項目: {m}"
    );
}

#[test]
fn test_card_missing_column_warns_after_load() {
    // [utest -> req~kanban.card-missing-column~1] card = ["nosuch", "author"] → 読み終えたときに table[1].kanban.card と nosuch を含む
    // 警告1行。札は名前と author の行。
    let (_t, mut a) = boot(
        "kcards9",
        NOTES,
        RULES,
        "kanban = { column = \"status\", card = [\"nosuch\", \"author\"] }",
        (120, 30),
    );
    let m = a.message.clone().unwrap_or_default();
    assert!(
        m.contains(".mdgrid/workspace.toml: table[1].kanban.card") && m.contains("nosuch"),
        "{m}"
    );
    assert!(!m.contains('\n'), "警告は1行: {m:?}");
    let s = open(&mut a);
    assert_eq!(rows_from(&s, "alpha", 2), ["alpha", "  yamada"], "{s}");
}

#[test]
fn test_card_existing_column_does_not_warn() {
    // [utest -> req~kanban.card-missing-column~1] 表にある列だけなら警告しない。
    let (_t, a) = boot("kcards10", NOTES, RULES, CARD, (120, 30));
    let m = a.message.clone().unwrap_or_default();
    assert!(!m.contains("kanban"), "{m}");
}

// ---- kanban.card-name-wrap・kanban.card-indent ----

/// 幅 60 で3つのレーン: 名前の欄は 1行目が幅 16、続きの行が幅 14 ほど。
const NARROW: (u16, u16) = (60, 30);
/// 2行には入る名前(幅 20)。
const LONG2: &str = "とても長い問いの名前";
/// 2行にも入らない名前(幅 40)。
const LONG3: &str = "とても長い問いの名前がここまで続いている";

#[test]
fn test_long_name_wraps_to_two_lines() {
    // [utest -> req~kanban.card-name-wrap~1, req~kanban.card-indent~1] 幅に入らない名前は2行に折り返し、つなぐと名前の全部(… は無い)。
    // 2行目は名前の頭より2字下げ。
    let notes: &[(&str, &str)] = &[
        ("a.md", "---\nstatus: todo\ntitle: x\n---\n"),
        (&format!("{LONG2}.md"), "---\nstatus: todo\n---\n"),
    ];
    let (_t, mut a) = boot(
        "kwrap1",
        notes,
        RULES,
        "kanban = { column = \"status\" }",
        NARROW,
    );
    let s = open(&mut a);
    let head = &LONG2[..LONG2.char_indices().nth(4).unwrap().0];
    let rows = rows_from(&s, head, 3);
    assert!(
        !rows[0].contains('…') && !rows[1].contains('…'),
        "2行に入るので … は無い:\n{s}"
    );
    assert!(
        rows[1].starts_with("  ") && !rows[1].starts_with("   "),
        "2行目は2字下げ: {rows:?}\n{s}"
    );
    assert_eq!(
        format!("{}{}", rows[0], rows[1].trim_start()),
        LONG2,
        "つなぐと名前の全部:\n{s}"
    );
    assert!(rows[0] != LONG2, "1行には入らない(前提):\n{s}");
}

#[test]
fn test_too_long_name_is_cut_on_the_second_line() {
    // [utest -> req~kanban.card-name-wrap~1] 2行にも入らない名前は2行で、2行目の末尾が …。3行目に続きは無い。
    let notes: &[(&str, &str)] = &[
        (&format!("{LONG3}.md"), "---\nstatus: todo\n---\n"),
        ("んん.md", "---\nstatus: todo\n---\n"),
    ];
    let (_t, mut a) = boot(
        "kwrap2",
        notes,
        RULES,
        "kanban = { column = \"status\" }",
        NARROW,
    );
    let s = open(&mut a);
    let head = &LONG3[..LONG3.char_indices().nth(4).unwrap().0];
    let rows = rows_from(&s, head, 3);
    assert!(rows[1].ends_with('…'), "2行目の末尾が …: {rows:?}\n{s}");
    assert_eq!(rows[2], "んん", "3行目は次の札: {rows:?}\n{s}");
    let joined = format!("{}{}", rows[0], rows[1].trim_start().trim_end_matches('…'));
    assert!(LONG3.starts_with(&joined), "名前の頭から: {joined}\n{s}");
}

#[test]
fn test_short_name_stays_one_line() {
    // [utest -> req~kanban.card-name-wrap~1, req~kanban.open-card-label~2] 幅に入る名前は1行(すぐ下は次の札)。
    let notes: &[(&str, &str)] = &[
        ("会議.md", "---\nstatus: todo\n---\n"),
        ("会議の後.md", "---\nstatus: todo\n---\n"),
    ];
    let (_t, mut a) = boot(
        "kwrap3",
        notes,
        RULES,
        "kanban = { column = \"status\" }",
        NARROW,
    );
    let s = open(&mut a);
    assert_eq!(rows_from(&s, "会議", 2), ["会議", "会議の後"], "{s}");
}

// ---- kanban.card-scroll・kanban.card-click・高さの違う札の H/L ----

/// todo に n01〜n12、doing に m01〜m12 の札(どれも値の列 a・b を持つ3行の札)。
fn tall_notes() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for i in 1..=12 {
        out.push((
            format!("n{i:02}.md"),
            format!("---\nstatus: todo\na: a{i:02}\nb: b{i:02}\n---\n"),
        ));
        out.push((
            format!("m{i:02}.md"),
            format!("---\nstatus: doing\na: c{i:02}\nb: d{i:02}\n---\n"),
        ));
    }
    out
}

const TALL: &str = "kanban = { column = \"status\", card = [\"a\", \"b\"] }";

fn boot_tall(tag: &str) -> (Tmp, App) {
    let notes = tall_notes();
    let refs: Vec<(&str, &str)> = notes
        .iter()
        .map(|(n, t)| (n.as_str(), t.as_str()))
        .collect();
    boot(tag, &refs, RULES, TALL, (80, 24))
}

/// カンバンを開いて todo のレーンへ移る(表の先頭の行 m01 は doing なので、開くと doing のレーンを選んでいる)。
fn open_todo(a: &mut App) -> String {
    open(a);
    ch(a, 'h');
    let s = screen(a);
    assert!(is_selected(&s, "n01"), "todo のレーンの n01 を選ぶ:\n{s}");
    s
}

/// 札 `name` の名前と値の2行が全部見える。
fn whole(s: &str, name: &str, a: &str, b: &str) -> bool {
    card_at(s, name).is_some() && {
        let rows = rows_from(s, name, 3);
        rows.len() == 3 && rows[1].trim() == a && rows[2].trim() == b
    }
}

#[test]
fn test_tall_cards_scroll_to_keep_selected_card_whole() {
    // [utest -> req~kanban.card-scroll~1, req~kanban.keys-move~1] 3行の札が 12枚のレーンで j を 10回 → 11枚目(n11)の札の名前と値の2行が
    // 全部見え、> はその札。k を 10回 → 1枚目(n01)の全部の行が見え、> はその札。
    let (_t, mut a) = boot_tall("kscroll1");
    let s = open_todo(&mut a);
    assert!(whole(&s, "n01", "a01", "b01"), "{s}");
    assert!(
        card_at(&s, "n11").is_none(),
        "前提: 11枚目は初めは見えない:\n{s}"
    );
    for _ in 0..10 {
        ch(&mut a, 'j');
    }
    let s = screen(&a);
    assert!(whole(&s, "n11", "a11", "b11"), "11枚目の全部の行:\n{s}");
    assert!(is_selected(&s, "n11"), "{s}");
    assert!(card_at(&s, "n01").is_none(), "1枚目は流れて見えない:\n{s}");
    for _ in 0..10 {
        ch(&mut a, 'k');
    }
    let s = screen(&a);
    assert!(whole(&s, "n01", "a01", "b01"), "1枚目の全部の行:\n{s}");
    assert!(is_selected(&s, "n01"), "{s}");
}

#[test]
fn test_move_tall_card_keeps_it_selected_and_whole() {
    // [utest -> req~kanban.card-scroll~1, req~kanban.move-card~1, req~kanban.move-keep-selection~1] 11枚目(n11)を L で doing へ動かす →
    // doing のレーンの最後(m12 の後)に入り、選んだまま全部の行が見える。H で戻すと todo で選んだまま全部見える。
    let (_t, mut a) = boot_tall("kscroll2");
    open_todo(&mut a);
    for _ in 0..10 {
        ch(&mut a, 'j');
    }
    ch(&mut a, 'L');
    let s = screen(&a);
    assert!(is_selected(&s, "n11"), "{:?}\n{s}", a.message);
    assert!(whole(&s, "n11", "a11", "b11"), "動かした札の全部の行:\n{s}");
    let (nx, _) = card_at(&s, "n11").unwrap();
    let (mx, _) = card_at(&s, "m12").unwrap_or_else(|| panic!("doing の札 m12 が見える:\n{s}"));
    assert_eq!(nx, mx, "n11 は doing のレーン:\n{s}");
    ch(&mut a, 'H');
    let s = screen(&a);
    assert!(is_selected(&s, "n11"), "{s}");
    assert!(whole(&s, "n11", "a11", "b11"), "{s}");
    let (tx, _) = card_at(&s, "n10").unwrap_or_else(|| panic!("todo の札 n10 が見える:\n{s}"));
    assert_eq!(
        card_at(&s, "n11").unwrap().0,
        tx,
        "n11 は todo に戻る:\n{s}"
    );
}

#[test]
fn test_click_on_value_line_selects_that_card() {
    // [utest -> req~kanban.card-click~1, req~kanban.keys-click-select~1] alpha を選んでいるとき、beta の値の行のクリックで beta を選ぶ。
    // 札の間の空きの行のクリックでは選びは変わらない。
    let (_t, mut a) = boot("kclick1", NOTES, RULES, CARD, (120, 30));
    let s = open(&mut a);
    assert!(is_selected(&s, "alpha"), "{s}");
    let (x, y) = card_at(&s, "beta").unwrap();
    // alpha の札は3行(名前・yamada・4)で、その下の空きの行は beta の名前の1行上。
    a.click(x as u16, (y - 1) as u16);
    let s = screen(&a);
    assert_eq!(a.mode, Mode::Kanban);
    assert!(is_selected(&s, "alpha"), "空きの行では変わらない:\n{s}");
    a.click((x + 2) as u16, (y + 1) as u16);
    let s = screen(&a);
    assert_eq!(a.mode, Mode::Kanban, "値の行の最初のクリックは選ぶだけ");
    assert!(is_selected(&s, "beta"), "値の行のクリックで beta:\n{s}");
    assert!(!is_selected(&s, "alpha"), "{s}");
}

#[test]
fn test_click_after_scroll_hits_the_card_under_the_mouse() {
    // [utest -> req~kanban.card-click~1, req~kanban.card-scroll~1] 流したあと(n11 を選んで1枚目が見えない)でも、見えている n10 の値の行の
    // クリックで n10 を選ぶ(札の高さで行を数える)。
    let (_t, mut a) = boot_tall("kclick2");
    open_todo(&mut a);
    for _ in 0..10 {
        ch(&mut a, 'j');
    }
    let s = screen(&a);
    let (x, y) = card_at(&s, "n10").unwrap_or_else(|| panic!("n10 が見える:\n{s}"));
    a.click((x + 2) as u16, (y + 2) as u16);
    let s = screen(&a);
    assert!(is_selected(&s, "n10"), "n10 の値の行のクリックで n10:\n{s}");
}

#[test]
fn test_click_on_wrapped_name_line_selects_that_card() {
    // [utest -> req~kanban.card-click~1] 名前の続きの行(2行目)のクリックでも、その札を選ぶ。
    let notes: &[(&str, &str)] = &[
        ("a.md", "---\nstatus: todo\n---\n"),
        (&format!("{LONG2}.md"), "---\nstatus: todo\n---\n"),
    ];
    let (_t, mut a) = boot(
        "kclick3",
        notes,
        RULES,
        "kanban = { column = \"status\" }",
        NARROW,
    );
    let s = open(&mut a);
    let head = &LONG2[..LONG2.char_indices().nth(4).unwrap().0];
    let (x, y) = card_at(&s, head).unwrap();
    a.click((x + 2) as u16, (y + 1) as u16);
    let s = screen(&a);
    assert_eq!(a.mode, Mode::Kanban);
    assert!(
        marks(&s, head).contains('>'),
        "続きの行のクリックで選ぶ:\n{s}"
    );
}
