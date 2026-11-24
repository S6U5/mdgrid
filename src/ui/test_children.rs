//! 印の `[[table]]` の `children`: 親子の並び(navigation.parent-child-config)、親を完了にしたときの子
//! (cell-edit.children-done・-complete・-refuse・-keep)、ガントで子が親の期間をはみ出したとき
//! (gantt.children-span-mark・-extend)。期待値は要求の受け入れの例から取った。今日は 2026-11-24 に固定する。

use super::grid::Slot;
use super::keymap::{Action, Mode};
use super::settings::Sec;
use super::startup::Startup;
use super::test_screen::{ch, col_named, press, text, typing, Tmp};
use super::*;
use mdgrid::gantt::Unit;
use mdgrid::source::markdown::Markdown;
use mdgrid::source::{NewValue, RowId};
use mdgrid::types::parse_date;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;
use std::path::{Path, PathBuf};

const TODAY: &str = "2026-11-24";

/// App の大きさで描いた画面の文字。
fn screen(a: &App) -> String {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    text(t.backend().buffer())
}

/// 画面の下の帯(メッセージ行。空でない最後の行)。
fn bottom(a: &App) -> String {
    screen(a)
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .unwrap_or_default()
        .to_string()
}

/// 印(根 = 一時フォルダ)の [[table]] に `lines` を書き、設定 `config`(config.toml の中身)で `target` を開く。
/// `target` が `.base` なら、そのビューの表として開く。
fn open(tmp: &Tmp, target: &Path, config: &str) -> App {
    let (cfg, warnings) = mdgrid::config::parse(config).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let (src, base) = if target.extension().is_some_and(|x| x == "base") {
        let b = mdgrid::base::parse(&std::fs::read_to_string(target).unwrap()).unwrap();
        let src = Markdown::open_vault(target).unwrap();
        (src, Some((b, "tasks.base".to_string(), None)))
    } else {
        (Markdown::open(&[target.to_path_buf()]).unwrap(), None)
    };
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(120, 30);
    a.start(Startup {
        config: cfg,
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: target.to_path_buf(),
        base,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    a.today = parse_date(TODAY).unwrap();
    a
}

/// `notes` を notes/ に置き、印の [[table]](path = "notes")に `lines` を書いて開く。
fn boot(tag: &str, notes: &[(&str, &str)], lines: &str, config: &str) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    for (n, t) in notes {
        tmp.write(n, t);
    }
    std::fs::create_dir_all(tmp.0.join(".mdgrid")).unwrap();
    std::fs::write(
        tmp.0.join(".mdgrid/workspace.toml"),
        format!("[[table]]\nname = \"T\"\npath = \"notes\"\n{lines}\n"),
    )
    .unwrap();
    let a = open(&tmp, &tmp.notes(), config);
    (tmp, a)
}

/// ノートの名前(拡張子なし)。
fn lab(a: &App, r: &RowId) -> String {
    a.src.label(r).trim_end_matches(".md").to_string()
}

fn row(a: &App, name: &str) -> RowId {
    a.src
        .rows()
        .into_iter()
        .find(|r| lab(a, r) == name)
        .unwrap_or_else(|| panic!("{name} が無い"))
}

/// 表の画面で `name` の行を選ぶ。
fn select(a: &mut App, name: &str) {
    a.row = (0..a.slots.len())
        .find(|&i| matches!(a.slots[i], Slot::Row(r) if lab(a, &a.rows[r]) == name))
        .unwrap_or_else(|| panic!("{name} の行が表に無い"));
}

/// `name` の行の status を `text` に直す(表のセルの入力で)。
fn set_status(a: &mut App, name: &str, text: &str) {
    select(a, name);
    col_named(a, "status");
    press(a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit, "{:?}", a.message);
    a.input.as_mut().unwrap().text.clear();
    a.input.as_mut().unwrap().cursor = 0;
    typing(a, text);
    press(a, KeyCode::Enter);
}

/// ためた値の文字(無ければ None)。
fn pending(a: &App, name: &str, col: &str) -> Option<String> {
    match a.changes.pending(&row(a, name), col)? {
        NewValue::Str(s) | NewValue::Date(s) => Some(s.clone()),
        v => Some(format!("{v:?}")),
    }
}

/// 表の行の名前の欄(字下げと印を含む。右の空白は除く)。表の上から順に。
fn names(a: &App) -> Vec<String> {
    let lay = super::view::layout(a);
    let (x0, w) = (lay.label_x(), lay.label_w);
    screen(a)
        .lines()
        .filter(|l| l.contains("todo") || l.contains("done") || l.contains("doing"))
        .filter(|l| !l.contains("status"))
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

// ---- navigation.parent-child-config ----

const DESIGN: &[(&str, &str)] = &[
    ("設計.md", "---\nstatus: doing\n---\n"),
    ("画面.md", "---\nparent: \"[[設計]]\"\nstatus: todo\n---\n"),
];

// [utest -> req~navigation.parent-child-config~1]
#[test]
fn test_children_key_opens_the_table_in_parent_child_order() {
    // 印に `children = { key = "parent" }` を書いた表を開くと、「▾ 設計」の下に2桁字下げした「画面」が並ぶ。
    let (_t, a) = boot("ch-open", DESIGN, "children = { key = \"parent\" }", "");
    let n = names(&a);
    let at = n
        .iter()
        .position(|x| x == "▾ 設計")
        .unwrap_or_else(|| panic!("{n:?}\n{:?}", a.message));
    assert_eq!(n.get(at + 1).map(String::as_str), Some("  画面"), "{n:?}");
}

/// ビューが2つ(全部・完了)の `.base`。名前はノートの欄に出す(`file.name` の列を並べない)。
const TWO_VIEWS: &str = "views:\n  - type: table\n    name: 全部\n    order: [status]\n  - type: table\n    name: 完了\n    order: [status]\n";

/// 今のビューの設定で、親子のオン・オフを切り替えて反映する(navigation.parent-child-apply)。
fn toggle_tree_in_settings(a: &mut App) {
    ch(a, 'o');
    assert_eq!(a.mode, Mode::Settings);
    for _ in 0..12 {
        if a.draft.as_ref().unwrap().sec == Sec::Tree {
            break;
        }
        press(a, KeyCode::Tab);
    }
    a.draft.as_mut().unwrap().sel[Sec::Tree as usize] = 0;
    press(a, KeyCode::Enter);
    let d = a.draft.as_mut().unwrap();
    d.sec = Sec::Buttons;
    d.sel[Sec::Buttons as usize] = 0;
    press(a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
}

fn indented(a: &App) -> bool {
    names(a).iter().any(|n| n == "  画面")
}

// [utest -> req~navigation.parent-child-config~1]
#[test]
fn test_view_turned_off_stays_off_after_reopening_and_others_stay_on() {
    // ビュー「完了」で親子をオフにして反映し、終了して開き直すと、「完了」は字下げなし、「全部」は親子のまま。
    let tmp = Tmp::new("ch-save");
    for (n, t) in DESIGN {
        tmp.write(n, t);
    }
    let base: PathBuf = tmp.notes().join("tasks.base");
    std::fs::write(&base, TWO_VIEWS).unwrap();
    std::fs::create_dir_all(tmp.0.join(".mdgrid")).unwrap();
    std::fs::write(
        tmp.0.join(".mdgrid/workspace.toml"),
        "[[table]]\nname = \"tasks\"\npath = \"notes/tasks.base\"\nchildren = { key = \"parent\" }\n",
    )
    .unwrap();
    let mut a = open(&tmp, &base, "");
    assert!(indented(&a), "全部: {:?} {:?}", names(&a), a.message);
    a.select_view(1);
    assert!(indented(&a), "完了: {:?}", names(&a));
    toggle_tree_in_settings(&mut a);
    assert!(!indented(&a), "オフにした完了: {:?}", names(&a));
    a.persist_state();
    drop(a);

    let mut b = open(&tmp, &base, "");
    assert!(indented(&b), "開き直した全部: {:?}", names(&b));
    b.select_view(1);
    assert!(!indented(&b), "開き直した完了: {:?}", names(&b));
    assert!(
        names(&b).iter().all(|n| !n.contains('▾')),
        "{:?}",
        names(&b)
    );
    b.select_view(0);
    assert!(indented(&b), "戻った全部: {:?}", names(&b));
}

// [utest -> req~navigation.parent-child-config~1]
#[test]
fn test_children_key_not_in_the_table_warns_and_stays_flat() {
    // `children = { key = "owner" }` で owner の列が表に無いと、場所を言う警告を出し、親子にしない。
    let (_t, a) = boot("ch-nokey", DESIGN, "children = { key = \"owner\" }", "");
    let m = a.message.clone().unwrap_or_default();
    assert!(
        m.contains(".mdgrid/workspace.toml: table[1].children.key: ") && m.contains("owner"),
        "{m}"
    );
    let n = names(&a);
    assert_eq!(n.len(), 2, "{n:?}");
    assert!(
        n.iter().all(|x| !x.starts_with(' ') && !x.contains('▾')),
        "{n:?}"
    );
}

// ---- cell-edit.children-done ----

/// doing の「設計」の子に todo の「画面」と done の「API」、「画面」の子に todo の「部品」。子の無い todo の「メモ」。
const TREE: &[(&str, &str)] = &[
    ("設計.md", "---\nstatus: doing\ndone_at:\n---\n"),
    (
        "画面.md",
        "---\nparent: \"[[設計]]\"\nstatus: todo\ndone_at:\n---\n",
    ),
    (
        "API.md",
        "---\nparent: \"[[設計]]\"\nstatus: done\ndone_at: 2026-11-01\n---\n",
    ),
    (
        "部品.md",
        "---\nparent: \"[[画面]]\"\nstatus: todo\ndone_at:\n---\n",
    ),
    ("メモ.md", "---\nstatus: todo\ndone_at:\n---\n"),
];

const THEN: &str = "[rules.status.then]\ndone = { done_at = \"{date}\" }\n";

// [utest -> req~cell-edit.children-done~1]
#[test]
fn test_unknown_on_done_warns_once_and_completes_the_children() {
    // `on_done = "nope"` は on_done を言う警告1行で、"complete" として扱う(設計を done にすると子孫も done)。
    let (_t, mut a) = boot(
        "ch-nope",
        TREE,
        "children = { key = \"parent\", on_done = \"nope\" }",
        "",
    );
    let m = a.message.clone().unwrap_or_default();
    assert_eq!(
        m.matches(".mdgrid/workspace.toml: table[1].children.on_done: ")
            .count(),
        1,
        "{m}"
    );
    assert_eq!(m.matches("children").count(), 1, "{m}");
    a.message = None;
    set_status(&mut a, "設計", "done");
    assert_eq!(pending(&a, "画面", "status").as_deref(), Some("done"));
    assert_eq!(pending(&a, "部品", "status").as_deref(), Some("done"));
}

// [utest -> req~cell-edit.children-done~1]
#[test]
fn test_closed_values_count_as_finished() {
    // `closed = ["done", "dropped"]` で、設計の子が dropped の「旧案」だけなら、設計を done にしても旧案は dropped のまま。
    let (_t, mut a) = boot(
        "ch-closed",
        &[
            ("設計.md", "---\nstatus: doing\n---\n"),
            (
                "旧案.md",
                "---\nparent: \"[[設計]]\"\nstatus: dropped\n---\n",
            ),
        ],
        "children = { key = \"parent\", closed = [\"done\", \"dropped\"] }",
        "",
    );
    assert!(a.message.is_none(), "{:?}", a.message);
    set_status(&mut a, "設計", "done");
    assert_eq!(pending(&a, "設計", "status").as_deref(), Some("done"));
    assert_eq!(pending(&a, "旧案", "status"), None);
}

// [utest -> req~cell-edit.children-done~1]
#[test]
fn test_parent_child_loop_completes_each_row_once() {
    // 親子が輪になった行(A の親が B・B の親が A)でも、A を done にすると B が done になって止まる。
    let (_t, mut a) = boot(
        "ch-loop",
        &[
            ("A.md", "---\nparent: \"[[B]]\"\nstatus: todo\n---\n"),
            ("B.md", "---\nparent: \"[[A]]\"\nstatus: todo\n---\n"),
        ],
        "children = { key = \"parent\" }",
        "",
    );
    set_status(&mut a, "A", "done");
    assert_eq!(pending(&a, "A", "status").as_deref(), Some("done"));
    assert_eq!(pending(&a, "B", "status").as_deref(), Some("done"));
}

// [utest -> req~cell-edit.children-done-complete~1]
#[test]
fn test_complete_stages_open_descendants_with_their_then_values() {
    // 設計を done に直すと、設計・画面・部品の status が done、done_at が今日のためる変更になり、API は変わらない。
    let (_t, mut a) = boot("ch-complete", TREE, "children = { key = \"parent\" }", THEN);
    set_status(&mut a, "設計", "done");
    for n in ["設計", "画面", "部品"] {
        assert_eq!(pending(&a, n, "status").as_deref(), Some("done"), "{n}");
        assert_eq!(pending(&a, n, "done_at").as_deref(), Some(TODAY), "{n}");
    }
    assert_eq!(pending(&a, "API", "status"), None);
    assert_eq!(pending(&a, "API", "done_at"), None);
    assert_eq!(pending(&a, "メモ", "status"), None);
}

// [utest -> req~cell-edit.children-done-complete~1]
#[test]
fn test_complete_is_undone_in_one_step() {
    // その直しのあと取り消しを1回で、4つの行とも直す前の値に戻る。
    let (_t, mut a) = boot("ch-undo", TREE, "children = { key = \"parent\" }", THEN);
    set_status(&mut a, "設計", "done");
    assert_eq!(pending(&a, "部品", "status").as_deref(), Some("done"));
    a.apply(Action::Undo);
    for n in ["設計", "画面", "API", "部品"] {
        assert_eq!(pending(&a, n, "status"), None, "{n}");
        assert_eq!(pending(&a, n, "done_at"), None, "{n}");
    }
    assert_eq!(a.changes.count(), 0);
}

// [utest -> req~cell-edit.children-done-complete~1]
#[test]
fn test_moving_the_parent_card_to_done_completes_the_children() {
    // 同じ表をカンバンで開き、設計の札を done のレーンへ動かすと、画面と部品も done のためる変更になる。
    let (_t, mut a) = boot(
        "ch-kanban",
        TREE,
        "children = { key = \"parent\" }\nkanban = { column = \"status\" }",
        "[rules.status]\nvalues = [\"todo\", \"doing\", \"done\"]\n",
    );
    a.open_kanban();
    assert_eq!(a.mode, Mode::Kanban, "{:?}", a.message);
    let design = row(&a, "設計");
    {
        let k = a.kanban.as_mut().unwrap();
        let lane = k
            .lanes
            .iter()
            .position(|l| l.rows.contains(&design))
            .unwrap();
        k.lane = lane;
        k.lanes[lane].sel = k.lanes[lane]
            .rows
            .iter()
            .position(|r| *r == design)
            .unwrap();
    }
    ch(&mut a, 'L');
    assert_eq!(
        pending(&a, "設計", "status").as_deref(),
        Some("done"),
        "{:?}",
        a.message
    );
    assert_eq!(pending(&a, "画面", "status").as_deref(), Some("done"));
    assert_eq!(pending(&a, "部品", "status").as_deref(), Some("done"));
    assert_eq!(pending(&a, "API", "status"), None);
}

// ---- cell-edit.children-done-refuse ----

/// todo の子が2つ(画面・API)ある「設計」と、子の無い「メモ」。
const TWO_KIDS: &[(&str, &str)] = &[
    ("設計.md", "---\nstatus: doing\n---\n"),
    ("画面.md", "---\nparent: \"[[設計]]\"\nstatus: todo\n---\n"),
    ("API.md", "---\nparent: \"[[設計]]\"\nstatus: todo\n---\n"),
    ("メモ.md", "---\nstatus: todo\n---\n"),
];

// [utest -> req~cell-edit.children-done-refuse~1]
#[test]
fn test_refuse_stages_nothing_and_says_how_many_children_are_left() {
    // on_done = "refuse" で todo の子が2つある設計を done に直すと、何もためず、終わっていない子が 2 件ある旨が出る。
    let (_t, mut a) = boot(
        "ch-refuse",
        TWO_KIDS,
        "children = { key = \"parent\", on_done = \"refuse\" }",
        "",
    );
    set_status(&mut a, "設計", "done");
    assert_eq!(a.changes.count(), 0);
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains("終わっていない子が 2 件"), "{m}");
}

// [utest -> req~cell-edit.children-done-refuse~1]
#[test]
fn test_refuse_lets_the_parent_finish_once_the_children_are_done() {
    // 子の2つを done にしたあとなら、設計を done に直すと done のためる変更になる。
    let (_t, mut a) = boot(
        "ch-refuse-ok",
        TWO_KIDS,
        "children = { key = \"parent\", on_done = \"refuse\" }",
        "",
    );
    set_status(&mut a, "画面", "done");
    set_status(&mut a, "API", "done");
    set_status(&mut a, "設計", "done");
    assert_eq!(
        pending(&a, "設計", "status").as_deref(),
        Some("done"),
        "{:?}",
        a.message
    );
}

// [utest -> req~cell-edit.children-done-refuse~1]
#[test]
fn test_refuse_in_a_bulk_edit_sets_only_the_rows_without_open_children() {
    // 設計と子の無いメモを選んで一括で done にすると、メモだけが done になり、直さなかった行の数(1)が理由と一緒に出る。
    let (_t, mut a) = boot(
        "ch-refuse-bulk",
        TWO_KIDS,
        "children = { key = \"parent\", on_done = \"refuse\" }",
        "",
    );
    a.marked.insert(row(&a, "設計"));
    a.marked.insert(row(&a, "メモ"));
    set_status(&mut a, "設計", "done");
    assert_eq!(
        pending(&a, "メモ", "status").as_deref(),
        Some("done"),
        "{:?}",
        a.message
    );
    assert_eq!(pending(&a, "設計", "status"), None);
    assert_eq!(a.changes.count(), 1);
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains("1行を飛ばした"), "{m}");
    assert!(m.contains("終わっていない子が 2 件"), "{m}");
}

// ---- cell-edit.children-done-keep ----

// [utest -> req~cell-edit.children-done-keep~1]
#[test]
fn test_keep_sets_only_the_parent_and_tells_the_open_children() {
    // on_done = "keep" で todo の子が2つある設計を done に直すと、設計だけが done になり、下の帯に終わっていない子が 2 件ある旨の1行。
    let (_t, mut a) = boot(
        "ch-keep",
        TWO_KIDS,
        "children = { key = \"parent\", on_done = \"keep\" }",
        "",
    );
    set_status(&mut a, "設計", "done");
    assert_eq!(pending(&a, "設計", "status").as_deref(), Some("done"));
    assert_eq!(a.changes.count(), 1);
    let last = bottom(&a);
    assert!(last.contains("終わっていない子が 2 件"), "{last}");
}

// ---- gantt.children-span-mark・gantt.children-span-extend ----

const GANTT: &str = "gantt = { start = \"start\", end = \"due\" }";

/// 親(11-01〜11-10)と、子(11-05〜`child_end`)。
fn span_notes(child_end: &str) -> Vec<(&'static str, String)> {
    vec![
        (
            "設計.md",
            "---\nstart: 2026-11-01\ndue: 2026-11-10\nstatus: doing\n---\n".to_string(),
        ),
        (
            "画面.md",
            format!("---\nparent: \"[[設計]]\"\nstart: 2026-11-05\ndue: {child_end}\nstatus: todo\n---\n"),
        ),
    ]
}

fn boot_gantt(tag: &str, notes: &[(&str, String)], children: &str) -> (Tmp, App) {
    let notes: Vec<(&str, &str)> = notes.iter().map(|(n, t)| (*n, t.as_str())).collect();
    let (t, mut a) = boot(tag, &notes, &format!("{GANTT}\n{children}"), "");
    a.open_gantt();
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    a.gantt.as_mut().unwrap().unit = Unit::Day;
    a.message = None;
    (t, a)
}

/// ガントで `name` の行を選ぶ。
fn gantt_select(a: &mut App, name: &str) {
    let r = row(a, name);
    let g = a.gantt.as_mut().unwrap();
    g.sel = g.rows.iter().position(|x| *x == r).unwrap();
}

/// ガントの画面の、ノート `name` の行。
fn gantt_line(a: &App, name: &str) -> String {
    screen(a)
        .lines()
        .find(|l| l.contains(name))
        .unwrap_or_else(|| panic!("{name} の行が無い:\n{}", screen(a)))
        .to_string()
}

/// 色に頼らない、はみ出しの印。
const MARK: char = '!';

// [utest -> req~gantt.children-span-mark~1]
#[test]
fn test_child_over_the_parent_period_marks_both_rows_and_tells_the_days() {
    // 親が 11-01〜11-10、子が 11-05〜11-12 なら、子と親の行に印が付き、子を選ぶと下の帯に「親の期間から 2 日はみ出す」旨。
    let (_t, mut a) = boot_gantt(
        "ch-span",
        &span_notes("2026-11-12"),
        "children = { key = \"parent\" }",
    );
    assert!(gantt_line(&a, "設計").contains(MARK), "{}", screen(&a));
    assert!(gantt_line(&a, "画面").contains(MARK), "{}", screen(&a));
    gantt_select(&mut a, "画面");
    let last = bottom(&a);
    assert!(last.contains("親の期間から 2 日はみ出す"), "{last}");
}

// [utest -> req~gantt.children-span-mark~1]
#[test]
fn test_child_inside_the_parent_period_has_no_mark() {
    // 子が 11-05〜11-10(親の期間の中)なら、どの行にも印が無い。
    let (_t, mut a) = boot_gantt(
        "ch-span-in",
        &span_notes("2026-11-10"),
        "children = { key = \"parent\" }",
    );
    assert!(!gantt_line(&a, "設計").contains(MARK), "{}", screen(&a));
    assert!(!gantt_line(&a, "画面").contains(MARK), "{}", screen(&a));
    gantt_select(&mut a, "画面");
    let last = bottom(&a);
    assert!(!last.contains("はみ出す"), "{last}");
}

// [utest -> req~gantt.children-span-mark~1]
#[test]
fn test_gantt_none_does_not_mark() {
    // `gantt = "none"` なら、子が 11-05〜11-12 でも、どの行にも印が無い。
    let (_t, mut a) = boot_gantt(
        "ch-span-none",
        &span_notes("2026-11-12"),
        "children = { key = \"parent\", gantt = \"none\" }",
    );
    assert!(a.message.is_none(), "{:?}", a.message);
    assert!(!gantt_line(&a, "設計").contains(MARK), "{}", screen(&a));
    assert!(!gantt_line(&a, "画面").contains(MARK), "{}", screen(&a));
    gantt_select(&mut a, "画面");
    let last = bottom(&a);
    assert!(!last.contains("はみ出す"), "{last}");
}

// [utest -> req~gantt.children-span-extend~1]
#[test]
fn test_extend_widens_the_parent_with_the_child_in_one_step() {
    // `gantt = "extend"`、親 11-01〜11-10、子 11-05〜11-10、日の単位で、子を選んで `>` を2回 → 子と親の終わりが 11-12。
    let (_t, mut a) = boot_gantt(
        "ch-extend",
        &span_notes("2026-11-10"),
        "children = { key = \"parent\", gantt = \"extend\" }",
    );
    gantt_select(&mut a, "画面");
    ch(&mut a, '>');
    ch(&mut a, '>');
    assert_eq!(
        pending(&a, "画面", "due").as_deref(),
        Some("2026-11-12"),
        "{:?}",
        a.message
    );
    assert_eq!(pending(&a, "設計", "due").as_deref(), Some("2026-11-12"));
    assert_eq!(pending(&a, "設計", "start"), None);
}

// [utest -> req~gantt.children-span-extend~1]
#[test]
fn test_extend_is_undone_with_the_child() {
    // その変更のあと `u` を2回で、子と親の終わりが 11-10 に戻る。
    let (_t, mut a) = boot_gantt(
        "ch-extend-undo",
        &span_notes("2026-11-10"),
        "children = { key = \"parent\", gantt = \"extend\" }",
    );
    gantt_select(&mut a, "画面");
    ch(&mut a, '>');
    ch(&mut a, '>');
    assert_eq!(pending(&a, "設計", "due").as_deref(), Some("2026-11-12"));
    ch(&mut a, 'u');
    ch(&mut a, 'u');
    assert_eq!(pending(&a, "画面", "due"), None);
    assert_eq!(pending(&a, "設計", "due"), None);
    assert_eq!(a.changes.count(), 0);
}

// [utest -> req~gantt.children-span-extend~1]
#[test]
fn test_extend_leaves_a_parent_without_own_dates() {
    // 自分の日付の無い親(棒は子孫の範囲)では、子を `>` で伸ばしても親には何もためない。
    let (_t, mut a) = boot_gantt(
        "ch-extend-nodate",
        &[
            ("設計.md", "---\nstatus: doing\n---\n".to_string()),
            (
                "画面.md",
                "---\nparent: \"[[設計]]\"\nstart: 2026-11-05\ndue: 2026-11-10\nstatus: todo\n---\n"
                    .to_string(),
            ),
        ],
        "children = { key = \"parent\", gantt = \"extend\" }",
    );
    gantt_select(&mut a, "画面");
    ch(&mut a, '>');
    assert_eq!(
        pending(&a, "画面", "due").as_deref(),
        Some("2026-11-11"),
        "{:?}",
        a.message
    );
    assert_eq!(pending(&a, "設計", "due"), None);
    assert_eq!(pending(&a, "設計", "start"), None);
    assert_eq!(a.changes.count(), 1);
}
