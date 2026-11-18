//! カンバンの画面(KB-1〜KB-5)の受け入れの試験。値は作ったもの。
//! 記録: specs/_history/decidespec/_changes/2026-11-02-kanban.md(タスク 1・3・4)。
//!
//! 実装の前に書いた。まだ無い型・関数・App の欄は使わず、キーとマウスを送り、画面の文字と今ある口
//! (`mode`・`message`・`changes.count()`・`cur_row()`・`click`・`wheel`・`hwheel`・`poll`)で確かめる。
//! 行番号や字の位置は決め打ちせず、題(「カンバン」)・レーンの見出し「値 (数)」・札の名前・`>` `*` `‹` `›` で探す。
//! レーンは見出しの行の縦の線(`│`)で区切られているとみなし(設計の「レーンの間は `│`」)、札がどのレーンに
//! あるかは、札の名前の桁が見出しの行のどの区画に入るかで決める。札の印は、名前の左の、同じ区画の字
//! (`>` は選んでいる札、`*` はためた変更のある札)で見る。

use super::app::{App, ColorMode};
use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, col_named, ctrl, edit_cell, edit_outside, press, read, text, Tmp};
use super::width::width;
use mdgrid::config::Config;
use mdgrid::i18n::Msg;
use mdgrid::source::markdown::Markdown;
use mdgrid::source::Value;
use mdgrid::types::Kind;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::style::Style;
use ratatui::Terminal;
use std::path::PathBuf;

/// 状態の列の決まり(CE-36)。レーンの順はこの順。
const RULES: &str = "[rules.status]\nvalues = [\"todo\", \"doing\", \"waiting\", \"done\"]\n";

/// 印の [[table]] のカンバンの設定。
const KANBAN: &str = "kanban = { column = \"status\" }";

/// KB-2 の例: todo 2件(alpha・beta)、doing 1件(gamma)、done 1件(delta)、状態なし1件(epsilon)。
/// 行は名前の順(alpha・beta・delta・epsilon・gamma)で、先頭の alpha は左の端のレーン(todo)にある。
const NOTES: &[(&str, &str)] = &[
    ("alpha.md", "---\nstatus: todo\n---\n"),
    ("beta.md", "---\nstatus: todo\n---\n"),
    ("gamma.md", "---\nstatus: doing\n---\n"),
    ("delta.md", "---\nstatus: done\n---\n"),
    ("epsilon.md", "---\ntitle: e\n---\n"),
];

/// KB-3 の例: NOTES に doing の2件目(zeta。名前の順で gamma の次)を足したもの。
const NOTES3: &[(&str, &str)] = &[
    ("alpha.md", "---\nstatus: todo\n---\n"),
    ("beta.md", "---\nstatus: todo\n---\n"),
    ("gamma.md", "---\nstatus: doing\n---\n"),
    ("zeta.md", "---\nstatus: doing\n---\n"),
    ("delta.md", "---\nstatus: done\n---\n"),
    ("epsilon.md", "---\ntitle: e\n---\n"),
];

/// 試験の札の名前(画面の中で札を探す候補)。
const NAMES: &[&str] = &[
    "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "kappa", "lambda", "lock",
];

/// 縦の線(レーンの間と枠の縦の辺)。
const SEPS: [char; 3] = ['│', '┃', '|'];

/// 枠の下の角(札の区画の終わり)。
const BOTTOM: [char; 6] = ['╰', '└', '┗', '╯', '┘', '┛'];

/// 印(根 = 一時フォルダ)の先頭に `top`(`[rules]`・`[types]` など)、[[table]] に `line` を書いて、`notes` を表として開く。
fn boot_full(
    tag: &str,
    notes: &[(&str, &str)],
    top: &str,
    line: &str,
    color: ColorMode,
    readonly: bool,
    size: (u16, u16),
) -> (Tmp, App) {
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
    let mut app = App::new(Box::new(src), color);
    app.resize(size.0, size.1);
    app.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly,
        no_color: matches!(color, ColorMode::None),
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

/// 色なし・書ける・幅 120 で開く。
fn boot(tag: &str, notes: &[(&str, &str)], top: &str, line: &str) -> (Tmp, App) {
    boot_full(tag, notes, top, line, ColorMode::None, false, (120, 30))
}

/// 決まりの順が効いていること(前提。実装の前でも通る)。
fn assert_rules(a: &App) {
    let want: Vec<String> = ["todo", "doing", "waiting", "done"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        a.prof
            .resolved
            .rules
            .get("status")
            .and_then(|r| r.values.clone()),
        Some(want),
        "前提: 印の [rules.status] が効いている"
    );
}

fn buf(a: &App) -> Buffer {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    t.backend().buffer().clone()
}

/// App の大きさで描いた画面の文字。
fn screen(a: &App) -> String {
    text(&buf(a))
}

/// 画面の行と桁で、`what` の最初の字の位置。
fn find(s: &str, what: &str) -> Option<(u16, u16)> {
    s.lines()
        .enumerate()
        .find_map(|(y, l)| l.find(what).map(|i| (width(&l[..i]) as u16, y as u16)))
}

fn w_of(c: char) -> usize {
    width(&c.to_string())
}

/// 行の中で、桁 `x` より左にある縦の線の数(= 桁 `x` の入る区画の番号)。
fn slot_at(line: &str, x: usize) -> usize {
    let mut col = 0;
    let mut n = 0;
    for c in line.chars() {
        if col >= x {
            break;
        }
        if SEPS.contains(&c) {
            n += 1;
        }
        col += w_of(c);
    }
    n
}

/// 行を縦の線で切った区画: (区画の始まりの桁, 中身)。
fn slots(line: &str) -> Vec<(usize, String)> {
    let mut out = vec![(0, String::new())];
    let mut col = 0;
    for c in line.chars() {
        col += w_of(c);
        if SEPS.contains(&c) {
            out.push((col, String::new()));
        } else {
            out.last_mut().unwrap().1.push(c);
        }
    }
    out
}

/// 見出しの区画の端の飾り(空白と、隠れたレーンの印)。
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

/// レーンの見出し: 区画の番号・値・数・値の最初の字の桁。
#[derive(Debug, Clone)]
struct Head {
    slot: usize,
    value: String,
    count: usize,
    x: usize,
}

/// レーンの見出しの行(区画の中身がどれも「値 (数)」で、2つ以上並ぶ最初の行)と、その見出し。
fn heading_row(s: &str) -> Option<(usize, Vec<Head>)> {
    for (y, line) in s.lines().enumerate() {
        let sl = slots(line);
        let filled: Vec<(usize, &(usize, String))> = sl
            .iter()
            .enumerate()
            .filter(|(_, (_, t))| !trim_head(t).is_empty())
            .collect();
        if filled.len() < 2 {
            continue;
        }
        let heads: Option<Vec<Head>> = filled
            .iter()
            .map(|(k, (x0, t))| {
                parse_head(t).map(|(value, count)| {
                    let i = t.find(&value).unwrap();
                    Head {
                        slot: *k,
                        x: x0 + width(&t[..i]),
                        value,
                        count,
                    }
                })
            })
            .collect();
        if let Some(h) = heads {
            return Some((y, h));
        }
    }
    None
}

/// 見えているレーンの (値, 数)。左から。
fn heads(s: &str) -> Vec<(String, usize)> {
    heading_row(s)
        .unwrap_or_else(|| panic!("レーンの見出しの行(「値 (数)」が縦の線で並ぶ行)が無い:\n{s}"))
        .1
        .into_iter()
        .map(|h| (h.value, h.count))
        .collect()
}

fn want(list: &[(&str, usize)]) -> Vec<(String, usize)> {
    list.iter().map(|(v, n)| (v.to_string(), *n)).collect()
}

/// 見出しの行の文字。
fn heading_line(s: &str) -> String {
    let (y, _) = heading_row(s).unwrap_or_else(|| panic!("レーンの見出しの行が無い:\n{s}"));
    s.lines().nth(y).unwrap().to_string()
}

/// 画面の札: 名前の最初の字の桁と行、名前の左の同じ区画の字(印)。
struct Card {
    x: usize,
    y: usize,
    marks: String,
}

/// 札 `name`(見出しの行より下、枠の下の辺より上で、最初に名前が出る所)。
fn card(s: &str, name: &str) -> Option<Card> {
    let (hy, _) = heading_row(s)?;
    let lines: Vec<&str> = s.lines().collect();
    let end = lines.len().saturating_sub(2);
    for (y, l) in lines.iter().enumerate().take(end).skip(hy + 1) {
        if l.contains(&BOTTOM[..]) {
            break;
        }
        if let Some(i) = l.find(name) {
            let start = l[..i]
                .char_indices()
                .rfind(|(_, c)| SEPS.contains(c))
                .map(|(j, c)| j + c.len_utf8())
                .unwrap_or(0);
            return Some(Card {
                x: width(&l[..i]),
                y,
                marks: l[start..i].to_string(),
            });
        }
    }
    None
}

fn card_or_panic(s: &str, name: &str) -> Card {
    card(s, name).unwrap_or_else(|| panic!("札 {name} が無い:\n{s}"))
}

/// 札 `name` のあるレーンの値。
fn lane_of(s: &str, name: &str) -> String {
    let (hy, hs) = heading_row(s).unwrap_or_else(|| panic!("レーンの見出しの行が無い:\n{s}"));
    let c = card_or_panic(s, name);
    let k = slot_at(s.lines().nth(hy).unwrap(), c.x);
    hs.iter()
        .find(|h| h.slot == k)
        .map(|h| h.value.clone())
        .unwrap_or_else(|| panic!("札 {name}(桁 {})の上にレーンの見出しが無い:\n{s}", c.x))
}

/// `>` の付いた札の名前(選んでいる札)。
fn selected(s: &str) -> Vec<String> {
    NAMES
        .iter()
        .filter(|n| card(s, n).is_some_and(|c| c.marks.contains('>')))
        .map(|n| n.to_string())
        .collect()
}

/// 題の行(「カンバン」と列の名前の両方がある行)。
fn title_line(s: &str, col: &str) -> String {
    above_message(s)
        .lines()
        .find(|l| l.contains("カンバン") && l.contains(col))
        .unwrap_or_else(|| panic!("題「カンバン: {col}」が無い:\n{s}"))
        .to_string()
}

/// メッセージ行(画面の一番下の行)を除いた画面の文字。理由や警告はメッセージ行に出て、表に戻っても残るので、
/// どの画面かはこの残りで見る。
fn above_message(s: &str) -> String {
    let lines: Vec<&str> = s.lines().collect();
    lines[..lines.len().saturating_sub(1)].join("\n")
}

/// カンバンの枠の題の行か(「カンバン」のすぐあとに `:` か `:`)。
fn is_kanban_title(l: &str) -> bool {
    l.match_indices("カンバン")
        .any(|(i, w)| l[i + w.len()..].trim_start().starts_with([':', ':']))
}

/// 見出し `value` の最初の字のセルの見た目。
fn head_style(a: &App, value: &str) -> Style {
    let b = buf(a);
    let s = text(&b);
    let (hy, hs) = heading_row(&s).unwrap_or_else(|| panic!("見出しの行が無い:\n{s}"));
    let h = hs
        .iter()
        .find(|h| h.value == value)
        .unwrap_or_else(|| panic!("見出し {value} が無い:\n{s}"));
    b[(h.x as u16, hy as u16)].style()
}

/// 行のノートの名前(拡張子なし)。
fn lab(a: &App, r: &mdgrid::source::RowId) -> String {
    a.src.label(r).trim_end_matches(".md").to_string()
}

/// 表で選んでいる行の名前。
fn cur(a: &App) -> String {
    lab(a, &a.cur_row().expect("表の行"))
}

/// 表の行の名前(表の順)。
fn row_names(a: &App) -> Vec<String> {
    a.rows.iter().map(|r| lab(a, r)).collect()
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
    assert!(body.contains("カンバン"), "カンバンの題が無い:\n{s}");
    assert!(
        heading_row(&body).is_some(),
        "レーンの見出しの行(「値 (数)」が縦の線で並ぶ行)が無い:\n{s}"
    );
    s
}

/// カンバンを開いたままか(表に戻っていない)。
fn assert_kanban(a: &App) {
    let s = screen(a);
    let body = above_message(&s);
    assert!(
        a.mode != Mode::Table && body.contains("カンバン") && heading_row(&body).is_some(),
        "カンバンのまま: {:?} {:?}\n{s}",
        a.mode,
        a.message
    );
}

/// 表に戻ったか(メッセージ行を除いた所で見る: ヘッダーの画面の型は表で、カンバンの枠の題とレーンの見出しの行が無い)。
fn assert_table(a: &App) {
    let s = screen(a);
    assert_eq!(a.mode, Mode::Table, "{:?}\n{s}", a.message);
    let body = above_message(&s);
    assert!(
        body.contains("表 ▾") && !body.contains("カンバン ▾"),
        "ヘッダーの画面の型は表:\n{s}"
    );
    assert!(
        !body.lines().any(is_kanban_title),
        "表に戻るとカンバンの枠の題は消える:\n{s}"
    );
    assert!(
        heading_row(&body).is_none(),
        "表に戻るとレーンの見出しは消える:\n{s}"
    );
}

/// 1行の理由が出ている(その文を返す)。
fn reason(a: &App) -> String {
    let m = a
        .message
        .clone()
        .unwrap_or_else(|| panic!("理由が出ない\n{}", screen(a)));
    assert!(
        !m.trim().is_empty() && !m.contains('\n'),
        "理由は1行: {m:?}"
    );
    m
}

/// 画面の型のメニューを開いて描いた文字(閉じて戻す)。
fn menu_text(a: &mut App) -> String {
    a.open_screen_menu();
    assert_eq!(a.mode, Mode::ScreenMenu, "{:?}", a.message);
    let s = screen(a);
    a.close_screen_menu();
    s
}

/// 札 `name` を選ぶ(左の端のレーンへ戻ってから、`l` でそのレーンへ、`j` でその札へ)。
fn select_card(a: &mut App, name: &str) {
    let target = lane_of(&screen(a), name);
    for _ in 0..10 {
        ch(a, 'h');
    }
    for _ in 0..10 {
        let s = screen(a);
        if selected(&s)
            .first()
            .is_some_and(|sel| lane_of(&s, sel) == target)
        {
            break;
        }
        ch(a, 'l');
    }
    for _ in 0..10 {
        ch(a, 'k');
    }
    for _ in 0..10 {
        if selected(&screen(a)) == [name] {
            return;
        }
        ch(a, 'j');
    }
    panic!("札 {name} を選べない:\n{}", screen(a));
}

// ---- KB-1: 設定・推定・警告 ----

#[test]
fn test_kb_1_setting_opens_with_b_and_menu_offers_kanban() {
    // [utest -> req~kanban.config-column~1, req~kanban.config-guess~1, req~kanban.config-empty-table~2] kanban = { column = "status" } の表 → 警告なし。画面の型のメニューに「カンバン」(切り替えのキー B)が
    // 出て、B で開く。題に列の名前 status があり、「推定」は無い。もう一度 B で表。メニューの「カンバン」を選んでも開く。
    let (_t, mut a) = boot("kb1set", NOTES, RULES, KANBAN);
    assert_eq!(a.message, None, "警告が無い");
    let m = menu_text(&mut a);
    assert!(m.contains("カンバン"), "窓に「カンバン」:\n{m}");
    let s = open(&mut a);
    let title = title_line(&s, "status");
    assert!(!title.contains("推定"), "設定なら推定と示さない: {title}");
    ch(&mut a, 'B');
    assert_table(&a);
    // メニューから。
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
            l.find(" カンバン")
                .filter(|_| {
                    l.trim_end()
                        .trim_end_matches(['│', '|'])
                        .trim_end()
                        .ends_with('B')
                })
                .map(|i| (width(&l[..i]) as u16 + 1, yy as u16))
        })
        .unwrap_or_else(|| panic!("窓に「カンバン」と切り替えのキー B:\n{s}"));
    a.click(x, y);
    assert_kanban(&a);
}

#[test]
fn test_kb_1_setting_column_wins_over_inference() {
    // [utest -> req~kanban.config-guess~1] 設定の列(推定の名前でない phase)で分ける。推定で選ばれる status の列があっても、設定の列が勝つ。
    let notes: &[(&str, &str)] = &[
        ("alpha.md", "---\nstatus: todo\nphase: plan\n---\n"),
        ("beta.md", "---\nstatus: done\nphase: build\n---\n"),
    ];
    let (_t, mut a) = boot("kb1phase", notes, "", "kanban = { column = \"phase\" }");
    assert_eq!(a.message, None, "警告が無い");
    let s = open(&mut a);
    let title = title_line(&s, "phase");
    assert!(!title.contains("推定"), "{title}");
    assert_eq!(heads(&s), want(&[("plan", 1), ("build", 1)]), "{s}");
    assert_eq!(lane_of(&s, "alpha"), "plan");
    assert_eq!(lane_of(&s, "beta"), "build");
}

#[test]
fn test_kb_1_infers_status_and_title_says_so() {
    // [utest -> req~kanban.config-guess~1] 設定なしで Status(大文字で始まる)の列がある → 推定して開き、題に「推定」と列の名前。
    let notes: &[(&str, &str)] = &[
        ("alpha.md", "---\nStatus: todo\n---\n"),
        ("beta.md", "---\nStatus: done\n---\n"),
    ];
    let (_t, mut a) = boot("kb1inf", notes, "", "");
    assert_eq!(a.message, None, "{:?}", a.message);
    assert!(menu_text(&mut a).contains("カンバン"));
    let s = open(&mut a);
    let title = title_line(&s, "Status");
    assert!(title.contains("推定"), "題に推定: {title}");
    assert_eq!(heads(&s), want(&[("todo", 1), ("done", 1)]), "{s}");
    assert_eq!(lane_of(&s, "alpha"), "todo");
}

#[test]
fn test_kb_1_infers_japanese_name_and_skips_list_column() {
    // [utest -> req~kanban.config-guess~1, req~kanban.config-bad-column~1] リストの型の列(stage)は推定で選ばない。stage と 状態 の列 → 状態 を推定する。
    // リストの型の stage しか無い表 → 推定できないので、B は理由1行で開かず、メニューにも出ない。
    let types = "[types]\nstage = \"list\"\n";
    let notes: &[(&str, &str)] = &[
        ("alpha.md", "---\nstage: [a, b]\n状態: 未着手\n---\n"),
        ("beta.md", "---\nstage: [c]\n状態: 完了\n---\n"),
    ];
    let (_t, mut a) = boot("kb1ja", notes, types, "");
    assert_eq!(a.col_type("stage"), Kind::List, "前提: stage はリストの型");
    let s = open(&mut a);
    let title = title_line(&s, "状態");
    assert!(title.contains("推定"), "{title}");
    assert!(!title.contains("stage"), "リストの列は選ばない: {title}");
    assert_eq!(heads(&s), want(&[("未着手", 1), ("完了", 1)]), "{s}");
    // stage しか無い表。
    let only: &[(&str, &str)] = &[
        ("alpha.md", "---\nstage: [a, b]\ntitle: x\n---\n"),
        ("beta.md", "---\nstage: [c]\ntitle: y\n---\n"),
    ];
    let (_t2, mut b) = boot("kb1list", only, types, "");
    assert_eq!(b.col_type("stage"), Kind::List, "前提: stage はリストの型");
    assert!(!menu_text(&mut b).contains("カンバン"));
    ch(&mut b, 'B');
    assert_table(&b);
    let m = reason(&b);
    assert!(m.contains("カンバン") || m.contains("kanban"), "{m}");
}

#[test]
fn test_kb_1_no_status_like_column_gives_reason() {
    // [utest -> req~kanban.config-guess-none~1, req~kanban.config-empty-table~2] 設定なしで状態らしい列が無い → B は理由1行で開かない。画面の型のメニューに「カンバン」は出ない。
    let notes: &[(&str, &str)] = &[
        ("a.md", "---\ntitle: x\npriority: p1\n---\n"),
        ("b.md", "---\ntitle: y\npriority: p2\n---\n"),
    ];
    let (_t, mut a) = boot("kb1none", notes, "", "");
    assert!(!menu_text(&mut a).contains("カンバン"));
    ch(&mut a, 'B');
    assert_table(&a);
    let m = reason(&a);
    assert!(
        m.contains("カンバン") || m.contains("kanban"),
        "何が無いかを言う: {m}"
    );
}

#[test]
fn test_kb_1_missing_column_warns_and_no_kanban() {
    // [utest -> req~kanban.config-guess-none~1, req~kanban.config-bad-column~1, req~kanban.config-empty-table~2] column = "none"(表に無い列)→ 読み終えたときに CLI-3 の形の警告1行で、カンバンを出さない。
    // 推定できる status の列があっても推定で開かず、メニューにも出ない。
    let (_t, mut a) = boot("kb1col", NOTES, RULES, "kanban = { column = \"none\" }");
    let m = a.message.clone().unwrap_or_default();
    assert!(
        m.contains(".mdgrid/workspace.toml: table[1].kanban") && m.contains("none"),
        "{m}"
    );
    assert!(!m.contains('\n'), "警告は1行: {m:?}");
    assert!(!menu_text(&mut a).contains("カンバン"));
    ch(&mut a, 'B');
    assert_table(&a);
}

#[test]
fn test_kb_1_list_column_setting_warns() {
    // [utest -> req~kanban.config-bad-column~1] 設定の列がリストの型(tags)→ CLI-3 の形で警告し、出さない(1行が複数のレーンに属さないように)。
    let types = "[types]\ntags = \"list\"\n";
    let notes: &[(&str, &str)] = &[
        ("alpha.md", "---\nstatus: todo\ntags: [x, y]\n---\n"),
        ("beta.md", "---\nstatus: done\ntags: [y]\n---\n"),
    ];
    let (_t, mut a) = boot("kb1tags", notes, types, "kanban = { column = \"tags\" }");
    assert_eq!(a.col_type("tags"), Kind::List, "前提: tags はリストの型");
    let m = a.message.clone().unwrap_or_default();
    assert!(
        m.contains(".mdgrid/workspace.toml: table[1].kanban") && m.contains("tags"),
        "{m}"
    );
    assert!(
        !m.contains(Msg::RsnUnknown.text()),
        "知らない項目ではない: {m}"
    );
    assert!(!menu_text(&mut a).contains("カンバン"));
    ch(&mut a, 'B');
    assert_table(&a);
}

#[test]
fn test_kb_1_bad_setting_shape_warns() {
    // [utest -> req~kanban.config-guess~1, req~kanban.config-bad-column~1, req~kanban.config-empty-table~2] 書き方の違う設定(column が文字でない・column が無い・表でない)→ CLI-3 の形の警告(場所は
    // table[1].kanban。「知らない項目」ではない)で、設定を使わない。推定できる列の無い表なのでカンバンは出ない。
    let notes: &[(&str, &str)] = &[
        ("a.md", "---\nphase: plan\n---\n"),
        ("b.md", "---\nphase: build\n---\n"),
    ];
    for (k, line) in [
        "kanban = { column = 3 }",
        "kanban = {}",
        "kanban = \"phase\"",
    ]
    .iter()
    .enumerate()
    {
        let (_t, mut a) = boot(&format!("kb1bad{k}"), notes, "", line);
        let m = a.message.clone().unwrap_or_default();
        assert!(
            m.contains(".mdgrid/workspace.toml: table[1].kanban"),
            "{line}: {m}"
        );
        assert!(
            !m.contains(Msg::RsnUnknown.text()),
            "{line}: kanban は知っている項目: {m}"
        );
        assert!(!m.contains('\n'), "{line}: 警告は1行: {m:?}");
        assert!(!menu_text(&mut a).contains("カンバン"), "{line}");
        ch(&mut a, 'B');
        assert_table(&a);
    }
}

// ---- KB-2: レーンと札 ----

#[test]
fn test_kb_2_lanes_follow_rule_order_with_counts() {
    // [utest -> req~kanban.open-lane-order~1] 決まり values = [todo, doing, waiting, done] で todo 2件・doing 1件・done 1件・状態なし1件 →
    // レーンは todo (2)・doing (1)・waiting (0)・done (1)・(なし) (1) の順。札は自分の状態のレーンにあり、
    // レーンの中は表の行の順(alpha が beta の上)。
    let (_t, mut a) = boot("kb2order", NOTES, RULES, KANBAN);
    assert_rules(&a);
    let s = open(&mut a);
    assert_eq!(
        heads(&s),
        want(&[
            ("todo", 2),
            ("doing", 1),
            ("waiting", 0),
            ("done", 1),
            ("(なし)", 1)
        ]),
        "{s}"
    );
    for (name, lane) in [
        ("alpha", "todo"),
        ("beta", "todo"),
        ("gamma", "doing"),
        ("delta", "done"),
        ("epsilon", "(なし)"),
    ] {
        assert_eq!(lane_of(&s, name), lane, "{name}:\n{s}");
    }
    assert!(
        card_or_panic(&s, "alpha").y < card_or_panic(&s, "beta").y,
        "表の順:\n{s}"
    );
}

#[test]
fn test_kb_2_values_outside_rules_come_before_none_in_row_order() {
    // [utest -> req~kanban.open-lane-order~1] 決まりに無い値(review・blocked)の行がある → done と (なし) の間に、行に現れた順(名前の順で
    // kappa の review が lambda の blocked より先)でレーン。名前の順(blocked・review)ではない。
    let notes: &[(&str, &str)] = &[
        ("alpha.md", "---\nstatus: todo\n---\n"),
        ("beta.md", "---\nstatus: done\n---\n"),
        ("epsilon.md", "---\ntitle: e\n---\n"),
        ("kappa.md", "---\nstatus: review\n---\n"),
        ("lambda.md", "---\nstatus: blocked\n---\n"),
    ];
    let (_t, mut a) = boot("kb2extra", notes, RULES, KANBAN);
    assert_rules(&a);
    let s = open(&mut a);
    assert_eq!(
        heads(&s),
        want(&[
            ("todo", 1),
            ("doing", 0),
            ("waiting", 0),
            ("done", 1),
            ("review", 1),
            ("blocked", 1),
            ("(なし)", 1)
        ]),
        "{s}"
    );
    assert_eq!(lane_of(&s, "kappa"), "review");
    assert_eq!(lane_of(&s, "lambda"), "blocked");
    assert_eq!(lane_of(&s, "epsilon"), "(なし)");
}

#[test]
fn test_kb_2_without_rules_lanes_follow_row_order_and_no_none_lane() {
    // [utest -> req~kanban.open-lane-order~1] 決まりの無い列 → レーンは行に現れた順(doing・todo・done。名前の順の doing・done・todo ではない)。
    // どの行にも値があるので「(なし)」のレーンは無い。
    let notes: &[(&str, &str)] = &[
        ("alpha.md", "---\nstatus: doing\n---\n"),
        ("beta.md", "---\nstatus: todo\n---\n"),
        ("delta.md", "---\nstatus: done\n---\n"),
        ("gamma.md", "---\nstatus: doing\n---\n"),
    ];
    let (_t, mut a) = boot("kb2norule", notes, "", KANBAN);
    let s = open(&mut a);
    assert_eq!(
        heads(&s),
        want(&[("doing", 2), ("todo", 1), ("done", 1)]),
        "{s}"
    );
    assert!(!s.contains("(なし)"), "{s}");
}

#[test]
fn test_kb_2_uses_view_rows_and_their_order() {
    // [utest -> req~kanban.open-lanes~1, req~kanban.open-lane-order~1] 今のビューの行(絞り込み・並べ替えのあと。TR-1 と同じ)だけを札にする。n の小さい順に並べ、
    // priority = p2 で絞る → 札は beta・alpha・delta だけで、todo のレーンは beta が alpha の上。
    // 値の無い epsilon は絞り込みで外れるので「(なし)」のレーンは無い。行の無い決まりの値もレーンにする。
    let notes: &[(&str, &str)] = &[
        ("alpha.md", "---\nstatus: todo\npriority: p2\nn: 2\n---\n"),
        ("beta.md", "---\nstatus: todo\npriority: p2\nn: 1\n---\n"),
        ("gamma.md", "---\nstatus: doing\npriority: p1\nn: 3\n---\n"),
        ("delta.md", "---\nstatus: done\npriority: p2\nn: 4\n---\n"),
        ("epsilon.md", "---\npriority: p1\nn: 5\n---\n"),
    ];
    let (_t, mut a) = boot("kb2view", notes, RULES, KANBAN);
    a.settings.sorts = vec![("n".into(), mdgrid::settings::Dir::Asc)];
    a.regrid = true;
    a.refresh_if_needed();
    assert_eq!(
        row_names(&a),
        ["beta", "alpha", "gamma", "delta", "epsilon"],
        "前提: n の順"
    );
    a.row = 0;
    col_named(&mut a, "priority");
    ch(&mut a, ',');
    assert_eq!(row_names(&a), ["beta", "alpha", "delta"], "前提: p2 で絞る");
    let s = open(&mut a);
    assert_eq!(
        heads(&s),
        want(&[("todo", 2), ("doing", 0), ("waiting", 0), ("done", 1)]),
        "{s}"
    );
    assert!(!s.contains("(なし)"), "{s}");
    assert!(
        card(&s, "gamma").is_none() && card(&s, "epsilon").is_none(),
        "{s}"
    );
    assert!(
        card_or_panic(&s, "beta").y < card_or_panic(&s, "alpha").y,
        "レーンの中は表の行の順:\n{s}"
    );
}

#[test]
fn test_kb_2_narrow_width_scrolls_lanes_and_marks_hidden_side() {
    // [utest -> req~kanban.open-lane-order~1, req~kanban.open-scroll~1, req~kanban.open-hidden-mark~1] 幅 40 で5つのレーン → 全部は入らない。選んでいるレーンがいつも見え、隠れたレーンのある側の
    // 見出しの端に印(左は `‹`、右は `›`)。開いた直後は左の端(todo)で、右に隠れたレーンがある。
    let (_t, mut a) = boot_full(
        "kb2narrow",
        NOTES,
        RULES,
        KANBAN,
        ColorMode::None,
        false,
        (40, 30),
    );
    let s = open(&mut a);
    let hs = heads(&s);
    assert!(hs.len() < 5, "幅 40 に5つは入らない: {hs:?}\n{s}");
    assert!(hs.iter().any(|(v, _)| v == "todo"), "{s}");
    let line = heading_line(&s);
    assert!(line.contains('›') && !line.contains('‹'), "{line}\n{s}");
    assert_eq!(selected(&s), ["alpha"], "{s}");
    // 右の端((なし))まで。
    for _ in 0..4 {
        ch(&mut a, 'l');
    }
    let s = screen(&a);
    let hs = heads(&s);
    assert!(hs.iter().any(|(v, _)| v == "(なし)"), "{s}");
    assert!(!hs.iter().any(|(v, _)| v == "todo"), "{s}");
    let line = heading_line(&s);
    assert!(line.contains('‹') && !line.contains('›'), "{line}\n{s}");
    assert_eq!(selected(&s), ["epsilon"], "{s}");
    // 真ん中(waiting)では両側に隠れたレーン。
    ch(&mut a, 'h');
    ch(&mut a, 'h');
    let s = screen(&a);
    assert!(heads(&s).iter().any(|(v, _)| v == "waiting"), "{s}");
    let line = heading_line(&s);
    assert!(line.contains('‹') && line.contains('›'), "{line}\n{s}");
}

#[test]
fn test_kb_2_no_color_marks_selected_card_and_lane() {
    // [utest -> req~kanban.open-no-color~1] 色なし(--no-color)でも、選んでいる札に `>`(1枚だけ)。選んでいるレーンの見出しはほかの見出しと
    // 見た目が違う。l で次のレーンへ移ると、`>` と見出しの見た目も移る。
    let (_t, mut a) = boot("kb2nocolor", NOTES, RULES, KANBAN);
    let s = open(&mut a);
    assert_eq!(selected(&s), ["alpha"], "{s}");
    assert_ne!(
        head_style(&a, "todo"),
        head_style(&a, "doing"),
        "選んでいるレーンの見出しを見分けられる:\n{s}"
    );
    ch(&mut a, 'l');
    let s = screen(&a);
    assert_eq!(selected(&s), ["gamma"], "{s}");
    assert_ne!(head_style(&a, "doing"), head_style(&a, "todo"), "{s}");
    assert_eq!(
        head_style(&a, "todo"),
        head_style(&a, "done"),
        "選んでいない(札のある)レーンの見出しは同じ見た目:\n{s}"
    );
}

#[test]
fn test_kb_2_pending_change_card_has_star() {
    // [utest -> req~kanban.open-no-color~1] 表で beta の status を doing にためる → カンバンでは beta の札がためた値のレーン(doing)にあり、
    // 色なしでも `*` で見分けられる。ためた変更の無い alpha には `*` が無い。
    let (_t, mut a) = boot("kb2star", NOTES, RULES, KANBAN);
    let i = row_names(&a).iter().position(|n| n == "beta").unwrap();
    edit_cell(&mut a, i, "status", "doing");
    assert_eq!(a.changes.count(), 1, "前提: ためた変更 {:?}", a.message);
    let s = open(&mut a);
    assert_eq!(lane_of(&s, "beta"), "doing", "{s}");
    assert!(card_or_panic(&s, "beta").marks.contains('*'), "{s}");
    assert!(!card_or_panic(&s, "alpha").marks.contains('*'), "{s}");
    assert_eq!(
        heads(&s)[..2],
        want(&[("todo", 1), ("doing", 2)])[..],
        "{s}"
    );
}

// ---- KB-3: キーとマウス ----

#[test]
fn test_kb_3_keys_move_and_enter_selects_that_row() {
    // [utest -> req~kanban.keys-move~1, req~kanban.keys-enter~1] B → l → j → Enter で、表の doing の2件目の行(zeta)が選ばれる。k で戻る。
    let (_t, mut a) = boot("kb3keys", NOTES3, RULES, KANBAN);
    let s = open(&mut a);
    assert_eq!(selected(&s), ["alpha"], "{s}");
    ch(&mut a, 'l');
    assert_eq!(selected(&screen(&a)), ["gamma"]);
    ch(&mut a, 'j');
    assert_eq!(selected(&screen(&a)), ["zeta"]);
    ch(&mut a, 'k');
    assert_eq!(selected(&screen(&a)), ["gamma"]);
    ch(&mut a, 'j');
    assert_eq!(selected(&screen(&a)), ["zeta"]);
    press(&mut a, KeyCode::Enter);
    assert_table(&a);
    assert_eq!(cur(&a), "zeta");
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_kb_3_arrow_keys_move_like_hjkl() {
    // [utest -> req~kanban.keys-move~1, req~kanban.keys-enter~1, req~kanban.keys-back~1, req~kanban.keys-click-select~1, req~kanban.keys-click-open~1, req~kanban.keys-wheel~1] → ↓ ↑ ← は l j k h と同じ。
    let (_t, mut a) = boot("kb3arrows", NOTES3, RULES, KANBAN);
    let s = open(&mut a);
    assert_eq!(selected(&s), ["alpha"], "{s}");
    press(&mut a, KeyCode::Right);
    assert_eq!(selected(&screen(&a)), ["gamma"]);
    press(&mut a, KeyCode::Down);
    assert_eq!(selected(&screen(&a)), ["zeta"]);
    press(&mut a, KeyCode::Up);
    assert_eq!(selected(&screen(&a)), ["gamma"]);
    press(&mut a, KeyCode::Left);
    assert_eq!(selected(&screen(&a)), ["alpha"]);
    press(&mut a, KeyCode::Down);
    assert_eq!(selected(&screen(&a)), ["beta"]);
    assert_kanban(&a);
}

#[test]
fn test_kb_3_esc_q_b_return_to_table() {
    // [utest -> req~kanban.keys-back~1] Esc・q・B で表に戻る(q で終わらない)。
    let (_t, mut a) = boot("kb3back", NOTES, RULES, KANBAN);
    open(&mut a);
    press(&mut a, KeyCode::Esc);
    assert_table(&a);
    open(&mut a);
    ch(&mut a, 'q');
    assert_table(&a);
    assert!(!a.quit);
    open(&mut a);
    ch(&mut a, 'B');
    assert_table(&a);
    assert!(!a.quit);
}

#[test]
fn test_kb_3_mouse_click_selects_and_second_click_opens_row() {
    // [utest -> req~kanban.keys-click-select~1, req~kanban.keys-click-open~1] 札のクリックでその札を選ぶ(カンバンのまま)。選んでいる札をもう一度クリック → 表のその行。
    let (_t, mut a) = boot("kb3click", NOTES3, RULES, KANBAN);
    let s = open(&mut a);
    let c = card_or_panic(&s, "zeta");
    a.click(c.x as u16, c.y as u16);
    assert_kanban(&a);
    assert_eq!(selected(&screen(&a)), ["zeta"], "{}", screen(&a));
    let s = screen(&a);
    let c = card_or_panic(&s, "zeta");
    a.click(c.x as u16 + 1, c.y as u16);
    assert_table(&a);
    assert_eq!(cur(&a), "zeta");
}

#[test]
fn test_kb_3_wheel_moves_card_and_hwheel_moves_lane() {
    // [utest -> req~kanban.keys-wheel~1] ホイールでレーンの中の札、横のホイールでレーンを動かす。
    let (_t, mut a) = boot("kb3wheel", NOTES3, RULES, KANBAN);
    let s = open(&mut a);
    assert_eq!(selected(&s), ["alpha"], "{s}");
    a.wheel(true);
    assert_eq!(selected(&screen(&a)), ["beta"]);
    a.wheel(false);
    assert_eq!(selected(&screen(&a)), ["alpha"]);
    a.hwheel(true);
    assert_eq!(selected(&screen(&a)), ["gamma"]);
    a.wheel(true);
    assert_eq!(selected(&screen(&a)), ["zeta"]);
    a.hwheel(false);
    assert_eq!(selected(&screen(&a)), ["alpha"]);
    assert_kanban(&a);
}

#[test]
fn test_kb_3_click_lane_heading_selects_that_lane() {
    // [utest -> req~kanban.keys-click-select~1, req~kanban.keys-wheel~1] レーンの見出しのクリックで、そのレーンを選ぶ(設計の「見出しのクリックでそのレーンを選ぶ」)。
    let (_t, mut a) = boot("kb3head", NOTES3, RULES, KANBAN);
    let s = open(&mut a);
    let (hy, hs) = heading_row(&s).unwrap();
    let done = hs.iter().find(|h| h.value == "done").unwrap();
    a.click(done.x as u16, hy as u16);
    assert_kanban(&a);
    assert_eq!(selected(&screen(&a)), ["delta"], "{}", screen(&a));
    let s = screen(&a);
    let (hy, hs) = heading_row(&s).unwrap();
    let doing = hs.iter().find(|h| h.value == "doing").unwrap();
    a.click(doing.x as u16, hy as u16);
    assert_eq!(selected(&screen(&a)), ["gamma"], "{}", screen(&a));
}

// ---- KB-4: 札を動かす ----

#[test]
fn test_kb_4_l_moves_card_then_undo_and_redo() {
    // [utest -> req~kanban.move-pending~1, req~kanban.move-keep-selection~1, req~kanban.move-redo~1] todo の札(alpha)で L → doing のレーンへ移って選んだまま、`*` が付き、下の帯の未保存が 1。
    // ノートはまだ書かない。u → todo に戻る(カンバンのまま、未保存 0)。U → また doing。
    let (t, mut a) = boot("kb4move", NOTES, RULES, KANBAN);
    let s = open(&mut a);
    assert_eq!(selected(&s), ["alpha"], "{s}");
    ch(&mut a, 'L');
    let s = screen(&a);
    assert_kanban(&a);
    assert_eq!(lane_of(&s, "alpha"), "doing", "{s}");
    assert_eq!(selected(&s), ["alpha"], "動かした先で選んだまま:\n{s}");
    assert!(card_or_panic(&s, "alpha").marks.contains('*'), "{s}");
    assert_eq!(a.changes.count(), 1);
    assert!(s.contains("未保存 1"), "{s}");
    assert_eq!(
        heads(&s)[..2],
        want(&[("todo", 1), ("doing", 2)])[..],
        "{s}"
    );
    assert_eq!(read(&t, "alpha.md"), NOTES[0].1, "ノートはまだ書かない");
    // 取り消し。
    ch(&mut a, 'u');
    let s = screen(&a);
    assert_kanban(&a);
    assert_eq!(lane_of(&s, "alpha"), "todo", "{s}");
    assert_eq!(selected(&s), ["alpha"], "{s}");
    assert!(!card_or_panic(&s, "alpha").marks.contains('*'), "{s}");
    assert_eq!(a.changes.count(), 0);
    // やり直し。
    ch(&mut a, 'U');
    let s = screen(&a);
    assert_kanban(&a);
    assert_eq!(lane_of(&s, "alpha"), "doing", "{s}");
    assert_eq!(a.changes.count(), 1);
}

#[test]
fn test_kb_4_ctrl_s_returns_to_table_and_opens_review() {
    // [utest -> req~kanban.move-save~1] L のあと Ctrl+s → 表に戻って保存の確かめ(差分)。確定でノートに書く。
    let (t, mut a) = boot("kb4save", NOTES, RULES, KANBAN);
    open(&mut a);
    ch(&mut a, 'L');
    assert_eq!(a.changes.count(), 1, "{:?}", a.message);
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Confirm, "{:?}", a.message);
    let s = screen(&a);
    assert!(
        s.contains("-status: todo") && s.contains("+status: doing"),
        "{s}"
    );
    assert_eq!(read(&t, "alpha.md"), NOTES[0].1, "確定までは書かない");
    press(&mut a, KeyCode::Enter);
    assert_eq!(read(&t, "alpha.md"), "---\nstatus: doing\n---\n");
    assert_eq!(a.changes.count(), 0);
    assert_table(&a);
}

#[test]
fn test_kb_4_h_moves_left_and_out_of_none_lane() {
    // [utest -> req~kanban.move-card~1, req~kanban.move-not-to-none~1] doing の gamma で H → todo へ。値の無い epsilon((なし) のレーン)で H → done へ動かせる(値を書く)。
    // 値の無い行が無くなれば「(なし)」のレーンは消える。
    let (_t, mut a) = boot("kb4left", NOTES, RULES, KANBAN);
    open(&mut a);
    select_card(&mut a, "gamma");
    ch(&mut a, 'H');
    let s = screen(&a);
    assert_eq!(lane_of(&s, "gamma"), "todo", "{s}");
    assert_eq!(selected(&s), ["gamma"], "{s}");
    assert_eq!(a.changes.count(), 1);
    select_card(&mut a, "epsilon");
    ch(&mut a, 'H');
    let s = screen(&a);
    assert_eq!(lane_of(&s, "epsilon"), "done", "{s}");
    assert_eq!(selected(&s), ["epsilon"], "{s}");
    assert_eq!(a.changes.count(), 2);
    assert_eq!(
        heads(&s),
        want(&[("todo", 3), ("doing", 0), ("waiting", 0), ("done", 2)]),
        "{s}"
    );
}

#[test]
fn test_kb_4_does_not_move_into_none_lane() {
    // [utest -> req~kanban.move-not-to-none~1] 右の隣が (なし) の札(done の delta)で L → 動かず理由1行。値は消さない。
    let (_t, mut a) = boot("kb4none", NOTES, RULES, KANBAN);
    open(&mut a);
    select_card(&mut a, "delta");
    a.message = None;
    ch(&mut a, 'L');
    reason(&a);
    let s = screen(&a);
    assert_kanban(&a);
    assert_eq!(lane_of(&s, "delta"), "done", "{s}");
    assert_eq!(selected(&s), ["delta"], "{s}");
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_kb_4_edges_give_reason() {
    // [utest -> req~kanban.move-not-to-none~1] 左の端のレーンの札で H、右の端のレーン((なし) の無い表の done)の札で L → 動かず理由1行。
    let (_t, mut a) = boot("kb4edgeL", NOTES, RULES, KANBAN);
    open(&mut a);
    assert_eq!(selected(&screen(&a)), ["alpha"]);
    a.message = None;
    ch(&mut a, 'H');
    reason(&a);
    assert_eq!(lane_of(&screen(&a), "alpha"), "todo");
    assert_eq!(a.changes.count(), 0);
    // (なし) の無い表。
    let (_t2, mut b) = boot("kb4edgeR", &NOTES[..4], RULES, KANBAN);
    let s = open(&mut b);
    assert!(!s.contains("(なし)"), "{s}");
    select_card(&mut b, "delta");
    b.message = None;
    ch(&mut b, 'L');
    reason(&b);
    assert_eq!(lane_of(&screen(&b), "delta"), "done");
    assert_eq!(b.changes.count(), 0);
    assert_kanban(&b);
}

#[test]
fn test_kb_4_readonly_does_not_move() {
    // [utest -> req~kanban.move-readonly~1] --readonly で開いたとき → カンバンは開けるが、L で動かず、読むだけの理由1行(WB-15 と同じ文)。
    let (_t, mut a) = boot_full(
        "kb4ro",
        NOTES,
        RULES,
        KANBAN,
        ColorMode::None,
        true,
        (120, 30),
    );
    a.message = None;
    let s = open(&mut a);
    assert_eq!(selected(&s), ["alpha"], "{s}");
    a.message = None;
    ch(&mut a, 'L');
    let m = reason(&a);
    assert_eq!(m, super::startup::READONLY.text(), "{m}");
    let s = screen(&a);
    assert_eq!(lane_of(&s, "alpha"), "todo", "{s}");
    assert_eq!(a.changes.count(), 0);
    assert_kanban(&a);
}

#[test]
fn test_kb_4_read_only_row_does_not_move() {
    // [utest -> req~kanban.move-card~1, req~kanban.move-not-to-none~1, req~kanban.move-readonly~1] 読むだけの行(BOM つきのノート。WB-5。値は読めて見せるが書かない)の札で L → 動かず、
    // 読むだけの理由1行(その行の理由を含む)。同じレーンの書けるノート(alpha)は動かせる。
    // (改行コードが混ざるノートは今の読みで status が空になり (なし) に入るので、BOM つきのノートにした。)
    let notes: &[(&str, &str)] = &[
        ("alpha.md", "---\nstatus: todo\n---\n"),
        ("lock.md", "\u{FEFF}---\nstatus: todo\n---\n"),
    ];
    let (_t, mut a) = boot("kb4lock", notes, RULES, KANBAN);
    assert_eq!(row_names(&a), ["alpha", "lock"], "前提");
    let lock = a.rows[1].clone();
    let cell = a.src.get(&lock, "status");
    assert_eq!(
        cell.value,
        Some(Value::Str("todo".into())),
        "前提: 読むだけの行も状態の値 todo を持つ"
    );
    let why = cell.lock.clone().expect("前提: lock.md は読むだけの行");
    open(&mut a);
    select_card(&mut a, "lock");
    a.message = None;
    ch(&mut a, 'L');
    let m = reason(&a);
    assert!(m.contains(Msg::ReadOnlyLead.text().trim()), "{m}");
    assert!(m.contains(&why), "その行の読むだけの理由({why}): {m}");
    let s = screen(&a);
    assert_eq!(lane_of(&s, "lock"), "todo", "{s}");
    assert_eq!(a.changes.count(), 0);
    select_card(&mut a, "alpha");
    ch(&mut a, 'L');
    assert_eq!(lane_of(&screen(&a), "alpha"), "doing");
    assert_eq!(a.changes.count(), 1);
}

// ---- KB-5: 外の変化 ----

#[test]
fn test_kb_5_external_change_moves_card_and_keeps_selection() {
    // [utest -> req~kanban.reload~1] 開いたまま外で beta の status を todo から done に書き換える → 札が done のレーンに移り、選んだまま。
    // ほかの行(gamma)が外で変わっても、選んでいる札は beta のまま。
    let (t, mut a) = boot("kb5ext", NOTES, RULES, KANBAN);
    open(&mut a);
    ch(&mut a, 'j');
    assert_eq!(selected(&screen(&a)), ["beta"]);
    edit_outside(&t, "beta.md", "---\nstatus: done\n---\n");
    a.poll();
    let s = screen(&a);
    assert_kanban(&a);
    assert_eq!(lane_of(&s, "beta"), "done", "{s}");
    assert_eq!(selected(&s), ["beta"], "{s}");
    assert_eq!(
        heads(&s),
        want(&[
            ("todo", 1),
            ("doing", 1),
            ("waiting", 0),
            ("done", 2),
            ("(なし)", 1)
        ]),
        "{s}"
    );
    edit_outside(&t, "gamma.md", "---\nstatus: waiting\n---\n");
    a.poll();
    let s = screen(&a);
    assert_eq!(lane_of(&s, "gamma"), "waiting", "{s}");
    assert_eq!(selected(&s), ["beta"], "{s}");
    assert_eq!(a.changes.count(), 0);
}

// [utest -> req~cell-edit.then-kanban~1]
#[test]
fn test_then_fills_the_other_column_when_a_card_moves_to_the_lane() {
    // todo の札を done のレーンまで動かすと、その行の done_at に今日がたまり、取り消し1回で前のレーンに戻って done_at も外れる。
    let top = format!("{RULES}\n[rules.status.then]\ndone = {{ done_at = \"{{date}}\" }}\n");
    let (_t, mut a) = boot("then-kanban", NOTES, &top, KANBAN);
    a.today = mdgrid::types::parse_date("2026-10-06").unwrap();
    let s = open(&mut a);
    assert_eq!(selected(&s), ["alpha"], "{s}");
    for _ in 0..3 {
        ch(&mut a, 'L');
    }
    assert_eq!(lane_of(&screen(&a), "alpha"), "done", "{}", screen(&a));
    let row = a.changes.rows().pop().unwrap();
    assert_eq!(
        a.changes.pending(&row, "done_at"),
        Some(&mdgrid::writeback::NewValue::Date("2026-10-06".into()))
    );
    ch(&mut a, 'u');
    assert_eq!(lane_of(&screen(&a), "alpha"), "waiting", "{}", screen(&a));
    assert_eq!(a.changes.pending(&row, "done_at"), None);
}
