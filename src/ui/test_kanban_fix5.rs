//! カンバンのレビュー5回目の指摘 #21 の直しの試験。記録: specs/_history/decidespec/_changes/2026-11-02-kanban.md の「レビューの指摘」と
//! 「実装の気づき」の「レビュー5回目は」。値は作ったもの。錠のある試験のファイルは変えず、補助は test_kanban_fix4.rs
//! から写した。
//!
//! カンバンの上にヘルプ・パレットを開いているあいだに外の変化で設定の列が使えなくなっても、窓は開いたままで、
//! 打ったキーが表のキーとして効かない。窓を閉じてカンバンに戻った最初のキーで、カンバンを閉じて理由を出す。
//! 実装の前の版(4bc740c)に無い名前は使わず、キーを送り、画面の文字と今ある口(`mode`・`quit`・`message`・
//! `changes`・`editor_request`・`poll`)で見る。

use super::app::{App, ColorMode};
use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, edit_outside, press, text, Tmp};
use mdgrid::config::Config;
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
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

/// 外で全部のノートの status を state に変えて poll する(設定の列 status が表から消える)。
fn remove_status_outside(t: &Tmp, a: &mut App) {
    for (n, body) in NOTES {
        edit_outside(t, n, &body.replace("status:", "state:"));
    }
    a.poll();
    assert!(
        !a.src.columns().iter().any(|c| c == "status"),
        "前提: status の列が消えた"
    );
}

/// 窓を閉じてカンバンに戻った最初のキー `L` で、札は動かさず、カンバンを閉じて表に戻り、理由に status が出る。
fn first_key_closes_kanban(a: &mut App) {
    assert!(
        !matches!(a.mode, Mode::Table | Mode::Help | Mode::Palette),
        "前提: 窓を閉じるとカンバンに戻る: {:?} {:?}\n{}",
        a.mode,
        a.message,
        screen(a)
    );
    a.message = None;
    ch(a, 'L');
    assert_eq!(
        a.changes.count(),
        0,
        "そのキーで札を動かさない: {:?}",
        a.message
    );
    assert_table(a);
    let m = reason(a);
    assert!(m.contains("status"), "無い列の名前: {m}");
    assert!(!a.quit);
}

#[test]
fn test_kb_5_help_stays_open_when_setting_column_is_removed_outside() {
    // [utest -> req~kanban.reload~1] #21: カンバンでヘルプ(`?`)を開いたまま、外で status を state に変えて poll → ヘルプは開いたまま。
    // ヘルプを閉じる `q` で終わらず(表の q として効かない)、カンバンに戻った最初のキーで、カンバンを閉じて理由を出す。
    let (t, mut a) = boot("kbfx21help", NOTES, RULES, KANBAN);
    open(&mut a);
    ch(&mut a, '?');
    assert_eq!(a.mode, Mode::Help, "前提: ヘルプが開く: {:?}", a.message);
    remove_status_outside(&t, &mut a);
    assert_eq!(
        a.mode,
        Mode::Help,
        "ヘルプは開いたまま: {:?}\n{}",
        a.message,
        screen(&a)
    );
    ch(&mut a, 'q');
    assert!(!a.quit, "ヘルプを閉じる q で終わらない");
    assert_ne!(a.mode, Mode::Help, "q でヘルプを閉じる");
    first_key_closes_kanban(&mut a);
}

#[test]
fn test_kb_5_palette_stays_open_when_setting_column_is_removed_outside() {
    // [utest -> req~kanban.reload~1] #21: カンバンでパレット(`:`)を開いたまま、外で status を state に変えて poll → パレットは開いたまま。
    // 打った `e` はパレットの文字で、表のキー(エディタで開く)として効かない。Esc でパレットを閉じてカンバンに
    // 戻った最初のキーで、カンバンを閉じて理由を出す。
    let (t, mut a) = boot("kbfx21pal", NOTES, RULES, KANBAN);
    open(&mut a);
    ch(&mut a, ':');
    assert_eq!(
        a.mode,
        Mode::Palette,
        "前提: パレットが開く: {:?}",
        a.message
    );
    remove_status_outside(&t, &mut a);
    assert_eq!(
        a.mode,
        Mode::Palette,
        "パレットは開いたまま: {:?}\n{}",
        a.message,
        screen(&a)
    );
    ch(&mut a, 'e');
    assert_eq!(a.mode, Mode::Palette, "打った字はパレットに入る");
    assert!(
        a.editor_request.is_none(),
        "表の e(エディタで開く)として効かない"
    );
    assert_eq!(a.changes.count(), 0);
    press(&mut a, KeyCode::Esc);
    first_key_closes_kanban(&mut a);
}
