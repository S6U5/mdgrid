//! カンバンのセキュリティの回の指摘 #23 の直しの試験。記録: specs/_history/decidespec/_changes/2026-11-02-kanban.md の「レビューの指摘」。
//! 値は作ったもの。錠のある試験のファイルは変えず、補助は test_kanban_fix2.rs・test_kanban_fix4.rs から写した。
//!
//! 直す前の版で落ち、直した後の版で通る期待にする。実装の前の版(4bc740c)に無い名前は使わず、キーを送り、画面の
//! 文字と今ある口(`mode`・`message`・`changes`・`rows`・`load_step`)で見る。
//!
//! 印の設定の列を `note.` 付きで書いた表(`kanban = { column = "note.status" }`)を `.base` で開くと、札のレーンは
//! `note.` を外した列(status)の値で分かれる。表の列は status で、`note.status` という列は無い。読み終えたあとは
//! 列が無いと断るが、読み込みの途中は列を確かめないので、`B` → `L` で `(行, "note.status")` に値がたまり、保存で
//! front matter に新しいキーが足される。

use super::app::{App, ColorMode};
use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, text, Tmp};
use mdgrid::config::Config;
use mdgrid::source::{NewValue, RowId};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

/// ビューが1つの `.base`(絞り込みなし)。
const BASE: &str = "views:\n  - type: table\n    name: 全部\n    order: [status]\n";

/// 縦の線(レーンの間と枠の縦の辺)。
const SEPS: [char; 3] = ['│', '┃', '|'];

/// レーンの見出しの (区画の番号, 値, 数)。
type HeadCell = (usize, String, usize);

/// 40 のノート(status は todo・doing・done の順に繰り返す)。読み込みの途中でも3つの値の行がある。
fn forty() -> Vec<(String, String)> {
    (0..40)
        .map(|i| {
            let st = ["todo", "doing", "done"][i % 3];
            (format!("n{i:02}.md"), format!("---\nstatus: {st}\n---\n"))
        })
        .collect()
}

/// 40 のノートと `notes/tasks.base` を置き、印(根 = 一時フォルダ)の [[table]] でその `.base` に `line` を書いて、
/// main と同じ道筋(`open_target`)で `.base` を開く(読み込みはしない)。
fn start_base(tag: &str, line: &str) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    for (n, t) in forty() {
        tmp.write(&n, &t);
    }
    tmp.write("tasks.base", BASE);
    std::fs::create_dir_all(tmp.0.join(".mdgrid")).unwrap();
    std::fs::write(
        tmp.0.join(".mdgrid/workspace.toml"),
        format!("[[table]]\nname = \"T\"\npath = \"notes/tasks.base\"\n{line}\n"),
    )
    .unwrap();
    let p = tmp.notes().join("tasks.base");
    let t = crate::open_target(std::slice::from_ref(&p), None).unwrap();
    let mut app = App::new(Box::new(t.src), ColorMode::None);
    app.resize(120, 30);
    app.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: p,
        base: t.base,
    });
    (tmp, app)
}

/// 一部だけ読んで、読み込みの途中にする。
fn partly_loaded(a: &mut App) {
    a.load_step(10);
    assert!(
        !a.loaded() && !a.rows.is_empty() && a.rows.len() < 40,
        "前提: 読み込みの途中({} 行)",
        a.rows.len()
    );
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

/// メッセージ行(空でない最後の行。描画は高さより1行少なく描くので一番下の行はいつも空)より上の画面の文字。
fn above_message(s: &str) -> String {
    let lines: Vec<&str> = s.lines().collect();
    let last = lines
        .iter()
        .rposition(|l| !l.trim().is_empty())
        .unwrap_or(0);
    lines[..last].join("\n")
}

/// 見えているレーンの値(左から)。
fn lane_values(s: &str) -> Vec<String> {
    heading_row(&above_message(s))
        .unwrap_or_else(|| panic!("レーンの見出しの行が無い:\n{s}"))
        .into_iter()
        .map(|(_, v, _)| v)
        .collect()
}

/// 1行の理由(その文を返す)。
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

/// `col` の列にためた変更のある行の (名前, ためた値)。
fn pending_on(a: &App, col: &str) -> Vec<(String, NewValue)> {
    a.rows
        .iter()
        .filter_map(|r: &RowId| {
            a.changes.pending(r, col).map(|v| {
                (
                    a.src.label(r).trim_end_matches(".md").to_string(),
                    v.clone(),
                )
            })
        })
        .collect()
}

/// カンバンの左の端のレーンへ移る。
fn to_first_lane(a: &mut App) {
    for _ in 0..6 {
        ch(a, 'h');
    }
}

#[test]
fn test_kb_4_setting_column_not_in_table_does_not_move_card_while_loading() {
    // [utest -> req~kanban.move-card~1, req~kanban.move-pending~1, req~kanban.move-readonly~1, req~kanban.config-column~1, req~kanban.config-bad-column~1, req~kanban.config-empty-table~2] #23: 印の設定の列が表の列に無い名前(`note.status`。表の列は status)の `.base` を、読み込みの途中に
    // B で開いて札を L で右のレーンへ動かす → 値をためず(未保存の変更は 0、`note.status` のキーへの変更も無い)、
    // 列が無いことを言う理由1行(列の名前 note.status を言う)を出す。読み終えたあとに B で断るのと同じく、表に無い
    // 列に書かない。直しが B の時点で断る形でもよい(そのときは B の理由で見る)。
    let (_t, mut a) = start_base("kbfx23ns", "kanban = { column = \"note.status\" }");
    partly_loaded(&mut a);
    assert!(
        a.src.columns().iter().any(|c| c == "status")
            && !a.src.columns().iter().any(|c| c == "note.status"),
        "前提: 表の列は status で、note.status は無い: {:?}",
        a.src.columns()
    );
    ch(&mut a, 'B');
    let m = if a.mode == Mode::Table {
        reason(&a)
    } else {
        to_first_lane(&mut a);
        a.message = None;
        ch(&mut a, 'L');
        assert_eq!(
            a.changes.count(),
            0,
            "L で表に無い列 note.status に値をためない: {:?} {:?}\n{}",
            pending_on(&a, "note.status"),
            a.message,
            screen(&a)
        );
        reason(&a)
    };
    assert!(
        pending_on(&a, "note.status").is_empty(),
        "note.status のキーへの変更が無い: {:?}",
        pending_on(&a, "note.status")
    );
    assert!(
        m.contains("note.status"),
        "列が無いことを言う理由(列の名前 note.status): {m}"
    );
    // 隣のレーンへ移って H(左へ動かす)でも、ためない。
    ch(&mut a, 'l');
    ch(&mut a, 'H');
    assert_eq!(
        a.changes.count(),
        0,
        "H でも値をためない: {:?} {:?}",
        pending_on(&a, "note.status"),
        a.message
    );
}

#[test]
fn test_kb_4_setting_column_in_table_moves_card_while_loading() {
    // [utest -> req~kanban.move-card~1, req~kanban.move-pending~1] #23 の対照: 印の設定の列が表の列にある名前(`status`)の `.base` なら、読み込みの途中に B で開いて、
    // 左の端のレーンの札を L で右の隣のレーンへ動かすと、今までどおり status に隣のレーンの値をためる(未保存の
    // 変更は 1)。レーンの順は行に現れた順(読み込みの順)なので、見出しから読む。
    let (_t, mut a) = start_base("kbfx23st", "kanban = { column = \"status\" }");
    partly_loaded(&mut a);
    ch(&mut a, 'B');
    assert_ne!(
        a.mode,
        Mode::Table,
        "前提: 途中でも設定の列で開く: {:?}\n{}",
        a.message,
        screen(&a)
    );
    to_first_lane(&mut a);
    let lanes = lane_values(&screen(&a));
    let states = ["todo", "doing", "done"];
    assert!(
        lanes.len() >= 2
            && states.contains(&lanes[0].as_str())
            && states.contains(&lanes[1].as_str()),
        "前提: 左の2つのレーンは状態の値: {lanes:?}\n{}",
        screen(&a)
    );
    ch(&mut a, 'L');
    assert_eq!(
        a.changes.count(),
        1,
        "L で値をためる: {:?}\n{}",
        a.message,
        screen(&a)
    );
    let moved = pending_on(&a, "status");
    assert_eq!(moved.len(), 1, "status に1つためる: {moved:?}");
    assert_eq!(
        moved[0].1,
        NewValue::Str(lanes[1].clone()),
        "動かした先のレーン {} の値: {moved:?}",
        lanes[1]
    );
    // 名前の番号から元の status(番号を 3 で割った余りで todo・doing・done)。
    let n: usize = moved[0].0[1..].parse().unwrap();
    assert_eq!(
        states[n % 3],
        lanes[0],
        "動かしたのは左の端のレーンの札: {moved:?}"
    );
    assert!(pending_on(&a, "note.status").is_empty());
}
