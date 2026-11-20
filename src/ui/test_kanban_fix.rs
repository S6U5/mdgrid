//! カンバンのレビューの指摘(#1〜#9)の直しの試験。記録: specs/_history/decidespec/_changes/2026-11-02-kanban.md の「レビューの指摘」と
//! 「実装の気づき」の「直し:」。値は作ったもの。錠のある試験のファイルは変えず、補助は test_kanban.rs から写した。
//!
//! 直す前の版で落ち、直した後の版で通る期待にする。まだ無い関数・文言の名前は使わず、キーとマウスを送り、画面の
//! 文字と今ある口(`mode`・`message`・`changes`・`cur_row()`・`click`・`resize`・`poll`・`apply`・`gantt` の欄)で見る。
//! レーンは見出しの行の縦の線(`│`)で区切られているとみなし、札がどのレーンにあるかは、札の名前の桁が見出しの行の
//! どの区画に入るかで決める。札の印は名前の左の同じ区画の字(`>` は選んでいる札)で見る。

use super::app::{App, ColorMode};
use super::grid::Slot;
use super::keymap::{Action, Mode};
use super::startup::Startup;
use super::test_screen::{ch, col_named, edit_outside, press, text, typing, Tmp};
use super::width::width;
use mdgrid::config::Config;
use mdgrid::settings::{Dir, Group};
use mdgrid::source::markdown::Markdown;
use mdgrid::source::NewValue;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;
use std::path::PathBuf;

/// 状態の列の決まり(CE-36)。レーンの順はこの順。
const RULES: &str = "[rules.status]\nvalues = [\"todo\", \"doing\", \"waiting\", \"done\"]\n";

/// 印の [[table]] のカンバンの設定。
const KANBAN: &str = "kanban = { column = \"status\" }";

/// todo 2件(alpha・beta)、doing 1件(gamma)、done 1件(delta)、状態なし1件(epsilon)。
/// レーンは todo・doing・waiting・done・(なし)。
const NOTES: &[(&str, &str)] = &[
    ("alpha.md", "---\nstatus: todo\n---\n"),
    ("beta.md", "---\nstatus: todo\n---\n"),
    ("gamma.md", "---\nstatus: doing\n---\n"),
    ("delta.md", "---\nstatus: done\n---\n"),
    ("epsilon.md", "---\ntitle: e\n---\n"),
];

/// 試験の札の名前(画面の中で札を探す候補)。
const NAMES: &[&str] = &["alpha", "beta", "gamma", "delta", "epsilon"];

/// 縦の線(レーンの間と枠の縦の辺)。
const SEPS: [char; 3] = ['│', '┃', '|'];

/// 枠の下の角(札の区画の終わり)。
const BOTTOM: [char; 6] = ['╰', '└', '┗', '╯', '┘', '┛'];

/// 印(根 = 一時フォルダ)の先頭に `top`、[[table]] に `line` を書いて `notes` を表として起動する(読み込みはしない)。
fn start_app(tag: &str, notes: &[(&str, &str)], top: &str, line: &str) -> (Tmp, App) {
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
    (tmp, app)
}

/// 起動して読み終えるまで読む(色なし・幅 120)。
fn boot(tag: &str, notes: &[(&str, &str)], top: &str, line: &str) -> (Tmp, App) {
    let (tmp, mut app) = start_app(tag, notes, top, line);
    finish_loading(&mut app);
    (tmp, app)
}

fn finish_loading(a: &mut App) {
    while !a.loaded() {
        a.load_step(100);
    }
}

/// App の大きさで描いた画面の文字。
fn screen(a: &App) -> String {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    text(t.backend().buffer())
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

/// 行を縦の線で切った区画の中身。
fn slots(line: &str) -> Vec<String> {
    let mut out = vec![String::new()];
    for c in line.chars() {
        if SEPS.contains(&c) {
            out.push(String::new());
        } else {
            out.last_mut().unwrap().push(c);
        }
    }
    out
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

/// レーンの見出しの (区画の番号, 値, 数)。
type HeadCell = (usize, String, usize);

/// レーンの見出しの行(区画の中身がどれも「値 (数)」で2つ以上並ぶ最初の行)と、(区画の番号, 値, 数)。
fn heading_row(s: &str) -> Option<(usize, Vec<HeadCell>)> {
    for (y, line) in s.lines().enumerate() {
        let sl = slots(line);
        let filled: Vec<(usize, &String)> = sl
            .iter()
            .enumerate()
            .filter(|(_, t)| !trim_head(t).is_empty())
            .collect();
        if filled.len() < 2 {
            continue;
        }
        let heads: Option<Vec<(usize, String, usize)>> = filled
            .iter()
            .map(|(k, t)| parse_head(t).map(|(v, n)| (*k, v, n)))
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
        .unwrap_or_else(|| panic!("レーンの見出しの行が無い:\n{s}"))
        .1
        .into_iter()
        .map(|(_, v, n)| (v, n))
        .collect()
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
        .find(|h| h.0 == k)
        .map(|h| h.1.clone())
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

/// 画面の文字の、空でない最後の行の番号。描画は画面の高さより1行少なく描くので、一番下の行はいつも空。
/// 理由や警告が出ていれば、この行がメッセージ行(出ていなければ下の帯)。
fn last_filled(lines: &[&str]) -> usize {
    lines
        .iter()
        .rposition(|l| !l.trim().is_empty())
        .unwrap_or(0)
}

/// メッセージ行(空でない最後の行)より上の画面の文字。理由や警告はメッセージ行に出て表に戻っても残るので、
/// どの画面かはこの残りで見る(メッセージが無ければ下の帯も除くが、画面の型の確かめには使わない)。
fn above_message(s: &str) -> String {
    let lines: Vec<&str> = s.lines().collect();
    lines[..last_filled(&lines)].join("\n")
}

/// 画面の型の題の行(`word` のすぐあとに `:` か `:`。メッセージ行は除く)。
fn title_of(s: &str, word: &str) -> Option<String> {
    above_message(s)
        .lines()
        .find(|l| {
            l.match_indices(word)
                .any(|(i, w)| l[i + w.len()..].trim_start().starts_with([':', ':']))
        })
        .map(str::to_string)
}

/// 行のノートの名前(拡張子なし)。
fn lab(a: &App, r: &mdgrid::source::RowId) -> String {
    a.src.label(r).trim_end_matches(".md").to_string()
}

/// 表で選んでいる行の名前。
fn cur(a: &App) -> String {
    lab(a, &a.cur_row().expect("表の行"))
}

/// 名前の行。
fn row_of(a: &App, name: &str) -> mdgrid::source::RowId {
    a.rows
        .iter()
        .find(|r| lab(a, r) == name)
        .cloned()
        .unwrap_or_else(|| panic!("行 {name} が無い"))
}

/// 名前の行が表に見えているか(畳んだ親の子・畳んだまとまりの行は見えない)。
fn shown_in_table(a: &App, name: &str) -> bool {
    let i = a.rows.iter().position(|r| lab(a, r) == name).unwrap();
    a.slots.contains(&Slot::Row(i))
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

/// 表に戻ったか(メッセージ行を除いた所で見る: ヘッダーの画面の型は表で、カンバンの枠の題とレーンの見出しが無い)。
fn assert_table(a: &App) {
    let s = screen(a);
    assert_eq!(a.mode, Mode::Table, "{:?}\n{s}", a.message);
    let body = above_message(&s);
    assert!(
        body.contains("表 ▾") && !body.contains("カンバン ▾"),
        "ヘッダーの画面の型は表:\n{s}"
    );
    assert!(title_of(&s, "カンバン").is_none(), "{s}");
    assert!(heading_row(&body).is_none(), "{s}");
}

/// 1行の理由が出ている(メッセージ行にも出ている)。その文を返す。
fn reason(a: &App) -> String {
    let s = screen(a);
    let m = a
        .message
        .clone()
        .unwrap_or_else(|| panic!("理由が出ない\n{s}"));
    assert!(
        !m.trim().is_empty() && !m.contains('\n'),
        "理由は1行: {m:?}"
    );
    // メッセージ行(空でない最後の行)に理由の頭が出ている(幅で切られても頭は見える)。
    let lines: Vec<&str> = s.lines().collect();
    let shown = lines[last_filled(&lines)];
    let head: String = m.trim().chars().take(6).collect();
    assert!(
        shown.contains(&head),
        "メッセージ行に理由 {m:?}: {shown:?}\n{s}"
    );
    m
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

/// パレットで動作の名前を打って選ぶ。
fn palette(a: &mut App, name: &str) {
    ch(a, ':');
    assert_eq!(a.mode, Mode::Palette, "{:?}", a.message);
    typing(a, name);
    press(a, KeyCode::Enter);
}

/// ガントで選んでいる行の名前。
fn gantt_sel(a: &App) -> String {
    let g = a.gantt.as_ref().expect("ガント");
    lab(a, &g.rows[g.sel])
}

// ---- #1: 大きさを変えたあとのクリック ----

#[test]
fn test_kb_3_click_after_resize_hits_the_drawn_lane() {
    // [utest -> req~kanban.keys-move~1, req~kanban.keys-enter~1, req~kanban.keys-click-select~1, req~kanban.keys-click-open~1] #1: 幅 120 で5つのレーンを見せて (なし) を選び、幅 40 に縮める → 描画は done と (なし)。
    // 見えている done の札(delta)のクリックはその札を選び(隠れた todo のレーンに当たらない)、もう一度で表の delta。
    let (_t, mut a) = boot("kbfix1", NOTES, RULES, KANBAN);
    let s = open(&mut a);
    assert_eq!(heads(&s).len(), 5, "前提: 幅 120 で5つ見える:\n{s}");
    for _ in 0..4 {
        ch(&mut a, 'l');
    }
    assert_eq!(selected(&screen(&a)), ["epsilon"]);
    a.resize(40, 30);
    let s = screen(&a);
    let hs = heads(&s);
    assert!(
        hs.iter().any(|(v, _)| v == "done") && !hs.iter().any(|(v, _)| v == "todo"),
        "前提: 幅 40 で done と (なし) が見える: {hs:?}\n{s}"
    );
    let c = card_or_panic(&s, "delta");
    a.click(c.x as u16, c.y as u16);
    assert_kanban(&a);
    let s = screen(&a);
    assert_eq!(selected(&s), ["delta"], "見えている done の札を選ぶ:\n{s}");
    let c = card_or_panic(&s, "delta");
    a.click(c.x as u16 + 1, c.y as u16);
    assert_eq!(a.mode, Mode::Table, "{:?}", a.message);
    assert_eq!(cur(&a), "delta");
    // 幅 40 ではヘッダーの「表 ▾」のボタンを省くので、幅を戻してから表の画面を確かめる。
    a.resize(120, 30);
    assert_table(&a);
    assert_eq!(cur(&a), "delta");
}

// ---- #2: 表で選べない札で Enter・パレットの動作 ----

#[test]
fn test_kb_3_enter_on_empty_lane_stays_with_reason() {
    // [utest -> req~kanban.keys-move~1, req~kanban.keys-enter~1, req~kanban.keys-wheel~1] #2: 札の無いレーン(waiting)で Enter → 表で選べる行が無いので、カンバンのまま理由1行。
    let (_t, mut a) = boot("kbfix2e", NOTES, RULES, KANBAN);
    open(&mut a);
    ch(&mut a, 'l');
    ch(&mut a, 'l');
    assert!(selected(&screen(&a)).is_empty(), "前提: waiting は空");
    a.message = None;
    press(&mut a, KeyCode::Enter);
    assert_kanban(&a);
    reason(&a);
}

#[test]
fn test_kb_3_palette_action_on_empty_lane_does_not_touch_table_row() {
    // [utest -> req~kanban.keys-move~1, req~kanban.keys-enter~1] #2: 札の無いレーンでパレットから「空にする」(clear)→ 開く前の表の行(alpha の status)に書かない。
    // カンバンのまま理由1行で、ためた変更は無い。
    let (_t, mut a) = boot("kbfix2p", NOTES, RULES, KANBAN);
    col_named(&mut a, "status");
    assert_eq!(cur(&a), "alpha", "前提: 表の今の行");
    open(&mut a);
    ch(&mut a, 'l');
    ch(&mut a, 'l');
    assert!(selected(&screen(&a)).is_empty(), "前提: waiting は空");
    a.message = None;
    palette(&mut a, "clear");
    assert_eq!(
        a.changes.count(),
        0,
        "表の別の行に書かない: {:?}",
        a.message
    );
    assert_kanban(&a);
    reason(&a);
}

/// 親 alpha(todo)の下に beta(doing)と gamma(done)。表で alpha を畳むと beta・gamma は表に見えない。
const TREE_NOTES: &[(&str, &str)] = &[
    ("alpha.md", "---\nstatus: todo\n---\n"),
    (
        "beta.md",
        "---\nparent: \"[[alpha]]\"\nstatus: doing\n---\n",
    ),
    (
        "gamma.md",
        "---\nparent: \"[[alpha]]\"\nstatus: done\n---\n",
    ),
];

/// 親子で並べて、表で alpha を畳み、status の列を選ぶ。
fn folded(tag: &str) -> (Tmp, App) {
    let (t, mut a) = boot(tag, TREE_NOTES, RULES, KANBAN);
    a.settings.tree = Some("parent".into());
    a.regrid = true;
    a.refresh_if_needed();
    assert_eq!(cur(&a), "alpha", "前提: 表の今の行は親");
    ch(&mut a, 'Z');
    assert!(
        !shown_in_table(&a, "beta") && !shown_in_table(&a, "gamma"),
        "前提: 畳んだ親の子は表に見えない: {:?}",
        a.message
    );
    col_named(&mut a, "status");
    (t, a)
}

#[test]
fn test_kb_3_enter_on_card_hidden_in_table_stays_with_reason() {
    // [utest -> req~kanban.keys-enter~1, req~kanban.keys-click-select~1] #2: 表で畳んだ親の子(beta)の札で Enter → 表でその行を選べないので、カンバンのまま理由1行。
    let (_t, mut a) = folded("kbfix2f");
    open(&mut a);
    select_card(&mut a, "beta");
    a.message = None;
    press(&mut a, KeyCode::Enter);
    assert_kanban(&a);
    reason(&a);
    assert_eq!(selected(&screen(&a)), ["beta"]);
}

#[test]
fn test_kb_3_palette_action_on_card_hidden_in_table_does_not_touch_other_row() {
    // [utest -> req~kanban.keys-enter~1] #2: 表で畳んだ親の子(beta)の札でパレットから clear → 表の今の行(親 alpha)の status を空にしない。
    // カンバンのまま理由1行で、ためた変更は無い。
    let (_t, mut a) = folded("kbfix2fp");
    open(&mut a);
    select_card(&mut a, "beta");
    a.message = None;
    palette(&mut a, "clear");
    assert_eq!(
        a.changes.count(),
        0,
        "表の別の行(alpha)に書かない: {:?}",
        a.message
    );
    assert_kanban(&a);
    reason(&a);
}

/// ガントの見本: beta だけ done。status でまとめて done のまとまりを畳むと、beta は表に見えない。
const GANTT_NOTES: &[(&str, &str)] = &[
    (
        "alpha.md",
        "---\nstart: 2026-10-01\ndue: 2026-10-03\nstatus: todo\n---\n",
    ),
    (
        "beta.md",
        "---\nstart: 2026-10-02\ndue: 2026-10-04\nstatus: done\n---\n",
    ),
    (
        "gamma.md",
        "---\nstart: 2026-10-03\ndue: 2026-10-05\nstatus: todo\n---\n",
    ),
];

/// status でまとめ、done のまとまりを畳み、表で alpha の status を選んでからガントを開き、beta を選ぶ。
fn gantt_on_hidden_row(tag: &str) -> (Tmp, App) {
    let (t, mut a) = boot(
        tag,
        GANTT_NOTES,
        "",
        "gantt = { start = \"start\", end = \"due\" }",
    );
    a.settings.group = Group::By {
        col: "status".into(),
        dir: Dir::Asc,
        hide_empty: false,
    };
    a.refresh();
    let head = a
        .slots
        .iter()
        .position(|s| *s == Slot::Head(0))
        .expect("前提: まとまりの見出し");
    a.row = head;
    press(&mut a, KeyCode::Enter);
    assert!(
        !shown_in_table(&a, "beta") && shown_in_table(&a, "alpha"),
        "前提: done のまとまりを畳むと beta は表に見えない"
    );
    let ai = a.rows.iter().position(|r| lab(&a, r) == "alpha").unwrap();
    a.row = a.slots.iter().position(|s| *s == Slot::Row(ai)).unwrap();
    col_named(&mut a, "status");
    assert_eq!(cur(&a), "alpha", "前提: 表の今の行");
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    for _ in 0..4 {
        if gantt_sel(&a) == "beta" {
            break;
        }
        ch(&mut a, 'k');
    }
    for _ in 0..4 {
        if gantt_sel(&a) == "beta" {
            break;
        }
        ch(&mut a, 'j');
    }
    assert_eq!(
        gantt_sel(&a),
        "beta",
        "前提: ガントでは畳んだまとまりの行も選べる"
    );
    (t, a)
}

#[test]
fn test_ga_4_enter_on_row_hidden_in_table_stays_with_reason() {
    // [utest -> req~gantt.keys-mouse-click~1] #2: 表で畳んだまとまりの行(beta)をガントで選んで Enter → 表でその行を選べないので、ガントのまま理由1行。
    let (_t, mut a) = gantt_on_hidden_row("gafix2e");
    a.message = None;
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Gantt, "ガントのまま: {:?}", a.message);
    reason(&a);
    assert_eq!(gantt_sel(&a), "beta");
}

#[test]
fn test_ga_4_palette_action_on_row_hidden_in_table_does_not_touch_other_row() {
    // [utest -> req~gantt.keys-today~1, req~gantt.keys-mouse-click~1, req~gantt.keys-mouse-click-again~1] #2: 表で畳んだまとまりの行(beta)をガントで選んでパレットから clear → 表の今の行(alpha)の status を
    // 空にしない。ガントのまま理由1行で、ためた変更は無い。
    let (_t, mut a) = gantt_on_hidden_row("gafix2p");
    a.message = None;
    palette(&mut a, "clear");
    assert_eq!(
        a.changes.count(),
        0,
        "表の別の行(alpha)に書かない: {:?}",
        a.message
    );
    assert_eq!(a.mode, Mode::Gantt, "ガントのまま: {:?}", a.message);
    reason(&a);
}

// ---- #3: 動かした札の元のレーンの位置 ----

#[test]
fn test_kb_4_move_keeps_the_position_in_the_source_lane() {
    // [utest -> req~kanban.move-card~1] #3: todo に alpha・beta・gamma。beta(2枚目)を L で doing へ動かし、h で todo へ戻る → todo の選びは
    // 同じ位置(2枚目)の gamma(先頭の alpha に戻らない)。
    let notes: &[(&str, &str)] = &[
        ("alpha.md", "---\nstatus: todo\n---\n"),
        ("beta.md", "---\nstatus: todo\n---\n"),
        ("gamma.md", "---\nstatus: todo\n---\n"),
        ("delta.md", "---\nstatus: doing\n---\n"),
    ];
    let (_t, mut a) = boot("kbfix3", notes, RULES, KANBAN);
    open(&mut a);
    select_card(&mut a, "beta");
    ch(&mut a, 'L');
    let s = screen(&a);
    assert_eq!(lane_of(&s, "beta"), "doing", "{s}");
    assert_eq!(selected(&s), ["beta"], "{s}");
    ch(&mut a, 'h');
    let s = screen(&a);
    assert_eq!(selected(&s), ["gamma"], "元のレーンは同じ位置:\n{s}");
}

// ---- #4: 列の型に合わない値 ----

#[test]
fn test_kb_4_value_not_fitting_column_type_gives_reason() {
    // [utest -> req~kanban.move-not-to-none~1] #4: 数の列 priority(決まりの values に "high")で、2 の札を L で high のレーンへ → 表の入力なら断る値
    // なので動かず理由1行、ためた変更は無い。1 の札を L で 2 のレーンへは動かせる。
    let top =
        "[types]\npriority = \"number\"\n\n[rules.priority]\nvalues = [\"1\", \"2\", \"high\"]\n";
    let notes: &[(&str, &str)] = &[
        ("alpha.md", "---\npriority: 1\n---\n"),
        ("beta.md", "---\npriority: 2\n---\n"),
    ];
    let (_t, mut a) = boot("kbfix4", notes, top, "kanban = { column = \"priority\" }");
    let s = open(&mut a);
    assert_eq!(
        heads(&s),
        [
            ("1".to_string(), 1),
            ("2".to_string(), 1),
            ("high".to_string(), 0)
        ],
        "前提:\n{s}"
    );
    select_card(&mut a, "beta");
    a.message = None;
    ch(&mut a, 'L');
    assert_eq!(
        a.changes.count(),
        0,
        "型に合わない値をためない: {:?}",
        a.message
    );
    reason(&a);
    let s = screen(&a);
    assert_eq!(lane_of(&s, "beta"), "2", "{s}");
    // 型に合う値へは動かせる。
    select_card(&mut a, "alpha");
    ch(&mut a, 'L');
    assert_eq!(lane_of(&screen(&a), "alpha"), "2");
    assert_eq!(a.changes.count(), 1);
}

// ---- #5: キーの名前の変更・削除をためた列 ----

const TWO: &[(&str, &str)] = &[
    ("alpha.md", "---\nstatus: todo\n---\n"),
    ("beta.md", "---\nstatus: doing\n---\n"),
];

#[test]
fn test_kb_4_pending_key_rename_blocks_move() {
    // [utest -> req~kanban.move-pending~1, req~kanban.move-not-to-none~1] #5: 表で status のキーの名前を phase に変える(ためる。CE-29)→ カンバンで alpha の札を H → 札は動かず、
    // メッセージ行に理由1行、ためた名前の変更は残る(値の書き換えで上書きしない)。
    let (_t, mut a) = boot("kbfix5r", TWO, RULES, KANBAN);
    col_named(&mut a, "status");
    a.apply(Action::RenameKey);
    assert_eq!(a.mode, Mode::Palette, "{:?}", a.message);
    typing(&mut a, "phase");
    press(&mut a, KeyCode::Enter);
    let alpha = row_of(&a, "alpha");
    let rename = NewValue::RenameKey("phase".into());
    assert_eq!(
        a.changes.pending(&alpha, "status"),
        Some(&rename),
        "前提: 名前の変更をためた"
    );
    let n = a.changes.count();
    let s = open(&mut a);
    let before = lane_of(&s, "alpha");
    select_card(&mut a, "alpha");
    a.message = None;
    ch(&mut a, 'H');
    assert_eq!(
        a.changes.pending(&alpha, "status"),
        Some(&rename),
        "ためた名前の変更が残る: {:?}",
        a.message
    );
    assert_eq!(a.changes.count(), n);
    let s = screen(&a);
    assert_eq!(lane_of(&s, "alpha"), before, "札は動かない:\n{s}");
    reason(&a);
}

#[test]
fn test_kb_4_pending_key_delete_blocks_move() {
    // [utest -> req~kanban.move-not-to-none~1] #5: 表で status のキーを消す(ためる。CE-29)→ カンバンで alpha の札を H → 札は動かず、
    // メッセージ行に理由1行、ためたキーの削除は残る。
    let (_t, mut a) = boot("kbfix5d", TWO, RULES, KANBAN);
    col_named(&mut a, "status");
    a.apply(Action::DeleteKey);
    assert_eq!(a.mode, Mode::Palette, "{:?}", a.message);
    typing(&mut a, "y");
    press(&mut a, KeyCode::Enter);
    let alpha = row_of(&a, "alpha");
    assert_eq!(
        a.changes.pending(&alpha, "status"),
        Some(&NewValue::DeleteKey),
        "前提: キーの削除をためた"
    );
    let n = a.changes.count();
    let s = open(&mut a);
    let before = lane_of(&s, "alpha");
    select_card(&mut a, "alpha");
    a.message = None;
    ch(&mut a, 'H');
    assert_eq!(
        a.changes.pending(&alpha, "status"),
        Some(&NewValue::DeleteKey),
        "ためたキーの削除が残る: {:?}",
        a.message
    );
    assert_eq!(a.changes.count(), n);
    let s = screen(&a);
    assert_eq!(lane_of(&s, "alpha"), before, "札は動かない:\n{s}");
    reason(&a);
}

// ---- #6: 読み込みの途中で開いたカンバン ----

/// 40 のノート(偶数は todo、奇数は done)。
fn forty() -> Vec<(String, String)> {
    (0..40)
        .map(|i| {
            let st = if i % 2 == 0 { "todo" } else { "done" };
            (format!("n{i:02}.md"), format!("---\nstatus: {st}\n---\n"))
        })
        .collect()
}

#[test]
fn test_kb_2_kanban_opened_while_loading_is_rebuilt_when_loading_ends() {
    // [utest -> req~kanban.open-lanes~1] #6: 読み込みの途中(40 のうち一部)で B → 読み終えたら、カンバンを組み直して全部の行(40)を札にする。
    let owned = forty();
    let notes: Vec<(&str, &str)> = owned
        .iter()
        .map(|(n, t)| (n.as_str(), t.as_str()))
        .collect();
    let (_t, mut a) = start_app("kbfix6", &notes, "", KANBAN);
    a.load_step(10);
    assert!(
        !a.loaded() && !a.rows.is_empty() && a.rows.len() < 40,
        "前提: 読み込みの途中({} 行)",
        a.rows.len()
    );
    let s = open(&mut a);
    let partial: usize = heads(&s).iter().map(|(_, n)| n).sum();
    assert!(partial < 40, "前提: 途中の札 {partial}");
    finish_loading(&mut a);
    assert_eq!(a.rows.len(), 40, "前提: 読み終えた");
    assert_kanban(&a);
    let s = screen(&a);
    let total: usize = heads(&s).iter().map(|(_, n)| n).sum();
    assert_eq!(total, 40, "読み終えたら全部の行を札に:\n{s}");
}

#[test]
fn test_kb_1_kanban_opened_while_loading_closes_when_setting_turns_off() {
    // [utest -> req~kanban.config-bad-column~1, req~kanban.config-empty-table~1] #6: 設定の列が表に無い(column = "none")表で、読み込みの途中に B で開く → 読み終えて列が無いと分かったら、
    // 警告1行(CLI-3 の形)を出し、開いているカンバンを閉じて表に戻る。
    let owned = forty();
    let notes: Vec<(&str, &str)> = owned
        .iter()
        .map(|(n, t)| (n.as_str(), t.as_str()))
        .collect();
    let (_t, mut a) = start_app("kbfix6off", &notes, "", "kanban = { column = \"none\" }");
    a.load_step(10);
    assert!(!a.loaded() && !a.rows.is_empty(), "前提: 読み込みの途中");
    ch(&mut a, 'B');
    // 途中では設定の列で開く。レーンは (なし) の1つだけなので、見出しの行でなく枠の題で見る。
    let s = screen(&a);
    assert!(
        a.mode != Mode::Table && title_of(&s, "カンバン").is_some_and(|l| l.contains("none")),
        "前提: 途中では設定の列で開く: {:?}\n{s}",
        a.message
    );
    finish_loading(&mut a);
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains("table[1].kanban") && m.contains("none"), "{m}");
    assert_table(&a);
}

// ---- #8: 設定の列が外の変化で消えたとき ----

#[test]
fn test_kb_1_setting_column_gone_does_not_fall_back_to_inference() {
    // [utest -> req~kanban.config-column~1, req~kanban.config-guess~1, req~kanban.config-guess-none~1, req~kanban.config-bad-column~1, req~kanban.config-empty-table~1] #8: 設定 column = "phase" で開いたまま、外で全部のノートから phase を消す → 推定(status)に落ちない:
    // 開いたままなら題は phase のままで「推定」と示さない。表に戻って B でも、推定の status のカンバンは開かない。
    let notes: &[(&str, &str)] = &[
        ("alpha.md", "---\nstatus: todo\nphase: plan\n---\n"),
        ("beta.md", "---\nstatus: done\nphase: build\n---\n"),
    ];
    let (t, mut a) = boot("kbfix8", notes, "", "kanban = { column = \"phase\" }");
    assert_eq!(a.message, None);
    let s = open(&mut a);
    assert!(
        title_of(&s, "カンバン").is_some_and(|l| l.contains("phase")),
        "{s}"
    );
    edit_outside(&t, "alpha.md", "---\nstatus: todo\n---\n");
    edit_outside(&t, "beta.md", "---\nstatus: done\n---\n");
    a.poll();
    assert!(
        !a.src.columns().iter().any(|c| c == "phase"),
        "前提: phase の列が消えた"
    );
    let not_inferred = |a: &App, when: &str| {
        let s = screen(a);
        if a.mode == Mode::Table {
            return;
        }
        let title =
            title_of(&s, "カンバン").unwrap_or_else(|| panic!("{when}: カンバンの題が無い:\n{s}"));
        assert!(
            title.contains("phase") && !title.contains("推定"),
            "{when}: 推定に落ちない: {title}\n{s}"
        );
    };
    not_inferred(&a, "開いたまま");
    if a.mode != Mode::Table {
        press(&mut a, KeyCode::Esc);
    }
    assert_eq!(a.mode, Mode::Table);
    ch(&mut a, 'B');
    not_inferred(&a, "表から B");
}

#[test]
fn test_ga_1_setting_column_gone_does_not_fall_back_to_inference() {
    // [utest -> req~gantt.config-start-end~1, req~gantt.config-days~1, req~gantt.config-infer-fail~1, req~gantt.config-missing-column~1, req~gantt.config-empty-table~1] #8: 設定 start = "kickoff" で開いたまま、外で全部のノートから kickoff を消す → 推定(start・due)に
    // 落ちない: 開いたままなら題は kickoff のままで「推定」と示さない。表に戻って C でも、推定のガントは開かない。
    let notes: &[(&str, &str)] = &[
        (
            "alpha.md",
            "---\nkickoff: 2026-10-01\nstart: 2026-10-02\ndue: 2026-10-05\n---\n",
        ),
        (
            "beta.md",
            "---\nkickoff: 2026-10-03\nstart: 2026-10-04\ndue: 2026-10-06\n---\n",
        ),
    ];
    let (t, mut a) = boot(
        "gafix8",
        notes,
        "",
        "gantt = { start = \"kickoff\", end = \"due\" }",
    );
    assert_eq!(a.message, None);
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    let s = screen(&a);
    assert!(
        title_of(&s, "ガント").is_some_and(|l| l.contains("kickoff") && !l.contains("推定")),
        "前提:\n{s}"
    );
    edit_outside(
        &t,
        "alpha.md",
        "---\nstart: 2026-10-02\ndue: 2026-10-05\n---\n",
    );
    edit_outside(
        &t,
        "beta.md",
        "---\nstart: 2026-10-04\ndue: 2026-10-06\n---\n",
    );
    a.poll();
    assert!(
        !a.src.columns().iter().any(|c| c == "kickoff"),
        "前提: kickoff の列が消えた"
    );
    let not_inferred = |a: &App, when: &str| {
        let s = screen(a);
        if a.mode == Mode::Table {
            return;
        }
        let title =
            title_of(&s, "ガント").unwrap_or_else(|| panic!("{when}: ガントの題が無い:\n{s}"));
        assert!(
            title.contains("kickoff") && !title.contains("推定"),
            "{when}: 推定に落ちない: {title}\n{s}"
        );
    };
    not_inferred(&a, "開いたまま");
    if a.mode != Mode::Table {
        press(&mut a, KeyCode::Esc);
    }
    assert_eq!(a.mode, Mode::Table);
    ch(&mut a, 'C');
    not_inferred(&a, "表から C");
}

// ---- #9: 見本なしの空の台帳 ----

#[test]
fn test_kb_1_empty_ledger_does_not_warn_and_kanban_opens_after_adding_note() {
    // [utest -> req~kanban.config-empty-table~1] #9: 行が0件の台帳(kanban = { column = "status" })→ 読み終えても「列が無い」と警告しない。
    // 同じ回で外からノートを足すと、B でカンバンが開く。
    let (t, mut a) = boot("kbfix9", &[], RULES, KANBAN);
    assert!(a.rows.is_empty(), "前提: 0件");
    let m = a.message.clone().unwrap_or_default();
    assert!(!m.contains("table[1].kanban"), "空の台帳で警告しない: {m}");
    t.write("first.md", "---\nstatus: todo\n---\n");
    a.poll();
    assert_eq!(a.rows.len(), 1, "前提: 足したノートが行になる");
    let s = open(&mut a);
    assert!(heads(&s).contains(&("todo".to_string(), 1)), "{s}");
}

#[test]
fn test_ga_1_empty_ledger_does_not_warn_and_gantt_opens_after_adding_note() {
    // [utest -> req~gantt.config-empty-table~1] #9: 行が0件の台帳(gantt = { start, end })→ 読み終えても「列が無い」と警告しない。
    // 同じ回で外からノートを足すと、C でガントが開く。
    let (t, mut a) = boot(
        "gafix9",
        &[],
        "",
        "gantt = { start = \"start\", end = \"due\" }",
    );
    assert!(a.rows.is_empty(), "前提: 0件");
    let m = a.message.clone().unwrap_or_default();
    assert!(!m.contains("table[1].gantt"), "空の台帳で警告しない: {m}");
    t.write("first.md", "---\nstart: 2026-10-01\ndue: 2026-10-03\n---\n");
    a.poll();
    assert_eq!(a.rows.len(), 1, "前提: 足したノートが行になる");
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
}

#[test]
fn test_tr_2_empty_ledger_does_not_warn_and_tree_opens_after_adding_note() {
    // [utest -> req~tree.config-empty-table~1] #9: 行が0件の台帳(tree = { path = "folder" })→ 読み終えても「列が無い」と警告しない。
    // 同じ回で外からノートを足すと、T で木が開く。
    let (t, mut a) = boot("trfix9", &[], "", "tree = { path = \"folder\" }");
    assert!(a.rows.is_empty(), "前提: 0件");
    let m = a.message.clone().unwrap_or_default();
    assert!(!m.contains("table[1].tree"), "空の台帳で警告しない: {m}");
    t.write("first.md", "---\nfolder: x > y\n---\n");
    a.poll();
    assert_eq!(a.rows.len(), 1, "前提: 足したノートが行になる");
    ch(&mut a, 'T');
    assert_eq!(a.mode, Mode::Tree, "{:?}", a.message);
}

// ---- #7: キーの文書(任意) ----

#[test]
fn test_sr_22_screen_menu_doc_names_gantt_and_kanban_keys() {
    // [utest -> req~screen.keys-docs-files~1, req~screen.keys-docs-by-mode~1] #7: キーの文書(docs/keys.md・keys.ja.md)の `screen_menu` の節に、ガントの `C` とカンバンの `B` がある。
    for file in ["docs/keys.md", "docs/keys.ja.md"] {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(file);
        let text = std::fs::read_to_string(&p).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        let start = lines
            .iter()
            .position(|l| l.trim_end() == "## `screen_menu`")
            .unwrap_or_else(|| panic!("{file} に screen_menu の節が無い"));
        let body: String = lines[start + 1..]
            .iter()
            .take_while(|l| !l.starts_with("## "))
            .filter(|l| !l.starts_with('|'))
            .copied()
            .collect::<Vec<_>>()
            .join("\n");
        for key in ["`C`", "`B`"] {
            assert!(
                body.contains(key),
                "{file} の screen_menu の節に {key} が無い:\n{body}"
            );
        }
    }
}
