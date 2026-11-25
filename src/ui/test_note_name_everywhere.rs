//! ノートの名前を見せる所(関係マップのつながった行とよく指される行・木の画面の葉・横断の検索の結果・リンクの列の
//! 候補)にも、ノートの欄と同じ `display.note_name` を当てる(screen.note-name-everywhere)試験。値は作ったもの。
//!
//! 期待値は要求の受け入れの例から取った: `20261121_画面` が `parent: "[[20261120_設計]]"` で `20261120_設計` を指す表。
//! 木の画面の note_path とリンクの列の候補は 2026-11-25 に見つけた所で、受け入れの例は人が書き足す。
//! 画面の文字で確かめる(行と桁の位置は決め打ちしない)。

use super::app::{App, ColorMode};
use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, press, text, typing, Tmp};
use super::width::width;
use mdgrid::config::{parse, Config};
use mdgrid::places::{self, Place};
use mdgrid::source::markdown::Markdown;
use mdgrid::workspace::{Scope, Source, WsTable};
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;
use std::time::{Duration, Instant};

/// 日付の頭を隠す設定(受け入れの例)。
const HEAD: &str = "[display]\nnote_name = '^\\d{8}_'\n";

/// `20261121_画面` が parent で `20261120_設計` を指す(関係マップ)。
const LINKED: &[(&str, &str)] = &[
    (
        "20261121_画面.md",
        "---\nparent: \"[[20261120_設計]]\"\n---\n",
    ),
    ("20261120_設計.md", "---\nstatus: todo\n---\n"),
];

/// 同じ名前のノートで、どちらも folder は「仕様」(木の画面の階層。値に日付の頭を含まない)。
const FOLDERED: &[(&str, &str)] = &[
    ("20261121_画面.md", "---\nfolder: 仕様\n---\n"),
    ("20261120_設計.md", "---\nfolder: 仕様\n---\n"),
];

/// 設定の TOML を読む(警告が無いことも確かめる)。
fn cfg(text: &str) -> Config {
    let (c, w) = parse(text).expect("設定の TOML は読める");
    assert!(w.is_empty(), "設定の警告: {w:?}");
    c
}

/// `notes` を置いたフォルダを登録した表「Notes」にし、印の [[table]] に `line` を書いて、設定 `config` で開く。
fn boot(tag: &str, notes: &[(&str, &str)], config: Config, line: &str) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    for (n, t) in notes {
        if let Some((dir, _)) = n.rsplit_once('/') {
            std::fs::create_dir_all(tmp.notes().join(dir)).unwrap();
        }
        tmp.write(n, t);
    }
    let conf = tmp.0.join("config");
    std::fs::create_dir_all(&conf).unwrap();
    places::save(
        &conf,
        Place {
            name: "Notes".into(),
            group: String::new(),
            path: tmp.notes().canonicalize().unwrap(),
            view: None,
        },
    )
    .unwrap();
    std::fs::create_dir_all(tmp.0.join(".mdgrid")).unwrap();
    std::fs::write(
        tmp.0.join(".mdgrid/workspace.toml"),
        format!("[[table]]\nname = \"Notes\"\npath = \"notes\"\n{line}\n"),
    )
    .unwrap();
    let dir = tmp.notes();
    let src = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(160, 40);
    a.start(Startup {
        config,
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(conf),
        target: dir,
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    (tmp, a)
}

/// App の大きさで描いた画面の文字。
fn screen(a: &App) -> String {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    text(t.backend().buffer())
}

/// 行を語に分ける(窓の縦の辺と選びの印 `>` は語に数えない)。
fn words(line: &str) -> Vec<String> {
    line.split(|c: char| c.is_whitespace() || "│┃|".contains(c))
        .map(|w| w.trim_start_matches('>'))
        .filter(|w| !w.is_empty())
        .map(str::to_string)
        .collect()
}

/// 画面のどこかの行に、語の並び `seq` が続けて出るか。
fn has_words(s: &str, seq: &[&str]) -> bool {
    s.lines().any(|l| {
        let w = words(l);
        w.windows(seq.len())
            .any(|x| x.iter().zip(seq).all(|(a, b)| a == b))
    })
}

/// 関係マップを開き、Notes.parent → Notes のつながりを選ぶ。
fn open_parent_link(a: &mut App) {
    ch(a, 'R');
    assert_eq!(a.mode, Mode::Relations, "{:?}", a.message);
    press(a, KeyCode::Right);
    let rm = a.relmap.as_ref().expect("関係マップ");
    let k = rm.link.expect("つながりを選んでいる");
    assert_eq!(
        (rm.links[k].from.as_str(), rm.links[k].column.as_str()),
        ("Notes", "parent")
    );
}

/// 詳しく見る窓の「よく指される」の次の行の、同じ桁からの最初の語。
fn first_top_target(s: &str) -> String {
    let head = mdgrid::i18n::Msg::RelMapTop.text();
    let lines: Vec<&str> = s.lines().collect();
    let y = lines
        .iter()
        .position(|l| l.contains(head))
        .unwrap_or_else(|| panic!("「{head}」が無い:\n{s}"));
    let x = width(&lines[y][..lines[y].find(head).unwrap()]);
    let next = lines.get(y + 1).copied().unwrap_or_default();
    // 桁 x より右(幅2の字を数える)。
    let mut col = 0;
    let rest: String = next
        .chars()
        .skip_while(|c| {
            let skip = col < x;
            col += width(&c.to_string());
            skip
        })
        .collect();
    words(&rest).into_iter().next().unwrap_or_default()
}

#[test]
fn test_note_name_relation_map_linked_rows_and_top_targets() {
    // [utest -> req~screen.note-name-everywhere~2] note_name = '^\d{8}_' で関係マップの parent のつながりを選ぶと、
    // つながった行は「画面 → 設計」、よく指される行は「設計」。
    let (_t, mut a) = boot("nne_relmap", LINKED, cfg(HEAD), "");
    open_parent_link(&mut a);
    let s = screen(&a);
    assert!(has_words(&s, &["画面", "→", "設計"]), "{s}");
    assert_eq!(first_top_target(&s), "設計", "{s}");
    assert!(!s.contains("20261120_") && !s.contains("20261121_"), "{s}");
}

#[test]
fn test_note_name_relation_map_without_setting_keeps_names() {
    // [utest -> req~screen.note-name-everywhere~2] display.note_name を書かない表の関係マップのつながった行は
    // 「20261121_画面 → 20261120_設計」のまま。
    let (_t, mut a) = boot("nne_relmap_raw", LINKED, Config::default(), "");
    open_parent_link(&mut a);
    let s = screen(&a);
    assert!(
        has_words(&s, &["20261121_画面", "→", "20261120_設計"]),
        "{s}"
    );
    assert_eq!(first_top_target(&s), "20261120_設計", "{s}");
}

/// 木の画面の右の欄の行の最初の語(ノートの名前)。選んだフォルダの行だけが出る。
fn tree_leaves(s: &str, names: &[&str]) -> Vec<String> {
    s.lines()
        .flat_map(words)
        .filter(|w| names.iter().any(|n| w.contains(n)))
        .collect()
}

#[test]
fn test_note_name_tree_leaf() {
    // [utest -> req~screen.note-name-everywhere~2] 同じ設定で木の設定のある表の 20261120_設計 は、木の画面の葉の名前が「設計」
    // (ノートの欄と同じ見せ方。`.md` も付けない)。
    let (_t, mut a) = boot(
        "nne_tree",
        FOLDERED,
        cfg(HEAD),
        "tree = { path = \"folder\" }",
    );
    ch(&mut a, 'T');
    assert_eq!(a.mode, Mode::Tree, "{:?}", a.message);
    let s = screen(&a);
    let mut leaves = tree_leaves(&s, &["設計", "画面"]);
    leaves.sort();
    assert_eq!(leaves, ["画面", "設計"], "{s}");
}

#[test]
fn test_note_name_tree_leaf_follows_note_path() {
    // [utest -> req~screen.note-name-everywhere~2] 木の画面の葉も、ノートの欄と同じく note_path に従う:
    // note_path = "name" で Sites/コードの置き場.md の葉は「コードの置き場」。
    let notes = [
        ("Sites/コードの置き場.md", "---\nfolder: 道具\n---\n"),
        ("Docs/説明書.md", "---\nfolder: 道具\n---\n"),
    ];
    let tmp_notes: Vec<(&str, &str)> = notes.to_vec();
    let (_t, mut a) = boot(
        "nne_tree_path",
        &tmp_notes,
        cfg("[display]\nnote_path = \"name\"\n"),
        "tree = { path = \"folder\" }",
    );
    ch(&mut a, 'T');
    assert_eq!(a.mode, Mode::Tree, "{:?}", a.message);
    let s = screen(&a);
    let mut leaves = tree_leaves(&s, &["コードの置き場", "説明書"]);
    leaves.sort();
    assert_eq!(leaves, ["コードの置き場", "説明書"], "{s}");
}

/// 横断の検索で `word` を探し、読み終えるまで待つ(上限 10 秒)。
fn across(a: &mut App, word: &str) {
    press(a, KeyCode::Char(':'));
    typing(a, "across_search");
    press(a, KeyCode::Enter);
    typing(a, word);
    press(a, KeyCode::Enter);
    let end = Instant::now() + Duration::from_secs(10);
    while a.across_busy() && Instant::now() < end {
        a.across_poll();
        std::thread::sleep(Duration::from_millis(10));
    }
    a.across_poll();
}

/// 横断の検索の結果の行(「名前 ノート: 値」)の名前。
fn across_names(s: &str) -> Vec<String> {
    let head = format!("{}:", super::view::LABEL_HEADER.text());
    s.lines()
        .map(words)
        .filter(|w| w.get(1) == Some(&head))
        .map(|w| w[0].clone())
        .collect()
}

#[test]
fn test_note_name_across_search_result() {
    // [utest -> req~screen.note-name-everywhere~2] 同じ設定のワークスペースで横断の検索で「設計」を探すと、結果の名前は「設計」。
    // 観点: 当てる文字はノートの名前の全部のまま(隠した頭の「20261120」でも当たる)。
    let (t, mut a) = boot(
        "nne_across",
        FOLDERED,
        cfg(&format!("{HEAD}[search]\nacross = true\n")),
        "",
    );
    a.scope = Some(Scope {
        name: "Work".into(),
        source: Source::Chosen,
        tables: vec![WsTable {
            name: "notes".into(),
            path: t.notes().canonicalize().unwrap(),
            ..WsTable::default()
        }],
        profile: Default::default(),
        warnings: Vec::new(),
    });
    across(&mut a, "設計");
    let s = screen(&a);
    assert_eq!(across_names(&s), vec!["設計".to_string()], "{s}");
    press(&mut a, KeyCode::Esc);
    across(&mut a, "20261120");
    let s = screen(&a);
    assert_eq!(across_names(&s), vec!["設計".to_string()], "{s}");
}

/// `20261121_画面` の parent(1つのリンク)と related(リンクのリスト)が `20261120_設計` を指す。
const LINK_CELLS: &[(&str, &str)] = &[
    (
        "20261121_画面.md",
        "---\nparent: \"[[20261120_設計]]\"\nrelated:\n  - \"[[20261120_設計]]\"\n---\n",
    ),
    ("20261120_設計.md", "---\nstatus: todo\n---\n"),
];

/// 名前に `name` を含むノートの行の、列 `col` のセルを選んで Enter で直し始める。
fn edit_link_cell(a: &mut App, name: &str, col: &str) {
    let at = a
        .rows
        .iter()
        .position(|r| a.src.label(r).contains(name))
        .unwrap_or_else(|| panic!("{name} の行が無い"));
    a.row = a
        .slots
        .iter()
        .position(|s| *s == super::grid::Slot::Row(at))
        .unwrap();
    a.col = a.cols.iter().position(|c| c == col).unwrap();
    a.scroll_into_view();
    press(a, KeyCode::Enter);
}

#[test]
fn test_note_name_link_cell_candidates() {
    // [utest -> req~screen.note-name-everywhere~2] 同じ設定で、リンクの列(parent)のセルを直すときの候補の名前も
    // ノートの欄と同じ見せ方(「設計」「画面」)。書く値は今までどおり完全な名前のリンク。
    let (_t, mut a) = boot("nne_link_cell", LINK_CELLS, cfg(HEAD), "");
    edit_link_cell(&mut a, "画面", "parent");
    let list = a
        .input
        .as_ref()
        .and_then(|i| i.list.as_ref())
        .expect("リンクの候補のリスト");
    let item = |t: &str| list.items.iter().find(|i| i.text == t);
    let design = item("設計").unwrap_or_else(|| {
        let texts: Vec<&str> = list.items.iter().map(|i| i.text.as_str()).collect();
        panic!("「設計」の候補が無い: {texts:?}")
    });
    assert_eq!(
        design.value,
        mdgrid::writeback::NewValue::Str("[[20261120_設計]]".into())
    );
    assert!(item("画面").is_some());
    assert!(!list.items.iter().any(|i| i.text.starts_with("2026")));
}

#[test]
fn test_note_name_link_list_cell_candidates() {
    // [utest -> req~screen.note-name-everywhere~2] リンクのリストの列(related)の付け外しの候補の名前も同じ見せ方。
    // 付け外しで書く要素は今までどおり完全な名前のリンク。
    let (_t, mut a) = boot("nne_link_list", LINK_CELLS, cfg(HEAD), "");
    edit_link_cell(&mut a, "画面", "related");
    let pick = a.pick.as_ref().expect("リストの選択");
    let design = pick
        .cands
        .iter()
        .find(|c| c.label.as_deref() == Some("設計"))
        .unwrap_or_else(|| {
            let labels: Vec<_> = pick.cands.iter().map(|c| c.label.clone()).collect();
            panic!("「設計」の候補が無い: {labels:?}")
        });
    assert_eq!(design.name, "[[20261120_設計]]");
    assert!(pick
        .cands
        .iter()
        .any(|c| c.label.as_deref() == Some("画面")));
}

/// 名前に `name` を含むノートの行を選ぶ。
fn select_row(a: &mut App, name: &str) {
    let at = a
        .rows
        .iter()
        .position(|r| a.src.label(r).contains(name))
        .unwrap_or_else(|| panic!("{name} の行が無い"));
    a.row = a
        .slots
        .iter()
        .position(|s| *s == super::grid::Slot::Row(at))
        .unwrap();
    a.scroll_into_view();
}

#[test]
fn test_note_name_linked_rows_list() {
    // [utest -> req~screen.note-name-everywhere~2] 同じ設定で、「設計」の行で「つながった行」を開くと、一覧の行は
    // 「Notes › parent ← 画面」で、選ぶと 20261121_画面 の行を開く。
    let (_t, mut a) = boot("nne_linked_rows", LINKED, cfg(HEAD), "");
    select_row(&mut a, "設計");
    a.start_linked_rows();
    let s = screen(&a);
    assert!(has_words(&s, &["Notes", "›", "parent", "←", "画面"]), "{s}");
    assert!(!s.contains("20261121_"), "{s}");
    press(&mut a, KeyCode::Enter);
    let row = a.cur_row().expect("選んだ行");
    assert!(
        a.src.label(&row).contains("20261121_画面"),
        "{:?}",
        a.message
    );
}

#[test]
fn test_note_name_open_link_choices() {
    // [utest -> req~screen.note-name-everywhere~2] 同じ設定で、「画面」の related が 20261120_設計 と 20261119_要件 を
    // 指すとき、「行き先を開く」の選ぶ一覧の行は「設計」と「要件」。
    let notes = [
        (
            "20261121_画面.md",
            "---\nrelated:\n  - \"[[20261120_設計]]\"\n  - \"[[20261119_要件]]\"\n---\n",
        ),
        ("20261120_設計.md", "---\nstatus: todo\n---\n"),
        ("20261119_要件.md", "---\nstatus: todo\n---\n"),
    ];
    let (_t, mut a) = boot("nne_open_link", &notes, cfg(HEAD), "");
    select_row(&mut a, "画面");
    a.col = a.cols.iter().position(|c| c == "related").unwrap();
    a.start_open_link();
    let s = screen(&a);
    assert!(
        s.lines().any(|l| words(l).contains(&"設計".to_string())),
        "{s}"
    );
    assert!(
        s.lines().any(|l| words(l).contains(&"要件".to_string())),
        "{s}"
    );
    assert!(!s.contains("20261120_") && !s.contains("20261119_"), "{s}");
}
