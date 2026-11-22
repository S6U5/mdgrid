//! カンバンのレビュー2回目の指摘(#10〜#13)の直しの試験。記録: specs/_history/decidespec/_changes/2026-11-02-kanban.md の
//! 「レビューの指摘」と「実装の気づき」の「2回目の直し:」。値は作ったもの。錠のある試験のファイルは変えず、補助は
//! test_kanban_fix.rs から写した。
//!
//! 直す前の版で落ち、直した後の版で通る期待にする。実装の前の版(4bc740c)に無い名前は使わず、キーとパレットを
//! 送り、画面の文字と今ある口(`mode`・`quit`・`message`・`changes`・`poll`・`apply`・`gantt` の欄)で見る。
//! レーンは見出しの行の縦の線(`│`)で区切られているとみなし、札がどのレーンにあるかは、札の名前の桁が見出しの行の
//! どの区画に入るかで決める。札の印は名前の左の同じ区画の字(`>` は選んでいる札)で見る。

use super::app::{App, ColorMode};
use super::grid::Slot;
use super::keymap::{Action, Mode};
use super::startup::Startup;
use super::test_screen::{ch, col_named, press, text, typing, Tmp};
use super::width::width;
use mdgrid::config::Config;
use mdgrid::settings::{Dir, Group};
use mdgrid::source::markdown::Markdown;
use mdgrid::source::{NewValue, RowId};
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

/// todo 2件(alpha・beta)、doing 1件(gamma)、done 1件(delta)、状態なし1件(epsilon)。どれも日付を持つ
/// (ガントにも切り替えられる)。レーンは todo・doing・waiting(空)・done・(なし)。
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
        "---\nstart: 2026-10-05\ndue: 2026-10-07\n---\n",
    ),
];

/// 試験の札の名前(画面の中で札を探す候補)。
const NAMES: &[&str] = &["alpha", "beta", "gamma", "delta", "epsilon"];

/// 縦の線(レーンの間と枠の縦の辺)。
const SEPS: [char; 3] = ['│', '┃', '|'];

/// 枠の下の角(札の区画の終わり)。
const BOTTOM: [char; 6] = ['╰', '└', '┗', '╯', '┘', '┛'];

/// レーンの見出しの (区画の番号, 値, 数)。
type HeadCell = (usize, String, usize);

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

/// レーンの見出しの行(区画の中身がどれも「値 (数)」で2つ以上並ぶ最初の行)と、その見出し。
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
        let heads: Option<Vec<HeadCell>> = filled
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

/// 画面の札: 名前の最初の字の桁、名前の左の同じ区画の字(印)。
struct Card {
    x: usize,
    marks: String,
}

/// 札 `name`(見出しの行より下、枠の下の辺より上で、最初に名前が出る所)。
fn card(s: &str, name: &str) -> Option<Card> {
    let (hy, _) = heading_row(s)?;
    let lines: Vec<&str> = s.lines().collect();
    let end = lines.len().saturating_sub(2);
    for l in lines.iter().take(end).skip(hy + 1) {
        if l.contains(&BOTTOM[..]) {
            break;
        }
        if let Some(i) = l.find(name) {
            let start = l[..i]
                .char_indices()
                .rfind(|(_, c)| SEPS.contains(c))
                .map_or(0, |(j, c)| j + c.len_utf8());
            return Some(Card {
                x: width(&l[..i]),
                marks: l[start..i].to_string(),
            });
        }
    }
    None
}

/// 札 `name` のあるレーンの値。
fn lane_of(s: &str, name: &str) -> String {
    let (hy, hs) = heading_row(s).unwrap_or_else(|| panic!("レーンの見出しの行が無い:\n{s}"));
    let c = card(s, name).unwrap_or_else(|| panic!("札 {name} が無い:\n{s}"));
    let k = slot_at(s.lines().nth(hy).unwrap(), c.x);
    hs.iter().find(|h| h.0 == k).map_or_else(
        || panic!("札 {name}(桁 {})の上にレーンの見出しが無い:\n{s}", c.x),
        |h| h.1.clone(),
    )
}

/// `>` の付いた札の名前(選んでいる札)。
fn selected(s: &str) -> Vec<String> {
    NAMES
        .iter()
        .filter(|n| card(s, n).is_some_and(|c| c.marks.contains('>')))
        .map(std::string::ToString::to_string)
        .collect()
}

/// 画面の文字の、空でない最後の行の番号(描画は高さより1行少なく描くので一番下の行はいつも空)。
fn last_filled(lines: &[&str]) -> usize {
    lines
        .iter()
        .rposition(|l| !l.trim().is_empty())
        .unwrap_or(0)
}

/// メッセージ行(空でない最後の行)より上の画面の文字。
fn above_message(s: &str) -> String {
    let lines: Vec<&str> = s.lines().collect();
    lines[..last_filled(&lines)].join("\n")
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

/// カンバンで札の無いレーン(waiting)を選ぶ。
fn select_empty_lane(a: &mut App) {
    for _ in 0..10 {
        ch(a, 'h');
    }
    ch(a, 'l');
    ch(a, 'l');
    let s = screen(a);
    assert!(
        selected(&s).is_empty() && heads(&s).contains(&("waiting".to_string(), 0)),
        "前提: 札の無い waiting のレーンを選ぶ:\n{s}"
    );
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

/// 親子で並べて表で alpha を畳み、カンバンを開いて、表に見えない beta の札を選ぶ。
fn kanban_on_hidden_card(tag: &str) -> (Tmp, App) {
    let (t, mut a) = boot(tag, TREE_NOTES, RULES, KANBAN);
    a.settings.tree = Some("parent".into());
    a.regrid = true;
    a.refresh_if_needed();
    assert_eq!(cur(&a), "alpha", "前提: 表の今の行は親");
    ch(&mut a, 'Z');
    assert!(
        !shown_in_table(&a, "beta"),
        "前提: 畳んだ親の子は表に見えない: {:?}",
        a.message
    );
    open(&mut a);
    select_card(&mut a, "beta");
    (t, a)
}

/// status でまとめて done のまとまりを畳み、表で alpha を選んでからガントを開き、表に見えない delta を選ぶ。
fn gantt_on_hidden_row(tag: &str) -> (Tmp, App) {
    let (t, mut a) = boot(tag, &NOTES[..4], "", GANTT);
    a.settings.group = Group::By {
        col: "status".into(),
        dir: Dir::Asc,
        hide_empty: false,
    };
    a.refresh();
    // delta(done)のあるまとまりの見出しで Enter(畳む)。
    let di = a.rows.iter().position(|r| lab(&a, r) == "delta").unwrap();
    let g = a
        .groups
        .iter()
        .position(|(_, r)| r.contains(&di))
        .expect("前提: delta のまとまり");
    let head = a
        .slots
        .iter()
        .position(|s| *s == Slot::Head(g))
        .expect("前提: まとまりの見出し");
    a.row = head;
    press(&mut a, KeyCode::Enter);
    assert!(
        !shown_in_table(&a, "delta") && shown_in_table(&a, "alpha"),
        "前提: done のまとまりを畳むと delta は表に見えない"
    );
    let ai = a.rows.iter().position(|r| lab(&a, r) == "alpha").unwrap();
    a.row = a.slots.iter().position(|s| *s == Slot::Row(ai)).unwrap();
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    for _ in 0..6 {
        if gantt_sel(&a) == "delta" {
            break;
        }
        ch(&mut a, 'k');
    }
    for _ in 0..6 {
        if gantt_sel(&a) == "delta" {
            break;
        }
        ch(&mut a, 'j');
    }
    assert_eq!(
        gantt_sel(&a),
        "delta",
        "前提: ガントでは畳んだまとまりの行も選べる"
    );
    (t, a)
}

// ---- #10: 表で選べない札・行から、行に効かない動作 ----

#[test]
fn test_sr_14_palette_quit_from_empty_kanban_lane_quits() {
    // [utest -> req~screen.palette-fuzzy~2, req~screen.palette-keys~1] #10: カンバンの札の無いレーン(waiting)で、パレットから `q`(終わる)→ 札の行は要らない動作なので、
    // 止めずに効く(ためた変更が無いので終わる)。
    let (_t, mut a) = boot("kbfx10q", NOTES, RULES, KANBAN);
    open(&mut a);
    select_empty_lane(&mut a);
    palette(&mut a, "q");
    assert!(
        a.quit,
        "終わる: {:?} {:?}\n{}",
        a.mode,
        a.message,
        screen(&a)
    );
}

#[test]
fn test_sr_14_palette_switch_screen_from_empty_kanban_lane() {
    // [utest -> req~screen.palette-keys~1] #10: カンバンの札の無いレーンで、パレットから画面の型の切り替え(gantt_view)→ 止めずにガントになる。
    let (_t, mut a) = boot("kbfx10sw", NOTES, RULES, KANBAN);
    open(&mut a);
    select_empty_lane(&mut a);
    palette(&mut a, "gantt_view");
    assert_eq!(
        a.mode,
        Mode::Gantt,
        "ガントに切り替わる: {:?}\n{}",
        a.message,
        screen(&a)
    );
}

#[test]
fn test_sr_14_palette_search_from_card_hidden_in_table_opens_search() {
    // [utest -> req~screen.palette-fuzzy~2, req~screen.palette-keys~1] #10: 表で畳んだ親の子(beta)の札で、パレットから検索(search)→ 札の行は要らない動作なので、
    // 表に戻って検索の入力が開く。
    let (_t, mut a) = kanban_on_hidden_card("kbfx10s");
    palette(&mut a, "search");
    assert_eq!(
        a.mode,
        Mode::Search,
        "検索の入力が開く: {:?}\n{}",
        a.message,
        screen(&a)
    );
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_sr_14_palette_quit_from_gantt_row_hidden_in_table_quits() {
    // [utest -> req~screen.palette-fuzzy~2, req~screen.palette-keys~1] #10: ガントで表の畳んだまとまりの行(delta)を選び、パレットから `q` → 止めずに効く(終わる)。
    let (_t, mut a) = gantt_on_hidden_row("gafx10q");
    palette(&mut a, "q");
    assert!(
        a.quit,
        "終わる: {:?} {:?}\n{}",
        a.mode,
        a.message,
        screen(&a)
    );
}

#[test]
fn test_sr_14_palette_search_from_gantt_row_hidden_in_table_opens_search() {
    // [utest -> req~screen.palette-fuzzy~2] #10: ガントで表の畳んだまとまりの行(delta)を選び、パレットから検索 → 表に戻って検索の入力が開く。
    let (_t, mut a) = gantt_on_hidden_row("gafx10s");
    palette(&mut a, "search");
    assert_eq!(
        a.mode,
        Mode::Search,
        "検索の入力が開く: {:?}\n{}",
        a.message,
        screen(&a)
    );
}

// ---- #11: キーの名前の変更をためた状態の列 ----

/// alpha(todo)・beta(doing)と、状態の無い epsilon。
const RENAME_NOTES: &[(&str, &str)] = &[
    ("alpha.md", "---\nstatus: todo\n---\n"),
    ("beta.md", "---\nstatus: doing\n---\n"),
    ("epsilon.md", "---\ntitle: e\n---\n"),
];

/// 表で status のキーの名前を phase に変える(ためる。CE-29)。status の無い epsilon は飛ばされる。
fn rename_status(a: &mut App) {
    col_named(a, "status");
    a.apply(Action::RenameKey);
    assert_eq!(a.mode, Mode::Palette, "{:?}", a.message);
    typing(a, "phase");
    press(a, KeyCode::Enter);
    let rename = NewValue::RenameKey("phase".into());
    assert_eq!(
        a.changes.pending(&row_of(a, "alpha"), "status"),
        Some(&rename),
        "前提: 名前の変更をためた"
    );
    assert_eq!(
        a.changes.pending(&row_of(a, "epsilon"), "status"),
        None,
        "前提: status の無い行は飛ばす"
    );
}

#[test]
fn test_kb_2_pending_key_rename_keeps_cards_in_their_value_lanes() {
    // [utest -> req~kanban.open-lanes~1, req~kanban.open-lane-order~2] #11: status のキーの名前の変更をためても、「→ phase」のレーンはできず、札は元の値のレーン
    // (alpha は todo、beta は doing)にある。
    let (_t, mut a) = boot("kbfx11l", RENAME_NOTES, RULES, KANBAN);
    rename_status(&mut a);
    let s = open(&mut a);
    let hs = heads(&s);
    assert!(
        !hs.iter().any(|(v, _)| v.starts_with('→')),
        "「→ …」のレーンを作らない: {hs:?}\n{s}"
    );
    assert_eq!(lane_of(&s, "alpha"), "todo", "{s}");
    assert_eq!(lane_of(&s, "beta"), "doing", "{s}");
}

#[test]
fn test_kb_4_card_is_not_moved_into_a_pending_rename_value() {
    // [utest -> req~kanban.move-card~1, req~kanban.move-not-to-none~1] #11: status のキーの名前の変更をためた表で、状態の無い epsilon の札((なし))を H で左へ動かしても、
    // 「→ phase」の値をためない(ためるのは決まりの値)。
    let (_t, mut a) = boot("kbfx11m", RENAME_NOTES, RULES, KANBAN);
    rename_status(&mut a);
    open(&mut a);
    select_card(&mut a, "epsilon");
    ch(&mut a, 'H');
    let eps = row_of(&a, "epsilon");
    let pending = a.changes.pending(&eps, "status").cloned();
    assert!(
        !matches!(&pending, Some(NewValue::Str(v)) if v.starts_with('→')),
        "「→ …」の値をためない: {pending:?} {:?}\n{}",
        a.message,
        screen(&a)
    );
}

// ---- #12: 空の台帳にノートを足したとき ----

/// 空の台帳を読み終え、外で設定の列の無いノートを足して poll したあとのメッセージ。
fn warn_after_first_note(tag: &str, line: &str) -> String {
    let (t, mut a) = boot(tag, &[], "", line);
    assert!(a.rows.is_empty(), "前提: 0件");
    t.write("first.md", "---\ntitle: x\n---\n");
    a.poll();
    assert_eq!(a.rows.len(), 1, "前提: 足したノートが行になる");
    a.message.clone().unwrap_or_default()
}

#[test]
fn test_kb_1_column_missing_warns_when_first_note_is_added() {
    // [utest -> req~kanban.config-empty-table~2] #12: 見本なしの空の台帳(kanban = { column = "status" })で読み終えたあと、status の無いノートを外で
    // 足す → そのとき CLI-3 の形で列が無いと警告する(「設定が無い」ではない)。
    let m = warn_after_first_note("kbfx12", KANBAN);
    assert!(
        m.contains("table[1].kanban") && m.contains("status"),
        "列が無いと警告: {m:?}"
    );
    assert!(!m.contains("設定が無"), "{m}");
}

#[test]
fn test_ga_1_column_missing_warns_when_first_note_is_added() {
    // [utest -> req~gantt.config-empty-table~1] #12: 見本なしの空の台帳(gantt = { start, end })で、start の無いノートを外で足す → そのとき警告する。
    let m = warn_after_first_note("gafx12", GANTT);
    assert!(
        m.contains("table[1].gantt") && m.contains("start"),
        "列が無いと警告: {m:?}"
    );
    assert!(!m.contains("設定が無"), "{m}");
}

#[test]
fn test_tr_2_column_missing_warns_when_first_note_is_added() {
    // [utest -> req~tree.config-empty-table~1] #12: 見本なしの空の台帳(tree = { path = "folder" })で、folder の無いノートを外で足す → そのとき警告する。
    let m = warn_after_first_note("trfx12", "tree = { path = \"folder\" }");
    assert!(
        m.contains("table[1].tree") && m.contains("folder"),
        "列が無いと警告: {m:?}"
    );
    assert!(!m.contains("設定が無"), "{m}");
}

// ---- #13: 読み込みの途中で開いたガント・木 ----

/// 40 のノート(日付つき)。
fn forty() -> Vec<(String, String)> {
    (0..40)
        .map(|i| {
            (
                format!("n{i:02}.md"),
                format!(
                    "---\nstatus: todo\nstart: 2026-10-01\ndue: 2026-10-0{}\n---\n",
                    2 + i % 7
                ),
            )
        })
        .collect()
}

/// 読み込みの途中(40 のうち一部)で `key` を押して `mode` の画面を開き、読み終える。
fn open_while_loading(tag: &str, line: &str, key: char, mode: Mode) -> (Tmp, App) {
    let owned = forty();
    let notes: Vec<(&str, &str)> = owned
        .iter()
        .map(|(n, t)| (n.as_str(), t.as_str()))
        .collect();
    let (t, mut a) = start_app(tag, &notes, "", line);
    a.load_step(10);
    assert!(!a.loaded() && !a.rows.is_empty(), "前提: 読み込みの途中");
    ch(&mut a, key);
    assert_eq!(
        a.mode, mode,
        "前提: 途中では設定の列で開く: {:?}",
        a.message
    );
    finish_loading(&mut a);
    (t, a)
}

#[test]
fn test_ga_1_gantt_opened_while_loading_closes_when_column_missing() {
    // [utest -> req~gantt.config-missing-column~1, req~gantt.config-empty-table~1] #13: 設定の列が表に無い(start = "none")表で、読み込みの途中に C でガントを開く → 読み終えて列が無いと
    // 分かったら、警告(CLI-3 の形)を出し、開いているガントを閉じて表に戻る。
    let (_t, a) = open_while_loading(
        "gafx13",
        "gantt = { start = \"none\", end = \"due\" }",
        'C',
        Mode::Gantt,
    );
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains("table[1].gantt") && m.contains("none"), "{m}");
    assert_eq!(a.mode, Mode::Table, "ガントを閉じる:\n{}", screen(&a));
    assert!(a.gantt.is_none(), "ガントの状態も残さない");
}

#[test]
fn test_tr_2_tree_opened_while_loading_closes_when_column_missing() {
    // [utest -> req~tree.config-invalid~2, req~tree.config-empty-table~1] #13: 設定の列が表に無い(path = "none")表で、読み込みの途中に T で木を開く → 読み終えて列が無いと
    // 分かったら、警告(CLI-3 の形)を出し、開いている木を閉じて表に戻る。
    let (_t, a) = open_while_loading("trfx13", "tree = { path = \"none\" }", 'T', Mode::Tree);
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains("table[1].tree") && m.contains("none"), "{m}");
    assert_eq!(a.mode, Mode::Table, "木を閉じる:\n{}", screen(&a));
    assert!(a.folder_tree.is_none(), "木の状態も残さない");
}
