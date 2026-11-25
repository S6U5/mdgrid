#![cfg(unix)]
//! カンバンの札の値と名前の折り返し(kanban.card-*)の、本物の実行ファイルでの確かめ(端末 80×24)。値は作ったもの。
//! 実装の前に書いた。行番号や字の位置は決め打ちせず、札の名前の出る所を起点に、同じレーンの字を下へ読む。

use crate::pty;

use pty::{assert_gone, TempDir, Tui};
use unicode_width::UnicodeWidthStr;

/// 印に状態の列の決まりと、[[table]] に `line` を書いた根。`notes` は t/ の下に置く。
fn root(tag: &str, line: &str, notes: &[(String, String)]) -> TempDir {
    let home = TempDir::new(tag);
    let h = home.path();
    let w = |rel: &str, text: &str| {
        let p = h.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    };
    w(
        ".mdgrid/workspace.toml",
        &format!(
            "[rules.status]\nvalues = [\"todo\", \"doing\", \"done\"]\n\n\
             [[table]]\nname = \"T\"\npath = \"t\"\n{line}\n"
        ),
    );
    for (n, t) in notes {
        w(&format!("t/{n}"), t);
    }
    std::fs::create_dir_all(h.join("state")).unwrap();
    home
}

fn note(name: &str, text: &str) -> (String, String) {
    (name.to_string(), text.to_string())
}

/// 縦の線か。
fn is_sep(c: char) -> bool {
    matches!(c, '│' | '┃' | '|')
}

/// 行の桁 `from` から、桁 `to` の手前までの字。
fn span(line: &str, from: usize, to: usize) -> String {
    let mut col = 0;
    let mut out = String::new();
    for c in line.chars() {
        let w = UnicodeWidthStr::width(c.to_string().as_str());
        if col >= from && col + w <= to {
            out.push(c);
        }
        col += w;
    }
    out
}

/// 行の桁 `x` より右で最初の縦の線の桁。
fn right_sep(line: &str, x: usize) -> usize {
    let mut col = 0;
    for c in line.chars() {
        if col >= x && is_sep(c) {
            return col;
        }
        col += UnicodeWidthStr::width(c.to_string().as_str());
    }
    col
}

/// カンバンの画面(題「カンバン」とレーンの見出し todo・doing が1行に)。
fn is_kanban(s: &str) -> bool {
    s.contains("カンバン")
        && s.lines()
            .any(|l| l.contains("todo (") && l.contains("doing ("))
}

/// 札 `name` の名前の最初の字の (桁, 行)。レーンの見出しより下で最初に名前が出る所。
fn card_at(s: &str, name: &str) -> Option<(usize, usize)> {
    let lines: Vec<&str> = s.lines().collect();
    let head = lines
        .iter()
        .position(|l| l.contains("todo (") && l.contains("doing ("))?;
    lines
        .iter()
        .enumerate()
        .skip(head + 1)
        .find_map(|(y, l)| l.find(name).map(|i| (UnicodeWidthStr::width(&l[..i]), y)))
}

/// 札 `name` の頭の行から下へ `n` 行の、同じレーンの字(名前の最初の字の桁から。末尾の空白を除く)。
fn rows_from(s: &str, name: &str, n: usize) -> Vec<String> {
    let Some((x, y)) = card_at(s, name) else {
        return Vec::new();
    };
    let lines: Vec<&str> = s.lines().collect();
    let end = right_sep(lines[y], x);
    lines[y..(y + n).min(lines.len())]
        .iter()
        .map(|l| span(l, x, end).trim_end().to_string())
        .collect()
}

/// 札 `name` の左の2桁に `>`(選んでいる札)。
fn is_selected(s: &str, name: &str) -> bool {
    card_at(s, name)
        .is_some_and(|(x, y)| span(s.lines().nth(y).unwrap(), x.saturating_sub(2), x).contains('>'))
}

/// 2行には入る長い名前(幅 40)。端末 80 で3つのレーンの名前の欄(幅 20 ほど)には1行で入らない。
const LONG: &str = "長い問いの名前が二行目まで続いている札";

#[test]
fn test_card_values_and_wrapped_name_e2e() {
    // [stest -> req~kanban.card-columns~1, req~kanban.card-name-wrap~1] card = [author, rating] の表で B → alpha の札は alpha・yamada・4 の
    // 3行(値は2字下げ)。幅に入らない長い名前の札は2行に折り返し、つなぐと名前の全部。
    let notes = vec![
        note(
            "alpha.md",
            "---\nstatus: todo\nauthor: yamada\nrating: 4\n---\n",
        ),
        note(
            &format!("{LONG}.md"),
            "---\nstatus: doing\nauthor: sato\n---\n",
        ),
    ];
    let home = root(
        "kcards-e2e",
        "kanban = { column = \"status\", card = [\"author\", \"rating\"] }",
        &notes,
    );
    let mut tui = Tui::spawn(&["t", "--no-color"], home.path(), &[]);
    tui.wait_until("表が出る", |s| {
        s.contains("alpha") && s.contains("▦ 表 ▾")
    });
    tui.send("B");
    tui.wait_until("カンバンの札に値", |s| {
        is_kanban(s) && rows_from(s, "alpha", 3) == ["alpha", "  yamada", "  4"]
    });
    let head = &LONG[..LONG.char_indices().nth(4).unwrap().0];
    let s = tui.screen();
    let rows = rows_from(&s, head, 3);
    assert_eq!(rows.len(), 3, "{s}");
    assert_eq!(
        format!("{}{}", rows[0], rows[1].trim_start()),
        LONG,
        "名前は2行に折り返し、つなぐと全部:\n{s}"
    );
    assert_eq!(rows[2].trim(), "sato", "名前の2行の下に値:\n{s}");
    assert_gone(tui.stop());
}

#[test]
fn test_tall_cards_scroll_and_click_e2e() {
    // [stest -> req~kanban.card-scroll~1, req~kanban.card-click~1] 3行の札が 12枚のレーンで j を 10回 → 11枚目(n11)の札の全部の行が見え、
    // 選んでいる。見えている n10 の値の行をクリック(SGR 1006 の報告)→ n10 を選ぶ。
    let mut notes = Vec::new();
    for i in 1..=12 {
        notes.push(note(
            &format!("n{i:02}.md"),
            &format!("---\nstatus: todo\na: a{i:02}\nb: b{i:02}\n---\n"),
        ));
    }
    let home = root(
        "kcards-e2e-tall",
        "kanban = { column = \"status\", card = [\"a\", \"b\"] }",
        &notes,
    );
    let mut tui = Tui::spawn(&["t", "--no-color"], home.path(), &[]);
    tui.wait_until("表が出る", |s| {
        s.contains("n01") && s.contains("▦ 表 ▾")
    });
    tui.send("B");
    tui.wait_until("カンバン", |s| is_kanban(s) && is_selected(s, "n01"));
    for _ in 0..10 {
        tui.send("j");
    }
    tui.wait_until("11枚目の札の全部の行", |s| {
        is_selected(s, "n11") && rows_from(s, "n11", 3) == ["n11", "  a11", "  b11"]
    });
    let s = tui.screen();
    let (x, y) = card_at(&s, "n10").unwrap_or_else(|| panic!("n10 が見える:\n{s}"));
    let (cx, cy) = (x + 2, y + 2);
    tui.send(&format!("\x1b[<0;{};{}M", cx + 1, cy + 1));
    tui.send(&format!("\x1b[<0;{};{}m", cx + 1, cy + 1));
    tui.wait_until("n10 の値の行のクリックで n10", |s| {
        is_kanban(s) && is_selected(s, "n10")
    });
    assert_gone(tui.stop());
}
