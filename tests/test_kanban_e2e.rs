#![cfg(unix)]
//! カンバン(KB-3)の、本物の実行ファイルでの確かめ(端末 80×24)。値は作ったもの。
//! 記録: specs/_history/decidespec/_changes/2026-11-02-kanban.md(タスク 6)。実装の前に書いた。
//! 行番号や字の位置は決め打ちせず、題「カンバン」・レーンの値・札の名前・下の帯の位置(「表 5/5行」)で見る。

use crate::pty;

use pty::{assert_gone, TempDir, Tui};
use unicode_width::UnicodeWidthStr;

/// 印の先頭に状態の列の決まり、[[table]] に `line` を書いた根と、その下の表 `t`。
/// 行は名前の順で alpha(todo)・beta(todo)・delta(done)・gamma(doing)・zeta(doing)。
fn root(tag: &str, line: &str) -> TempDir {
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
            "[rules.status]\nvalues = [\"todo\", \"doing\", \"waiting\", \"done\"]\n\n\
             [[table]]\nname = \"T\"\npath = \"t\"\n{line}\n"
        ),
    );
    w("t/alpha.md", "---\nstatus: todo\n---\n");
    w("t/beta.md", "---\nstatus: todo\n---\n");
    w("t/gamma.md", "---\nstatus: doing\n---\n");
    w("t/zeta.md", "---\nstatus: doing\n---\n");
    w("t/delta.md", "---\nstatus: done\n---\n");
    std::fs::create_dir_all(h.join("state")).unwrap();
    home
}

const KANBAN: &str = "kanban = { column = \"status\" }";

/// 1つの行に `words` がこの順で並ぶ。
fn in_order(s: &str, words: &[&str]) -> bool {
    s.lines().any(|l| {
        let mut at = 0;
        words.iter().all(|w| match l[at..].find(w) {
            Some(i) => {
                at += i + w.len();
                true
            }
            None => false,
        })
    })
}

/// カンバンの画面: 題「カンバン」と、レーンの見出し(todo・doing・waiting・done)がこの順で1行に。
fn is_kanban(s: &str) -> bool {
    s.contains("カンバン") && in_order(s, &["todo", "doing", "waiting", "done"])
}

/// 表の画面(ヘッダーの「▦ 表 ▾」があり、カンバンの題が無い)。
fn is_table(s: &str) -> bool {
    s.contains("▦ 表 ▾") && !s.contains("カンバン")
}

/// 札 `name` の (桁, 行)。0 始まり。レーンの見出しより下で最初に名前が出る所。
fn card_pos(s: &str, name: &str) -> Option<(usize, usize)> {
    let lines: Vec<&str> = s.lines().collect();
    let head = lines
        .iter()
        .position(|l| in_order(l, &["todo", "doing", "waiting", "done"]))?;
    lines
        .iter()
        .enumerate()
        .skip(head + 1)
        .find_map(|(y, l)| l.find(name).map(|i| (UnicodeWidthStr::width(&l[..i]), y)))
}

/// 札 `name` の左の、同じレーンの字に `>`(選んでいる札)がある。
fn is_selected(s: &str, name: &str) -> bool {
    let lines: Vec<&str> = s.lines().collect();
    let Some((_, y)) = card_pos(s, name) else {
        return false;
    };
    let l = lines[y];
    let i = l.find(name).unwrap();
    let start = l[..i]
        .char_indices()
        .rfind(|(_, c)| matches!(c, '│' | '┃' | '|'))
        .map(|(j, c)| j + c.len_utf8())
        .unwrap_or(0);
    l[start..i].contains('>')
}

#[test]
fn test_kb_3_b_opens_kanban_and_enter_returns_e2e() {
    // [stest -> req~kanban.keys-enter~1] B → カンバンの題とレーン(todo・doing・waiting・done の順)。l → j → Enter で表に戻り、
    // doing の2件目(zeta。表の5行目)が選ばれる。q で終わる(終了コード 0)。
    let home = root("kanban-e2e", KANBAN);
    let mut tui = Tui::spawn(&["t", "--no-color"], home.path(), &[]);
    tui.wait_until("表が出る", |s| {
        s.contains("alpha") && s.contains("▦ 表 ▾") && s.contains("1/5行")
    });
    tui.send("B");
    tui.wait_until("カンバンの画面", is_kanban);
    let s = tui.screen();
    for name in ["alpha", "beta", "gamma", "zeta", "delta"] {
        assert!(card_pos(&s, name).is_some(), "札 {name}:\n{s}");
    }
    tui.send("l");
    tui.wait_until("doing の1件目を選ぶ", |s| is_selected(s, "gamma"));
    tui.send("j");
    tui.wait_until("doing の2件目を選ぶ", |s| is_selected(s, "zeta"));
    tui.send("\r");
    tui.wait_until("表に戻って zeta(5行目)", |s| {
        is_table(s) && s.contains("5/5行")
    });
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}

#[test]
fn test_kb_3_esc_q_b_return_to_table_e2e() {
    // [stest -> req~kanban.keys-enter~1, req~kanban.keys-back~1] カンバンから Esc・q・B で表に戻る(q で終わらない)。表で q を押すと終わる。
    let home = root("kanban-e2e-back", KANBAN);
    let mut tui = Tui::spawn(&["t"], home.path(), &[]);
    tui.wait_until("表が出る", |s| {
        s.contains("alpha") && s.contains("▦ 表 ▾")
    });
    for back in ["\x1b", "q", "B"] {
        tui.send("B");
        tui.wait_until("カンバンの画面", is_kanban);
        tui.send(back);
        tui.wait_until(&format!("{back:?} で表に戻る"), is_table);
        assert_eq!(tui.try_exit(), None, "{back:?} で終わらない");
    }
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}

#[test]
fn test_kb_3_click_card_twice_opens_row_e2e() {
    // [stest -> req~kanban.keys-click-select~1, req~kanban.keys-click-open~1] 札のクリック(SGR 1006 の報告)でその札を選び、もう一度クリックで表のその行(zeta。5行目)。
    let home = root("kanban-e2e-click", KANBAN);
    let mut tui = Tui::spawn(&["t", "--no-color"], home.path(), &[]);
    tui.wait_until("表が出る", |s| {
        s.contains("alpha") && s.contains("1/5行")
    });
    tui.send("B");
    tui.wait_until("カンバンの画面", is_kanban);
    let s = tui.screen();
    let (x, y) = card_pos(&s, "zeta").unwrap_or_else(|| panic!("札 zeta:\n{s}"));
    let click = |t: &mut Tui| {
        t.send(&format!("\x1b[<0;{};{}M", x + 2, y + 1));
        t.send(&format!("\x1b[<0;{};{}m", x + 2, y + 1));
    };
    click(&mut tui);
    tui.wait_until("クリックで zeta を選ぶ", |s| {
        is_kanban(s) && is_selected(s, "zeta")
    });
    click(&mut tui);
    tui.wait_until("もう一度のクリックで表の zeta", |s| {
        is_table(s) && s.contains("5/5行")
    });
    assert_gone(tui.stop());
}
