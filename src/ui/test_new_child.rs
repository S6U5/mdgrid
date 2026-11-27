//! 選んだ行の子を作る(cell-edit.new-child-note)の試験。期待値は要求の受け入れの例から取った。
//! 印の children の key が parent のタスクの表を、名前の頭を付けない新しいノートの決まりで開く。

use super::grid::Slot;
use super::keymap::{Action, Mode};
use super::startup::Startup;
use super::test_screen::{ch, ctrl, press, text, typing, Tmp};
use super::*;
use mdgrid::i18n::Msg;
use mdgrid::source::RowId;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;

/// 印: タスクの表。`table_line` は `[[table]]` の中に足す行(children など)、`ask` は新しいノートの窓で聞く列。
fn marker(table_line: &str, ask: &str, views: &str) -> String {
    format!(
        r#"[rules.status]
values = ["todo", "doing", "done"]

[[table]]
name = "タスク"
{table_line}

[table.new_note]
folder = "Tasks"
ask = [{ask}]

[table.new_note.set]
status = "todo"

[table.base]
filters = {{ and = ['file.inFolder("Tasks")'] }}

{views}
"#
    )
}

const CHILDREN: &str = r#"children = { key = "parent" }"#;

/// 状態と親を並べるビュー。
const ALL: &str = r#"[[table.base.views]]
type = "table"
name = "全部"
order = ["status", "parent"]
"#;

/// 状態でまとめるビュー(先頭の行はまとまりの見出し)。
const BY_STATUS: &str = r#"[[table.base.views]]
type = "table"
name = "状態ごと"
order = ["status", "parent"]
groupBy = { property = "status", direction = "ASC" }
"#;

/// `(名前, フロントマターの中)` のノートを Tasks/ に置き、印で開く。
fn boot(tag: &str, notes: &[(&str, &str)], marker: &str) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    std::fs::create_dir_all(tmp.notes().join("Tasks")).unwrap();
    for (n, fm) in notes {
        tmp.write(&format!("Tasks/{n}.md"), &format!("---\n{fm}---\n"));
    }
    let dir = tmp.notes().join(".mdgrid");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("workspace.toml"), marker).unwrap();
    let p = tmp.notes().join("タスク.base");
    let cdir = tmp.0.join("config");
    std::fs::create_dir_all(&cdir).unwrap();
    let t = crate::open_target(std::slice::from_ref(&p), None).unwrap();
    let mut a = App::new(Box::new(t.src), ColorMode::None);
    a.resize(120, 30);
    a.start(Startup {
        config: mdgrid::config::Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(cdir),
        target: p.clone(),
        base: t.base,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    (tmp, a)
}

/// 見積もりを出す・部屋の片づけ(どちらも親なし)。
const TWO: &[(&str, &str)] = &[
    ("見積もりを出す", "status: todo\n"),
    ("部屋の片づけ", "status: todo\n"),
];

fn screen(a: &App) -> String {
    let (w, h) = a.size;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw(f, a)).unwrap();
    text(term.backend().buffer())
}

fn row(a: &App, stem: &str) -> RowId {
    a.src
        .rows()
        .into_iter()
        .find(|r| std::path::Path::new(&r.0).file_name() == Some(format!("{stem}.md").as_ref()))
        .unwrap_or_else(|| panic!("{stem} が無い"))
}

fn select(a: &mut App, stem: &str) {
    let r = row(a, stem);
    a.row = (0..a.slots.len())
        .find(|&i| matches!(a.slots[i], Slot::Row(k) if a.rows[k] == r))
        .unwrap_or_else(|| panic!("{stem} の行が表に無い"));
}

/// 表の行の名前の欄の文字(右の空白を除く)を上から。
fn names(a: &App) -> Vec<String> {
    let lay = super::view::layout(a);
    let (x0, w) = (lay.label_x(), lay.label_w);
    screen(a)
        .lines()
        .filter(|l| l.contains("todo"))
        .map(|l| {
            let mut col = 0;
            let mut out = String::new();
            for c in l.chars() {
                let cw = super::width::width(&c.to_string());
                if col >= x0 && col + cw <= x0 + w {
                    out.push(c);
                }
                col += cw;
            }
            out.trim_end().to_string()
        })
        .collect()
}

fn note(t: &Tmp, stem: &str) -> String {
    std::fs::read_to_string(t.notes().join(format!("Tasks/{stem}.md"))).unwrap()
}

fn cur_stem(a: &App) -> String {
    let r = a.cur_row().expect("行を選んでいない");
    std::path::Path::new(&r.0)
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

#[test]
fn test_new_child_note_links_the_selected_row() {
    // [utest -> req~cell-edit.new-child-note~1] 「見積もりを出す」の行で c → 名前に「見積書を作る」と打って Ctrl+S
    // → `parent: "[[見積もりを出す]]"` で作られ、「見積もりを出す」のすぐ下に字下げして出て、選ばれる。
    let (t, mut a) = boot("nchild_link", TWO, &marker(CHILDREN, "\"status\"", ALL));
    select(&mut a, "見積もりを出す");
    ch(&mut a, 'c');
    assert_eq!(a.mode, Mode::Edit, "{:?}", a.message);
    typing(&mut a, "見積書を作る");
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Table, "{:?}", a.message);
    let body = note(&t, "見積書を作る");
    assert!(
        body.lines().any(|l| l == "parent: \"[[見積もりを出す]]\""),
        "{body}"
    );
    assert_eq!(
        names(&a),
        ["▾ 見積もりを出す (1)", "    見積書を作る", "  部屋の片づけ"]
    );
    assert_eq!(cur_stem(&a), "見積書を作る");
}

#[test]
fn test_new_child_note_parent_field_is_filled_and_can_change() {
    // [utest -> req~cell-edit.new-child-note~1] ask に parent がある表で「見積もりを出す」の行で c → 窓の parent の欄に
    // 「見積もりを出す」。欄を開いて「部屋の片づけ」に選び直して作ると、parent は部屋の片づけ。
    let (t, mut a) = boot("nchild_field", TWO, &marker(CHILDREN, "\"parent\"", ALL));
    select(&mut a, "見積もりを出す");
    ch(&mut a, 'c');
    assert_eq!(a.mode, Mode::Edit, "{:?}", a.message);
    let s = screen(&a);
    let line = s
        .lines()
        .find(|l| l.contains("│") && l.contains("parent"))
        .unwrap_or_else(|| panic!("parent の欄が無い:\n{s}"));
    assert!(line.contains("見積もりを出す"), "{line}");
    typing(&mut a, "見積書を作る");
    press(&mut a, KeyCode::Enter);
    // parent の欄の候補から「部屋の片づけ」を選ぶ。
    let list = |a: &App| {
        a.input.as_ref().and_then(|i| i.list.as_ref()).map(|l| {
            (
                l.sel,
                l.items.iter().map(|i| i.text.clone()).collect::<Vec<_>>(),
            )
        })
    };
    let (sel, items) = list(&a).unwrap_or_else(|| panic!("候補が無い:\n{}", screen(&a)));
    assert_eq!(
        items[sel], "見積もりを出す",
        "今の値を選んで開く: {items:?}"
    );
    let to = items.iter().position(|i| i == "部屋の片づけ").unwrap();
    for _ in sel..to {
        press(&mut a, KeyCode::Down);
    }
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Table, "{:?}", a.message);
    let body = note(&t, "見積書を作る");
    assert!(
        body.lines().any(|l| l == "parent: \"[[部屋の片づけ]]\""),
        "{body}"
    );
}

#[test]
fn test_new_child_note_under_a_child_makes_a_grandchild() {
    // [utest -> req~cell-edit.new-child-note~1] 「見積もりを出す」の子「見積書を作る」の行で c → 「金額を調べる」を作る
    // → 「見積書を作る」のすぐ下に、もう1段字下げして出る。
    let notes = &[
        ("見積もりを出す", "status: todo\n"),
        (
            "見積書を作る",
            "status: todo\nparent: \"[[見積もりを出す]]\"\n",
        ),
        ("部屋の片づけ", "status: todo\n"),
    ];
    let (_t, mut a) = boot("nchild_grand", notes, &marker(CHILDREN, "\"status\"", ALL));
    select(&mut a, "見積書を作る");
    ch(&mut a, 'c');
    typing(&mut a, "金額を調べる");
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Table, "{:?}", a.message);
    assert_eq!(
        names(&a),
        [
            "▾ 見積もりを出す (1)",
            "  ▾ 見積書を作る (1)",
            "      金額を調べる",
            "  部屋の片づけ"
        ]
    );
}

#[test]
fn test_new_child_note_needs_a_tree() {
    // [utest -> req~cell-edit.new-child-note~1] 親子で並べていない表で c → 窓は開かず、親子で並べていないことと
    // ビューの設定を開くキー(既定 o)を下の行に出す。
    let (_t, mut a) = boot("nchild_notree", TWO, &marker("", "\"status\"", ALL));
    select(&mut a, "見積もりを出す");
    ch(&mut a, 'c');
    assert_eq!(a.mode, Mode::Table);
    assert!(a.note.flow.is_none());
    assert_eq!(
        a.message.as_deref(),
        Some(Msg::TreeOff.fill(&[&"o"]).as_str())
    );
}

#[test]
fn test_new_child_note_needs_a_row() {
    // [utest -> req~cell-edit.new-child-note~1] まとまりの見出しの行を選んでいる親子の表で c → 窓は開かず、子を作る
    // 行を選ぶよう下の行に出す。
    let (_t, mut a) = boot(
        "nchild_head",
        TWO,
        &marker(CHILDREN, "\"status\"", BY_STATUS),
    );
    a.row = 0;
    assert!(matches!(a.slots[0], Slot::Head(_)));
    ch(&mut a, 'c');
    assert_eq!(a.mode, Mode::Table);
    assert!(a.note.flow.is_none());
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains("子を作る行"), "{m}");
}

#[test]
fn test_new_child_note_in_row_menu_and_palette() {
    // [utest -> req~cell-edit.new-child-note~1] 行の操作の窓とコマンドのパレットにも「子を作る」がある(親子の表で行を
    // 選んでいるとき)。
    let (_t, mut a) = boot("nchild_menu", TWO, &marker(CHILDREN, "\"status\"", ALL));
    select(&mut a, "見積もりを出す");
    let act = Action::by_name("new_child_note").expect("動作 new_child_note が無い");
    assert!(super::menu::item_actions(&a).contains(&act));
    let found = super::help::candidates(&a, "new_child");
    assert!(
        found
            .iter()
            .any(|c| c.target == super::help::Target::Run(act)),
        "パレットに無い"
    );
}
