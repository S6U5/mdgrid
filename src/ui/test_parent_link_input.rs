//! 親を指す列の入れ方(relations.link-input-children-key・-typed-name・-same-shown-name)と、名前を付けた列を
//! 表にある列と数えること(navigation.declared-columns)の試験。スターターから作った台帳と同じ形(新しいノートの
//! 名前の頭に日付を付け、表では隠す)の印で開く。期待値は要求の受け入れの例から取った。今日は 2026-11-26 に固定する。

use super::grid::Slot;
use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{col_named, ctrl, press, text, typing, Tmp};
use super::*;
use mdgrid::source::{NewValue, RowId};
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;

/// 印: タスクの表。親子は parent、名前の頭の日付は隠す。新しいノートの窓は parent を聞く。`extra` は表の印の後ろに足す。
fn marker(children_key: &str, extra: &str) -> String {
    format!(
        r#"[display]
note_name = '^\d{{8}}_'

[rules.status]
values = ["todo", "doing", "done"]

[[table]]
name = "タスク"
children = {{ key = "{children_key}" }}

[table.new_note]
folder = "Tasks"
name = "{{date:YYYYMMDD}}_"
name_head = "hide"
ask = ["parent"]

[table.base]
filters = {{ and = ['file.inFolder("Tasks")'] }}

{extra}
"#
    )
}

/// 名前を付けた parent と、それを並べるビュー(スターターの印と同じ)。
const DECLARED: &str = r#"[table.base.properties]
"note.parent" = { displayName = "親タスク" }

[[table.base.views]]
type = "table"
name = "全部"
order = ["status", "parent"]
"#;

/// 名前を付けず、ビューにも並べない(owner はどこにも無い)。
const UNDECLARED: &str = r#"[[table.base.views]]
type = "table"
name = "全部"
order = ["status"]
"#;

/// どのノートにも parent が無い3つのタスク。
const THREE: &[&str] = &[
    "20261126_見積もりを出す",
    "20261126_見積書を作る",
    "20261126_部屋の片づけ",
];

/// 同じ名前のタスクが別の日にもう1つずつ。
const TWINS: &[&str] = &["20261126_請求書を送る", "20261226_請求書を送る"];

fn boot(tag: &str, names: &[&str], marker: &str) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    std::fs::create_dir_all(tmp.notes().join("Tasks")).unwrap();
    for n in names {
        tmp.write(&format!("Tasks/{n}.md"), "---\nstatus: todo\n---\n");
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
    a.today = mdgrid::types::parse_date("2026-11-26").unwrap();
    (tmp, a)
}

fn screen(a: &App) -> String {
    let (w, h) = a.size;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw(f, a)).unwrap();
    text(term.backend().buffer())
}

/// ファイルの名前(拡張子なし)が `stem` の行。
fn row(a: &App, stem: &str) -> RowId {
    a.src
        .rows()
        .into_iter()
        // パスの区切りは OS で違う(Windows は `\`)ので、ファイルの名前で比べる。
        .find(|r| std::path::Path::new(&r.0).file_name() == Some(format!("{stem}.md").as_ref()))
        .unwrap_or_else(|| panic!("{stem} が無い"))
}

/// 表の画面で、ファイルの名前が `stem` の行を選ぶ。
fn select(a: &mut App, stem: &str) {
    let r = row(a, stem);
    a.row = (0..a.slots.len())
        .find(|&i| matches!(a.slots[i], Slot::Row(k) if a.rows[k] == r))
        .unwrap_or_else(|| panic!("{stem} の行が表に無い"));
}

/// `stem` の行の parent のセルを開く。
fn edit_parent(a: &mut App, stem: &str) {
    select(a, stem);
    col_named(a, "parent");
    press(a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit, "{:?}", a.message);
}

/// 開いている入力の候補の見せる名前。
fn items(a: &App) -> Vec<String> {
    a.input
        .as_ref()
        .and_then(|i| i.list.as_ref())
        .map(|l| l.items.iter().map(|i| i.text.clone()).collect())
        .unwrap_or_default()
}

/// 候補のリストで、見せる名前が `text` の項目を選ぶ(先頭まで ↑ で上がってから ↓ で探す)。
fn choose(a: &mut App, text: &str) {
    let n = items(a).len();
    for _ in 0..n {
        press(a, KeyCode::Up);
    }
    for _ in 0..n {
        let l = a.input.as_ref().and_then(|i| i.list.as_ref()).unwrap();
        if l.items.get(l.sel).is_some_and(|it| it.text == text) {
            return;
        }
        press(a, KeyCode::Down);
    }
    panic!("候補に {text} が無い: {:?}", items(a));
}

fn pending(a: &App, stem: &str) -> Option<NewValue> {
    a.changes.pending(&row(a, stem), "parent").cloned()
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

// ---- relations.link-input-children-key ----

#[test]
fn test_parent_candidates_are_the_table_notes_before_any_parent_is_set() {
    // [utest -> req~relations.link-input-children-key~1] どのノートにも parent が無い表で、見積書を作るの parent のセルの
    // 候補は「見積もりを出す」「部屋の片づけ」と「なし」(自分は出ない)。見積もりを出すを選ぶと、頭の日付ごとの
    // ファイルの名前へのリンク `[[20261126_見積もりを出す]]` を書く。
    let (_t, mut a) = boot("plk_cands", THREE, &marker("parent", DECLARED));
    edit_parent(&mut a, "20261126_見積書を作る");
    assert_eq!(
        items(&a),
        vec![
            "見積もりを出す".to_string(),
            "部屋の片づけ".into(),
            "なし".into()
        ]
    );
    choose(&mut a, "見積もりを出す");
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        pending(&a, "20261126_見積書を作る"),
        Some(NewValue::Str("[[20261126_見積もりを出す]]".into()))
    );
}

#[test]
fn test_new_note_form_parent_field_lists_the_table_notes() {
    // [utest -> req~relations.link-input-children-key~1] 新しいノートの窓で parent の欄を開くと、表の3つのタスクが
    // 頭を外した名前で並ぶ。
    let (_t, mut a) = boot("plk_form", THREE, &marker("parent", DECLARED));
    press(&mut a, KeyCode::Char('a'));
    assert!(a.note.flow.is_some(), "窓が開く:\n{}", screen(&a));
    typing(&mut a, "見積もりの確認");
    press(&mut a, KeyCode::Enter);
    let got = items(&a);
    for n in ["見積もりを出す", "見積書を作る", "部屋の片づけ"] {
        assert!(got.contains(&n.to_string()), "{n}: {got:?}");
    }
}

// ---- relations.link-input-typed-name ----

#[test]
fn test_typed_shown_name_writes_the_link() {
    // [utest -> req~relations.link-input-typed-name~1] parent のセルに候補の見せる名前「見積もりを出す」を打って Enter で、
    // 打った文字ではなく `[[20261126_見積もりを出す]]` を書く。
    let (_t, mut a) = boot("plk_typed", THREE, &marker("parent", DECLARED));
    edit_parent(&mut a, "20261126_見積書を作る");
    typing(&mut a, "見積もりを出す");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table, "{:?}", a.message);
    assert_eq!(
        pending(&a, "20261126_見積書を作る"),
        Some(NewValue::Str("[[20261126_見積もりを出す]]".into()))
    );
}

#[test]
fn test_typed_file_name_writes_the_link() {
    // [utest -> req~relations.link-input-typed-name~1] 頭の日付ごとのファイルの名前「20261126_見積もりを出す」を打っても同じ。
    let (_t, mut a) = boot("plk_typed_stem", THREE, &marker("parent", DECLARED));
    edit_parent(&mut a, "20261126_見積書を作る");
    typing(&mut a, "20261126_見積もりを出す");
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        pending(&a, "20261126_見積書を作る"),
        Some(NewValue::Str("[[20261126_見積もりを出す]]".into()))
    );
}

#[test]
fn test_typed_name_of_two_notes_writes_nothing_and_asks_to_choose() {
    // [utest -> req~relations.link-input-typed-name~1] 「請求書を送る」が 2 つあるとき、その名前を打って Enter では何も
    // 書かず、入力を開いたまま、2 件あるので ↑↓ で選ぶように下の帯に出す。
    let notes: Vec<&str> = THREE.iter().chain(TWINS).copied().collect();
    let (_t, mut a) = boot("plk_twins", &notes, &marker("parent", DECLARED));
    edit_parent(&mut a, "20261126_見積書を作る");
    typing(&mut a, "請求書を送る");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit, "入力は開いたまま");
    assert_eq!(pending(&a, "20261126_見積書を作る"), None);
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains('2') && m.contains("↑↓"), "{m}");
}

#[test]
fn test_typed_name_of_no_note_is_written_as_typed() {
    // [utest -> req~relations.link-input-typed-name~1] どのノートの名前とも合わない「まだ無いタスク」は、打った文字のまま書く。
    let (_t, mut a) = boot("plk_free", THREE, &marker("parent", DECLARED));
    edit_parent(&mut a, "20261126_見積書を作る");
    typing(&mut a, "まだ無いタスク");
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        pending(&a, "20261126_見積書を作る"),
        Some(NewValue::Str("まだ無いタスク".into()))
    );
}

#[test]
fn test_typed_shown_name_in_the_new_note_form_writes_the_link() {
    // [utest -> req~relations.link-input-typed-name~1] 新しいノートの窓の parent の欄でも、「見積もりを出す」と打って決めると、
    // 作ったノートに `parent: "[[20261126_見積もりを出す]]"`。
    let (t, mut a) = boot("plk_form_typed", THREE, &marker("parent", DECLARED));
    press(&mut a, KeyCode::Char('a'));
    typing(&mut a, "見積もりの確認");
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "見積もりを出す");
    press(&mut a, KeyCode::Enter);
    if a.note.flow.is_some() {
        ctrl(&mut a, 's');
    }
    assert_eq!(a.mode, Mode::Table, "{:?}\n{}", a.message, screen(&a));
    let made = std::fs::read_to_string(t.notes().join("Tasks/20261126_見積もりの確認.md")).unwrap();
    assert!(
        made.contains("parent: \"[[20261126_見積もりを出す]]\""),
        "{made}"
    );
}

// ---- relations.link-input-same-shown-name ----

#[test]
fn test_candidates_with_the_same_shown_name_show_the_file_name() {
    // [utest -> req~relations.link-input-same-shown-name~1] 見せる名前が重なる 2 つの「請求書を送る」は、候補に
    // 「20261126_請求書を送る」「20261226_請求書を送る」と並び、ほかは「見積もりを出す」のように頭を外したまま。
    let notes: Vec<&str> = THREE.iter().chain(TWINS).copied().collect();
    let (_t, mut a) = boot("plk_same", &notes, &marker("parent", DECLARED));
    edit_parent(&mut a, "20261126_見積書を作る");
    let got = items(&a);
    for n in [
        "20261126_請求書を送る",
        "20261226_請求書を送る",
        "見積もりを出す",
        "部屋の片づけ",
    ] {
        assert!(got.contains(&n.to_string()), "{n}: {got:?}");
    }
    assert!(!got.contains(&"請求書を送る".to_string()), "{got:?}");
}

// ---- navigation.declared-columns ----

#[test]
fn test_declared_parent_column_does_not_warn_and_nests_after_saving() {
    // [utest -> req~navigation.declared-columns~1] properties に名前を付けた parent は、どのノートにも値が無くても警告を
    // 出さない。見積書を作るの parent に見積もりを出すへのリンクを入れて保存すると、開き直さずに見積もりを出すの下に
    // 字下げした見積書を作るが並ぶ。
    let (_t, mut a) = boot("plk_declared", THREE, &marker("parent", DECLARED));
    let m = a.message.clone().unwrap_or_default();
    assert!(!m.contains("children"), "警告を出さない: {m}");
    edit_parent(&mut a, "20261126_見積書を作る");
    choose(&mut a, "見積もりを出す");
    press(&mut a, KeyCode::Enter);
    ctrl(&mut a, 's');
    if a.mode == Mode::Confirm {
        press(&mut a, KeyCode::Enter);
    }
    assert_eq!(a.changes.count(), 0, "{:?}", a.message);
    let n = names(&a);
    let at = n
        .iter()
        .position(|x| x == "▾ 見積もりを出す (1)")
        .unwrap_or_else(|| panic!("{n:?}\n{}", screen(&a)));
    assert_eq!(
        n.get(at + 1).map(String::as_str),
        Some("    見積書を作る"),
        "{n:?}"
    );
}

#[test]
fn test_undeclared_children_key_still_warns() {
    // [utest -> req~navigation.declared-columns~1] owner がどのノートにも、properties にも、ビューの order にも無ければ、
    // 今までどおり警告する。
    let (_t, a) = boot("plk_undeclared", THREE, &marker("owner", UNDECLARED));
    let m = a.message.clone().unwrap_or_default();
    assert!(
        m.contains("table[1].children.key: ") && m.contains("owner"),
        "{m}"
    );
}
