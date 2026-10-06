//! リストの選択の試験(list-select のタスク 2。CE-16・CE-17・CE-19)。材料は test_screen の Tmp を使う。

use super::super::keymap::{self, Action, Mode, BINDINGS};
use super::super::test_screen::{
    app_of, assert_fits, col_named, ctrl, golden, make, press, read, screen, typing, Tmp,
};
use super::super::{view, width, App, ColorMode};
use super::{Mark, PickRow};
use mdgrid::source::{NewValue, RowId};
use ratatui::crossterm::event::KeyCode;

/// 会議 5件・本 2件・会計 1件・旅行 1件。n6 は tags が無い。
const TAGS: &[(&str, &str)] = &[
    ("n1.md", "---\ntitle: 一\ntags:\n  - 会議\n  - 本\n---\n"),
    ("n2.md", "---\ntitle: 二\ntags: [会議, 本]\n---\n"),
    ("n3.md", "---\ntitle: 三\ntags: [会議]\n---\n"),
    ("n4.md", "---\ntitle: 四\ntags: [会議, 会計]\n---\n"),
    ("n5.md", "---\ntitle: 五\ntags: [会議, 旅行]\n---\n"),
    ("n6.md", "---\ntitle: 六\n---\n"),
];

fn pending(a: &App, row: usize, col: &str) -> Option<NewValue> {
    let r: &RowId = &a.rows[row];
    a.changes.pending(r, col).cloned()
}

fn list(items: &[&str]) -> Option<NewValue> {
    Some(NewValue::List(
        items.iter().map(|s| s.to_string()).collect(),
    ))
}

/// 行 `row` の tags のセルでリストの選択を開く。
fn open_tags(a: &mut App, row: usize) {
    a.row = row;
    col_named(a, "tags");
    press(a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::ListPick, "{:?}", a.message);
}

/// 見せている行(新規は `+名前`、候補は `印 名前 件数`)。
fn shown(a: &App) -> Vec<String> {
    let p = a.pick.as_ref().unwrap();
    p.rows()
        .iter()
        .map(|r| match r {
            PickRow::New(t) => format!("+{t}"),
            PickRow::Cand(i) => {
                let c = &p.cands[*i];
                format!("{} {} {}", c.mark.text(), c.name, c.count)
            }
        })
        .collect()
}

/// 候補を名前で選ぶ。
fn select(a: &mut App, name: &str) {
    let p = a.pick.as_mut().unwrap();
    let rows = p.rows();
    p.sel = rows
        .iter()
        .position(|r| matches!(r, PickRow::Cand(i) if p.cands[*i].name == name))
        .unwrap_or_else(|| panic!("候補に無い: {name}"));
}

fn space(a: &mut App) {
    press(a, KeyCode::Char(' '));
}

#[test]
fn test_ce_16_open_filter_and_add() {
    // [CE-16] tags のセルで Enter → 保管庫の中のタグの一覧。[CE-19] 件数の多い順、同じ件数は文字の順、今の値に [x]。
    let (_t, mut a) = make("ce16", TAGS);
    open_tags(&mut a, 2);
    assert_eq!(
        shown(&a),
        vec!["[x] 会議 5", "[ ] 本 2", "[ ] 会計 1", "[ ] 旅行 1"]
    );
    let s = screen(&a);
    assert!(s.contains("[x] 会議  5件"), "{s}");
    golden("ce_16", &s);
    // 「会」と打つ → 会議・会計 だけに絞られる(打った「会」は候補に無いので先頭に新規)。
    typing(&mut a, "会");
    assert_eq!(shown(&a), vec!["+会", "[x] 会議 5", "[ ] 会計 1"]);
    let s = screen(&a);
    assert!(s.contains("+ 新規: 会") && !s.contains("旅行"), "{s}");
    golden("ce_16_filter", &s);
    // [SR-17] 端末のカーソルは検索の入力の位置。
    let (x, y) = view::cursor(&a).unwrap();
    let line = s.lines().nth(y as usize).unwrap();
    assert!(line.contains("検索: 会"), "{line}");
    let at = line.find("検索: 会").unwrap();
    assert_eq!(
        x as usize,
        width::width(&line[..at]) + width::width("検索: 会")
    );
    assert_fits(&mut a);
    // 検索中の Enter は、選んだ 会計 を付けて検索を空に戻す(窓は開いたまま)。Tab で確定
    // → tags に 会計 が足されたためる変更(元の並びは保ち、末尾に足す)。
    select(&mut a, "会計");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::ListPick);
    assert_eq!(a.pick.as_ref().unwrap().query, "");
    assert_eq!(shown(&a)[2], "[x] 会計 1");
    press(&mut a, KeyCode::Tab);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(pending(&a, 2, "tags"), list(&["会議", "会計"]));
    // tags の無いノートで、会議 を付けて確定(検索が空なら Enter で確定)。
    open_tags(&mut a, 5);
    typing(&mut a, "会");
    select(&mut a, "会議");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(pending(&a, 5, "tags"), list(&["会議"]));
    // 一覧に無い 新規 を打って Enter で足し、もう一度 Enter で確定 → 新規 が足される。
    open_tags(&mut a, 5);
    typing(&mut a, "新規");
    assert_eq!(shown(&a), vec!["+新規"]);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::ListPick);
    assert_eq!(a.pick.as_ref().unwrap().query, "");
    assert!(shown(&a).contains(&"[x] 新規 0".to_string()));
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(pending(&a, 5, "tags"), list(&["会議", "新規"]));
}

#[test]
fn test_ce_19_hash_and_no_duplicates() {
    // [CE-19] tags の列で `#会議` と打つ → 会議 として扱い、既にあれば足さない(新規を出さない)。
    let (_t, mut a) = make("ce19hash", TAGS);
    open_tags(&mut a, 2);
    typing(&mut a, "#会議");
    assert_eq!(shown(&a), vec!["[x] 会議 5"]);
    // Enter(検索中は付けるだけ)→ 既に付いているので何もせず、検索が空に戻る。もう一度 Enter で確定 → 何もためない。
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::ListPick);
    assert_eq!(a.pick.as_ref().unwrap().query, "");
    assert_eq!(shown(&a)[0], "[x] 会議 5");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.changes.count(), 0);
    // `#` だけ打って Enter → 検索が空と同じ(確定)。何も変わらない。
    open_tags(&mut a, 2);
    typing(&mut a, "#");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.changes.count(), 0);
    // 新規も `#` を除いて足す。足した要素をもう一度打っても2つ足さない。
    open_tags(&mut a, 2);
    typing(&mut a, "#新規");
    assert_eq!(shown(&a), vec!["+新規"]);
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "新規");
    assert_eq!(shown(&a), vec!["[x] 新規 0"]);
    press(&mut a, KeyCode::Tab);
    assert_eq!(pending(&a, 2, "tags"), list(&["会議", "新規"]));
    // [CE-16] 打った文字は大文字小文字を区別しない部分一致。全角の文字は全角のまま入る(SR-17)。
    let (_t2, mut b) = make(
        "ce19case",
        &[
            ("a.md", "---\ntags: [Meeting, Ｂook]\n---\n"),
            ("b.md", "---\ntags: [other]\n---\n"),
        ],
    );
    open_tags(&mut b, 1);
    typing(&mut b, "mee");
    assert_eq!(shown(&b), vec!["+mee", "[ ] Meeting 1"]);
    press(&mut b, KeyCode::Backspace);
    press(&mut b, KeyCode::Backspace);
    press(&mut b, KeyCode::Backspace);
    typing(&mut b, "ｂ");
    assert_eq!(b.pick.as_ref().unwrap().query, "ｂ");
    assert_eq!(shown(&b), vec!["+ｂ", "[ ] Ｂook 1"]);
}

#[test]
fn test_ce_19_remove_all_writes_key() {
    // [CE-19] 要素を全部外して保存 → `tags:` の行が残る(キーは消さない。CE-9)。
    let (t, mut a) = make("ce19empty", TAGS);
    open_tags(&mut a, 2);
    space(&mut a);
    assert_eq!(shown(&a)[0], "[ ] 会議 5");
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 2, "tags"), list(&[]));
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Confirm);
    press(&mut a, KeyCode::Enter);
    assert_eq!(read(&t, "n3.md"), "---\ntitle: 三\ntags:\n---\n");
}

#[test]
fn test_ce_19_read_only_list_not_opened() {
    // [CE-19] [CE-8] 2行にまたがるフローのリストのセルで Enter → 開かずに理由。
    let (_t, mut a) = make(
        "ce19lock",
        &[
            ("a.md", "---\ntags: [a,\n b]\n---\n"),
            ("b.md", "---\ntags: [c]\n---\n"),
        ],
    );
    col_named(&mut a, "tags");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    let m = a.message.clone().unwrap();
    assert!(m.starts_with("読むだけ"), "{m}");
}

#[test]
fn test_ce_1_esc_and_ce_11_revert() {
    // [CE-1] Esc → 何もためずに表へ。
    let (_t, mut a) = make("ce16esc", TAGS);
    open_tags(&mut a, 2);
    space(&mut a);
    typing(&mut a, "x");
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    assert!(a.pick.is_none());
    assert_eq!(a.changes.count(), 0);
    // [CE-11] Ctrl+R → 開いたときの状態に戻る(足した新規も消える)。そのまま確定しても何も書かない。
    open_tags(&mut a, 2);
    space(&mut a);
    typing(&mut a, "新規");
    press(&mut a, KeyCode::Enter);
    select(&mut a, "本");
    space(&mut a);
    typing(&mut a, "本");
    ctrl(&mut a, 'r');
    let p = a.pick.as_ref().unwrap();
    assert_eq!(p.query, "");
    assert!(p.cands.iter().all(|c| c.mark == c.initial && !c.added));
    assert_eq!(
        shown(&a),
        vec!["[x] 会議 5", "[ ] 本 2", "[ ] 会計 1", "[ ] 旅行 1"]
    );
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.changes.count(), 0);
}

/// 3行(p1〜p3)を選ぶ材料。p4 は 会議 を候補に出すため。
const BULK: &[(&str, &str)] = &[
    ("p1.md", "---\ntags: [本, 秋]\n---\n"),
    ("p2.md", "---\ntags:\n  - 映画\n---\n"),
    ("p3.md", "---\ntags: [旅行]\n---\n"),
    ("p4.md", "---\ntags: [会議]\n---\n"),
];

#[test]
fn test_ce_17_bulk_add_and_remove() {
    // [CE-17] 3行を選び、tags に 会議 を足す → 3行とも足され、元のタグは残る。
    let (t, mut a) = make("ce17", BULK);
    col_named(&mut a, "tags");
    for _ in 0..3 {
        space(&mut a);
    }
    press(&mut a, KeyCode::Up);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::ListPick);
    let p = a.pick.as_ref().unwrap();
    assert!(p.bulk && p.targets.len() == 3);
    // 1行だけが持つ 本 は「一部」の印 [-]。誰も持たない 会議 は [ ]。
    let mark = |a: &App, n: &str| {
        let p = a.pick.as_ref().unwrap();
        p.cands.iter().find(|c| c.name == n).unwrap().mark
    };
    assert_eq!(mark(&a, "本"), Mark::Some);
    assert_eq!(mark(&a, "会議"), Mark::None);
    let s = screen(&a);
    assert!(s.contains("[-] 本") && s.contains("一括 3行"), "{s}");
    golden("ce_17", &s);
    assert_fits(&mut a);
    select(&mut a, "会議");
    space(&mut a);
    assert_eq!(mark(&a, "会議"), Mark::All);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert!(a.message.clone().unwrap().contains("一括: 3行を直した"));
    assert_eq!(pending(&a, 0, "tags"), list(&["本", "秋", "会議"]));
    assert_eq!(pending(&a, 1, "tags"), list(&["映画", "会議"]));
    assert_eq!(pending(&a, 2, "tags"), list(&["旅行", "会議"]));
    assert_eq!(pending(&a, 3, "tags"), None);
    // 本 を外す: Space で [-] → [x] → [ ]。確定 → その1行から 本 が消える(ほかの行は変わらない)。
    press(&mut a, KeyCode::Enter);
    assert_eq!(mark(&a, "会議"), Mark::All);
    select(&mut a, "本");
    space(&mut a);
    assert_eq!(mark(&a, "本"), Mark::All);
    space(&mut a);
    assert_eq!(mark(&a, "本"), Mark::None);
    space(&mut a);
    assert_eq!(mark(&a, "本"), Mark::Some);
    space(&mut a);
    space(&mut a);
    press(&mut a, KeyCode::Tab);
    assert_eq!(pending(&a, 0, "tags"), list(&["秋", "会議"]));
    assert_eq!(pending(&a, 1, "tags"), list(&["映画", "会議"]));
    // 触らずに閉じたら何も書かない(1手も足さない)。
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Enter);
    ctrl(&mut a, 'z');
    assert_eq!(pending(&a, 0, "tags"), list(&["本", "秋", "会議"]));
    ctrl(&mut a, 'r');
    // [CE-18] 保存の差分とファイル: フローは1行のまま、縦のリストは同じ字下げで1行増える。
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Confirm);
    let s = screen(&a);
    assert!(s.contains("+  - 会議"), "{s}");
    golden("ce_17_review", &s);
    press(&mut a, KeyCode::Enter);
    assert_eq!(read(&t, "p1.md"), "---\ntags: [秋, 会議]\n---\n");
    assert_eq!(read(&t, "p2.md"), "---\ntags:\n  - 映画\n  - 会議\n---\n");
    assert_eq!(read(&t, "p3.md"), "---\ntags: [旅行, 会議]\n---\n");
    assert_eq!(read(&t, "p4.md"), BULK[3].1);
}

#[test]
fn test_ce_17_bulk_skips_read_only_rows() {
    // [CE-17] [CE-10] 選んだ行のうち書けない行(読めないフロントマターのノート。WB-5)は飛ばし、数と理由を出す。
    // [WB-3] フロントマターの無いノート(d.md)は飛ばさず、ためる変更になる。
    let (_t, mut a) = make(
        "ce17skip",
        &[
            ("a.md", "---\ntags: [x]\n---\n"),
            ("b.md", "---\ntags: [w]\n本文\n"),
            ("c.md", "---\ntags: [y]\n---\n"),
            ("d.md", "本文だけ\n"),
        ],
    );
    for (i, l) in ["a.md", "b.md", "c.md", "d.md"].iter().enumerate() {
        assert_eq!(a.src.label(&a.rows[i]), *l);
    }
    col_named(&mut a, "tags");
    ctrl(&mut a, 'a');
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::ListPick);
    assert_eq!(a.pick.as_ref().unwrap().targets.len(), 3);
    typing(&mut a, "z");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Enter);
    let m = a.message.clone().unwrap();
    assert!(
        m.contains("一括: 3行を直した") && m.contains("1行を飛ばした"),
        "{m}"
    );
    assert_eq!(pending(&a, 0, "tags"), list(&["x", "z"]));
    assert_eq!(pending(&a, 1, "tags"), None);
    assert_eq!(pending(&a, 2, "tags"), list(&["y", "z"]));
    assert_eq!(pending(&a, 3, "tags"), list(&["z"]));
}

#[test]
fn test_nv_6_detail_enter_opens_pick() {
    // [NV-6] 詳細の表示の Enter からも開き、Esc で詳細の表示へ戻る。
    let (_t, mut a) = make("ce16detail", TAGS);
    a.row = 2;
    col_named(&mut a, "tags");
    press(&mut a, KeyCode::Char('K'));
    assert_eq!(a.mode, Mode::Detail);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::ListPick);
    let s = screen(&a);
    assert!(s.contains("[x] 会議  5件") && s.contains("詳細"), "{s}");
    assert_fits(&mut a);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Detail);
    press(&mut a, KeyCode::Enter);
    select(&mut a, "本");
    space(&mut a);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Detail);
    assert_eq!(pending(&a, 2, "tags"), list(&["会議", "本"]));
}

#[test]
fn test_sr_9_window_fits_small_terminal() {
    // [SR-9] 候補の窓は画面の幅と高さに収める(下に入らなければ上、どちらも狭ければ候補を減らす)。
    let notes: Vec<(String, String)> = (0..30)
        .map(|i| {
            (
                format!("n{i:02}.md"),
                format!("---\ntags: [とても長いタグの名前その{i}, 共通]\n---\n"),
            )
        })
        .collect();
    let refs: Vec<(&str, &str)> = notes
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let (_t, mut a) = make("sr9pick", &refs);
    open_tags(&mut a, 0);
    assert_fits(&mut a);
    for (w, h) in [(80u16, 24u16), (30, 10), (20, 6), (12, 5), (8, 4)] {
        a.resize(w, h);
        let lines = view::render(&a, (w - 1) as usize, (h - 1) as usize);
        assert!(lines.len() <= (h - 1) as usize);
        for l in &lines {
            let lw: usize = l.spans.iter().map(|s| width::width(&s.content)).sum();
            assert!(lw <= (w - 1) as usize, "幅 {w} 高さ {h} で {lw}: {l:?}");
        }
        if let Some((x, y)) = view::cursor(&a) {
            assert!(x < w - 1 && y < h - 1, "{w}x{h}: {x},{y}");
        }
    }
    // 下の方の行では上に出す。
    a.resize(80, 24);
    a.apply(Action::Cancel);
    open_tags(&mut a, 17);
    let s = screen(&a);
    let at = |pat: &str| s.lines().position(|l| l.contains(pat)).unwrap();
    assert!(at("検索: ") < at("n17 "), "{s}");
}

#[test]
fn test_sr_16_list_pick_keys() {
    // [SR-4] [SR-16] リストの選択のキーは表にあり、文字1つのキーは割り当てない(打つ文字は検索に入る)。
    let keys: Vec<_> = BINDINGS
        .iter()
        .filter(|b| b.mode == Mode::ListPick)
        .collect();
    assert!(!keys.is_empty());
    for b in &keys {
        assert!(b.key.chars().count() > 1, "文字のキー: {}", b.key);
    }
    let k = |key| keymap::lookup(BINDINGS, Mode::ListPick, key);
    assert_eq!(k("Space"), Some(Action::Toggle));
    assert_eq!(k("Enter"), Some(Action::Run));
    assert_eq!(k("Tab"), Some(Action::Commit));
    assert_eq!(k("Esc"), Some(Action::Cancel));
    assert_eq!(k("Ctrl+r"), Some(Action::Revert));
    assert_eq!(Mode::by_name("list_select"), Some(Mode::ListPick));
    // 下の帯は表から。
    let (_t, mut a) = make("sr16pick", TAGS);
    open_tags(&mut a, 0);
    let s = screen(&a);
    assert!(
        s.contains("Space 付け外し") && s.contains("リストの選択"),
        "{s}"
    );
}

/// types.json で tags を List にした保管庫。
fn typed_tags(name: &str, notes: &[(&str, &str)]) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    std::fs::create_dir_all(tmp.notes().join(".obsidian")).unwrap();
    std::fs::write(
        tmp.notes().join(".obsidian/types.json"),
        r#"{"types": {"tags": "tags"}}"#,
    )
    .unwrap();
    for (n, t) in notes {
        tmp.write(n, t);
    }
    let a = app_of(&tmp, ColorMode::None);
    (tmp, a)
}

#[test]
fn test_ce_16_tab_never_opens_line_input_on_list() {
    // [CE-16] [CE-11] (a) Tab で隣へ移る道: 書けるリストのセルならリストの選択を開き、1行の入力は開かない。
    let (_t, mut a) = make(
        "ce16tab",
        &[
            ("a.md", "---\ntitle: a\n---\n"),
            ("b.md", "---\ntitle: b\ntags: [x]\n---\n"),
            ("c.md", "---\ntitle: c\ntags: [a,\n b]\n---\n"),
        ],
    );
    col_named(&mut a, "title");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    press(&mut a, KeyCode::Tab);
    assert_eq!(a.mode, Mode::ListPick);
    assert!(a.input.is_none());
    assert_eq!(a.cols[a.col], "tags");
    // 打った文字は検索に入り、`tags: foo` のような1つの値は書けない。
    typing(&mut a, "foo");
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.changes.count(), 0);
    // 書けないリスト(2行にまたがるフロー)の行では、Tab は tags を飛ばして理由を出す。
    a.row = 2;
    col_named(&mut a, "title");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Tab);
    assert_eq!(a.mode, Mode::Table);
    let m = a.message.clone().unwrap();
    assert!(
        m.contains("右に直せるセルが無い") && m.contains("読むだけ"),
        "{m}"
    );
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_ce_17_bulk_when_current_row_cannot_open() {
    // [CE-17] (b) 一括で今の行が開けない(`tags: 2024` は文字列でない1つの値)→ 1行の入力は開かず、
    // リストの選択を開いて開けない行を飛ばす。
    let (_t, mut a) = typed_tags(
        "ce17cur",
        &[
            ("a.md", "---\ntags: 2024\n---\n"),
            ("b.md", "---\ntags: [x]\n---\n"),
            ("c.md", "---\ntags:\n  - y\n---\n"),
        ],
    );
    col_named(&mut a, "tags");
    ctrl(&mut a, 'a');
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::ListPick);
    assert!(a.input.is_none());
    assert_eq!(a.pick.as_ref().unwrap().targets.len(), 2);
    typing(&mut a, "z");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Tab);
    let m = a.message.clone().unwrap();
    assert!(
        m.contains("一括: 2行を直した") && m.contains("文字列でない要素を含むリスト"),
        "{m}"
    );
    assert_eq!(pending(&a, 0, "tags"), None);
    assert_eq!(pending(&a, 1, "tags"), list(&["x", "z"]));
    assert_eq!(pending(&a, 2, "tags"), list(&["y", "z"]));
}

#[test]
fn test_ce_19_read_only_shapes_have_reasons() {
    // [CE-19] [CE-8] 書けない形のリストは開かず(表のモードのまま)、形ごとの理由を出す。
    let (_t, mut a) = typed_tags(
        "ce19shapes",
        &[
            ("a.md", "---\ntags:\n  - a\n  # c\n  - b\n---\n"),
            ("b.md", "---\ntags:\n  - a\n  - 2024\n---\n"),
            ("c.md", "---\ntags: |\n  x\n---\n"),
        ],
    );
    col_named(&mut a, "tags");
    for (row, reason) in [
        (0, "要素の間にコメントがあるリスト"),
        (1, "文字列でない要素を含むリスト"),
        (2, "複数行の値"),
    ] {
        a.row = row;
        press(&mut a, KeyCode::Enter);
        assert_eq!(a.mode, Mode::Table, "{row}");
        assert_eq!(
            a.message.as_deref(),
            Some(format!("読むだけ: {reason}").as_str())
        );
    }
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_ce_16_space_in_search() {
    // [CE-16] 検索が空でなければ Space は空白として入る。aliases の列で `new item` と打って Enter
    // → `new item` が1つの要素として足される。
    let (_t, mut a) = make(
        "ce16space",
        &[
            ("a.md", "---\naliases: [a, Meeting]\n---\n"),
            ("b.md", "---\naliases: [b]\n---\n"),
        ],
    );
    a.row = 1;
    col_named(&mut a, "aliases");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::ListPick);
    typing(&mut a, "new item");
    assert_eq!(a.pick.as_ref().unwrap().query, "new item");
    assert_eq!(shown(&a), vec!["+new item"]);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::ListPick);
    // [CE-19] tags 以外の列は大文字小文字を区別する(`meeting` は新規)。
    typing(&mut a, "meeting");
    assert_eq!(shown(&a), vec!["+meeting", "[ ] Meeting 1"]);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.changes.count(), 0);
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "new item");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Tab);
    assert_eq!(pending(&a, 1, "aliases"), list(&["b", "new item"]));
}

#[test]
fn test_ce_19_tags_case_insensitive_new() {
    // [CE-19] tags の列では、`Meeting` があるとき `meeting` は新規を出さず、既にある Meeting を選ぶ。
    let (_t, mut a) = make(
        "ce19tagcase",
        &[
            ("a.md", "---\ntags: [Meeting, meet]\n---\n"),
            ("b.md", "---\ntags: [b]\n---\n"),
        ],
    );
    open_tags(&mut a, 1);
    typing(&mut a, "meeting");
    assert_eq!(shown(&a), vec!["[ ] Meeting 1"]);
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Tab);
    assert_eq!(pending(&a, 1, "tags"), list(&["b", "Meeting"]));
}

#[test]
fn test_sr_3_commit_only_when_cell_drawn() {
    // [SR-3] 表から開いたリストの選択は、選んだセルが見えていなければ確定しない(理由を出して閉じない)。
    let (_t, mut a) = make("sr3pick", TAGS);
    open_tags(&mut a, 2);
    space(&mut a);
    a.resize(80, 4);
    press(&mut a, KeyCode::Tab);
    assert_eq!(a.mode, Mode::ListPick);
    assert!(a.message.clone().unwrap().contains("見えていない"));
    assert_eq!(a.changes.count(), 0);
    a.resize(80, 24);
    press(&mut a, KeyCode::Tab);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(pending(&a, 2, "tags"), list(&[]));
}

#[test]
fn test_ce_17_bulk_enter_in_search_only_attaches() {
    // [CE-17] [CE-19] 一括で、全部の行が持つ 会議 を打って Enter 2回 → 付けるだけなので何も変わらない。
    let (_t, mut a) = make("ce17enter", TAGS);
    col_named(&mut a, "tags");
    for _ in 0..3 {
        space(&mut a);
    }
    press(&mut a, KeyCode::Up);
    press(&mut a, KeyCode::Enter);
    assert!(a.pick.as_ref().unwrap().bulk);
    typing(&mut a, "会議");
    press(&mut a, KeyCode::Enter);
    assert_eq!(shown(&a)[0], "[x] 会議 5");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.changes.count(), 0);
    // 一部の行だけが持つ 本 は、検索中の Enter で [-] → [x](外れない)。
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.pick.as_ref().unwrap().cands[1].mark, Mark::Some);
    typing(&mut a, "本");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.pick.as_ref().unwrap().cands[1].mark, Mark::All);
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 2, "tags"), list(&["会議", "本"]));
    assert_eq!(pending(&a, 0, "tags"), None);
}

#[test]
fn test_ce_19_tags_case_prefers_held_spelling() {
    // [CE-19] tags で Meeting(2件)と、この行が持つ meeting がある → `meeting` と打つと行が持つ meeting を選び、
    // Enter・Tab で同じ要素を2つにしない。一覧から Meeting を付けても、大文字小文字違いで持つので足さない。
    let (_t, mut a) = make(
        "ce19held",
        &[
            ("a.md", "---\ntags: [Meeting, y]\n---\n"),
            ("a2.md", "---\ntags: [Meeting]\n---\n"),
            ("b.md", "---\ntags: [meeting, x]\n---\n"),
        ],
    );
    open_tags(&mut a, 2);
    typing(&mut a, "MEETING");
    assert_eq!(shown(&a)[0], "[x] meeting 1");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Tab);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.changes.count(), 0);
    open_tags(&mut a, 2);
    select(&mut a, "Meeting");
    space(&mut a);
    press(&mut a, KeyCode::Tab);
    assert_eq!(pending(&a, 2, "tags"), None);
    // 持たない行では、同じ綴りが無ければ件数の多い Meeting を選ぶ。
    let (_t2, mut b) = make(
        "ce19held2",
        &[
            ("a.md", "---\ntags: [Meeting]\n---\n"),
            ("a2.md", "---\ntags: [Meeting]\n---\n"),
            ("b.md", "---\ntags: [meeting]\n---\n"),
            ("c.md", "---\ntags: [x]\n---\n"),
        ],
    );
    open_tags(&mut b, 3);
    typing(&mut b, "MEETING");
    assert_eq!(shown(&b)[0], "[ ] Meeting 2");
    press(&mut b, KeyCode::Enter);
    press(&mut b, KeyCode::Tab);
    assert_eq!(pending(&b, 3, "tags"), list(&["x", "Meeting"]));
}
