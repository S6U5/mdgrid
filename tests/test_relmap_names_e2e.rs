#![cfg(unix)]
//! [base-view.display-name-everywhere] 印に中身を書いた2つの表(人・会議)のワークスペースで、関係マップの箱と詳細と、
//! つながった行の一覧の列の名前が、その列のある表の displayName(出席者)になることを、本物の実行ファイルで確かめる。
//! 期待値は要求の受け入れの例から取った。

use crate::pty;

use pty::{TempDir, Tui};

/// 人と会議の台帳(会議の attendees に displayName「出席者」、日付の頭を隠す)。
const MARKER: &str = r#"name = "人と会議"
open = "人"

[display]
note_path = "name"
note_name = '^\d{8}_'

[types]
attendees = "list"

[[table]]
name = "人"

[table.base]
filters = { and = ['file.inFolder("People")'] }

[table.base.properties]
"note.kana" = { displayName = "読み" }

[[table.base.views]]
type = "table"
name = "全部"
order = ["kana"]

[[table]]
name = "会議"

[table.base]
filters = { and = ['file.inFolder("Meetings")'] }

[table.base.properties]
"note.attendees" = { displayName = "出席者" }

[[table.base.views]]
type = "table"
name = "全部"
order = ["attendees"]
"#;

/// 台帳を置いたフォルダ。
fn ledger(tag: &str) -> TempDir {
    let t = TempDir::new(tag);
    let root = t.path();
    for d in ["People", "Meetings", ".mdgrid", "config/mdgrid", "state"] {
        std::fs::create_dir_all(root.join(d)).unwrap();
    }
    std::fs::write(root.join(".mdgrid/workspace.toml"), MARKER).unwrap();
    std::fs::write(root.join("People/安藤.md"), "---\nkana: あんどう\n---\n").unwrap();
    std::fs::write(root.join("People/井上.md"), "---\nkana: いのうえ\n---\n").unwrap();
    std::fs::write(
        root.join("Meetings/20261102_定例.md"),
        "---\nattendees:\n  - \"[[安藤]]\"\n---\n",
    )
    .unwrap();
    t
}

#[test]
fn test_relmap_and_linked_rows_use_display_name_e2e() {
    // [stest -> req~base-view.display-name-everywhere~3] 人の表で R → 80 桁の端末(1つの段組み)のつながりの行は
    // 「会議.出席者 → 人」で、attendees は出ない。表に戻って「安藤」の行で「つながった行」を開くと
    // 「会議 › 出席者 ← 定例」。
    let t = ledger("relmap-names");
    let mut tui = Tui::spawn(&[], t.path(), &[]);
    tui.wait_until("人の表", |s| {
        s.contains("安藤") && s.contains("いのうえ")
    });
    tui.send("R");
    tui.wait_until("関係マップ", |s| s.contains("会議.出席者 → 人"));
    let s = tui.screen();
    assert!(!s.contains("attendees"), "{s}");
    tui.send("\x1b");
    tui.wait_until("表に戻る", |s| {
        s.contains("いのうえ") && !s.contains("会議.出席者 → 人")
    });
    // 「安藤」の行(ファイルの名前の順で最後)を選んで、パレットから「つながった行」を開く。
    tui.send("G");
    tui.send(":");
    tui.wait_until("パレット", |s| {
        s.contains("\n: ") || s.lines().any(|l| l.starts_with(": "))
    });
    tui.send("linked_rows");
    tui.wait_for("つながった行");
    tui.send("\r");
    tui.wait_until("つながった行の一覧", |s| {
        s.contains("会議 › 出席者 ← 定例")
    });
    let s = tui.screen();
    assert!(!s.contains("attendees") && !s.contains("20261102_"), "{s}");
    drop(tui);
}
