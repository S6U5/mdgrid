//! カンバンのレビュー4回目の指摘 #19(カンバンの分)の直しの試験。記録: specs/_history/decidespec/_changes/2026-11-02-kanban.md の
//! 「レビューの指摘」と「実装の気づき」の「レビュー4回目は」。値は作ったもの。錠のある試験のファイルは変えず、
//! 補助は test_kanban_fix3.rs から写した。
//!
//! 直す前の版で落ち、直した後の版で通る期待にする。実装の前の版(4bc740c)に無い名前は使わず、キーを送り、画面の
//! 文字と今ある口(`mode`・`message`・`changes`・`poll`・`col_type`)で見る。

use super::app::{App, ColorMode};
use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, edit_outside, text, Tmp};
use mdgrid::config::Config;
use mdgrid::source::markdown::Markdown;
use mdgrid::types::Kind;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::path::PathBuf;

/// 状態の列の決まり(CE-36)。レーンの順はこの順。
const RULES: &str = "[rules.status]\nvalues = [\"todo\", \"doing\", \"waiting\", \"done\"]\n";

/// 印の [[table]] のカンバンの設定。
const KANBAN: &str = "kanban = { column = \"status\" }";

/// alpha(todo)・beta(doing)・gamma(done)。
const NOTES: &[(&str, &str)] = &[
    ("alpha.md", "---\nstatus: todo\n---\n"),
    ("beta.md", "---\nstatus: doing\n---\n"),
    ("gamma.md", "---\nstatus: done\n---\n"),
];

/// 縦の線(レーンの間と枠の縦の辺)。
const SEPS: [char; 3] = ['│', '┃', '|'];

/// レーンの見出しの (区画の番号, 値, 数)。
type HeadCell = (usize, String, usize);

/// 印(根 = 一時フォルダ)の先頭に `top`、[[table]] に `line` を書いて `notes` を表として開き、読み終える。
fn boot(tag: &str, notes: &[(&str, &str)], top: &str, line: &str) -> (Tmp, App) {
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

/// カンバンの枠の題の行か(「カンバン」のすぐあとに `:` か `:`)。
fn is_kanban_title(l: &str) -> bool {
    l.match_indices("カンバン")
        .any(|(i, w)| l[i + w.len()..].trim_start().starts_with([':', ':']))
}

/// `B` でカンバンを開く(開かなければ落ちる)。
fn open(a: &mut App) {
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
        body.lines().any(is_kanban_title) && heading_row(&body).is_some(),
        "カンバンの題とレーンの見出し:\n{s}"
    );
}

/// 表に戻ったか(メッセージ行を除いた所で見る: ヘッダーの画面の型は表で、カンバンの枠の題とレーンの見出しが無い)。
fn assert_table(a: &App) {
    let s = screen(a);
    assert_eq!(
        a.mode,
        Mode::Table,
        "カンバンを閉じる: {:?}\n{s}",
        a.message
    );
    let body = above_message(&s);
    assert!(
        body.contains("表 ▾") && !body.contains("カンバン ▾"),
        "ヘッダーの画面の型は表:\n{s}"
    );
    assert!(!body.lines().any(is_kanban_title), "{s}");
    assert!(heading_row(&body).is_none(), "{s}");
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

/// 閉じたあとに、札を動かすキー(H・L)と B を押しても、ためる変更ができず、カンバンも開かない。
fn no_card_moves(a: &mut App) {
    for key in ['H', 'L', 'H'] {
        ch(a, key);
        assert_eq!(
            a.changes.count(),
            0,
            "{key} で値をためない: {:?}",
            a.message
        );
    }
    a.message = None;
    ch(a, 'B');
    assert_eq!(a.mode, Mode::Table, "B でも開かない: {:?}", a.message);
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_kb_5_setting_column_removed_outside_closes_kanban() {
    // [utest -> req~kanban.reload~1, req~kanban.config-empty-table~1] #19: 読み終えてからカンバンを開き、外で全部のノートの status を state に変えて poll → 古い設定の
    // まま組み直さず、カンバンを閉じて表に戻り、理由1行に無い列の名前 status が出る。そのあと札を動かすキーを
    // 押しても、表に無い列に値をためない。
    // 設定に status を書いていない台帳で確かめる(`[rules.status]` に書いた列は、ノートに無くても表にあると数える。kanban.config-declared-column)。
    let (t, mut a) = boot("kbfx19gone", NOTES, "", KANBAN);
    open(&mut a);
    for (n, body) in NOTES {
        edit_outside(&t, n, &body.replace("status:", "state:"));
    }
    a.poll();
    assert!(
        !a.src.columns().iter().any(|c| c == "status"),
        "前提: status の列が消えた"
    );
    assert_table(&a);
    let m = reason(&a);
    assert!(m.contains("status"), "無い列の名前: {m}");
    no_card_moves(&mut a);
}

#[test]
fn test_kb_5_setting_column_turned_list_outside_closes_kanban() {
    // [utest -> req~kanban.reload~1, req~kanban.config-guess~1, req~kanban.config-empty-table~1] #19: 読み終えてからカンバンを開き、外で全部のノートの status をリストの値にして poll → 列の型が
    // リストになるので、古い設定のまま組み直さず、カンバンを閉じて表に戻り、理由1行にリストのことと列の名前が出る。
    // そのあと札を動かすキーを押しても、リストの値を文字の値で上書きしてためない。
    let (t, mut a) = boot("kbfx19list", NOTES, RULES, KANBAN);
    open(&mut a);
    edit_outside(&t, "alpha.md", "---\nstatus: [todo, x]\n---\n");
    edit_outside(&t, "beta.md", "---\nstatus: [doing]\n---\n");
    edit_outside(&t, "gamma.md", "---\nstatus: [done, y]\n---\n");
    a.poll();
    assert_eq!(
        a.col_type("status"),
        Kind::List,
        "前提: status の列がリストの型になった"
    );
    assert_table(&a);
    let m = reason(&a);
    assert!(
        m.contains("リスト") && m.contains("status"),
        "リストの列なので出さない理由: {m}"
    );
    no_card_moves(&mut a);
}
