//! タスク以外の台帳の実地の操作で見つかった穴の直し(2026-11-27): タブの `where` の値を新しいノートに入れる・値の
//! 候補を今の表から集めてくり返しの無い列には出さない・日時の見せ方・詳細のリンクの名前・切っても残る子の数・
//! カンバンのレーンの題の札。

use super::grid::Slot;
use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{buffer, ch, col_named, ctrl, press, screen, typing, Tmp};
use super::*;
use ratatui::crossterm::event::KeyCode;
use ratatui::style::Color;

/// 印に `marker` を書き、`files`(根からの相対のパスと中身)を置いて、設定 `config` で根のフォルダを開く。
fn boot(
    tag: &str,
    marker: &str,
    files: &[(&str, &str)],
    config: &str,
    color: ColorMode,
) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    for (rel, text) in files {
        let p = tmp.notes().join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
    let dir = tmp.notes().join(".mdgrid");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("workspace.toml"), marker).unwrap();
    let root = tmp.notes();
    let cdir = tmp.0.join("config");
    std::fs::create_dir_all(&cdir).unwrap();
    let (cfg, w) = mdgrid::config::parse(config).unwrap();
    assert!(w.is_empty(), "{w:?}");
    let (table, warns) = mdgrid::workspace::marker_open(&root);
    assert!(warns.is_empty(), "{warns:?}");
    let path = table.map_or(root, |t| t.path);
    let t = crate::open_target(std::slice::from_ref(&path), None).unwrap();
    let mut a = App::new(Box::new(t.src), color);
    a.resize(120, 30);
    a.start(Startup {
        config: cfg,
        warnings: warns,
        readonly: false,
        no_color: color == ColorMode::None,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(cdir),
        target: crate::state_target(std::slice::from_ref(&path)),
        base: t.base,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    (tmp, a)
}

/// 表「T」(T/ のノート)の印。`table` は表の項目の行、`views` はビューの区画(無ければ「全部」の1つ)。
fn marker(top: &str, table: &str, views: &str) -> String {
    let views = if views.is_empty() {
        "[[table.base.views]]\ntype = \"table\"\nname = \"全部\"\n"
    } else {
        views
    };
    format!(
        "open = \"T\"\n{top}\n\n[[table]]\nname = \"T\"\n{table}\n\n[table.new_note]\nfolder = \"T\"\n\n\
         [table.base]\nfilters = {{ and = ['file.inFolder(\"T\")'] }}\n\n{views}\n"
    )
}

/// 名前(拡張子なし)が `name` の行を選ぶ。
fn select(a: &mut App, name: &str) {
    a.row = (0..a.slots.len())
        .find(|&i| matches!(a.slots[i], Slot::Row(r) if a.src.label(&a.rows[r]).ends_with(&format!("{name}.md"))))
        .unwrap_or_else(|| panic!("{name} の行が無い:\n{}", screen(a)));
}

/// 値の候補のリストの項目の字(リストが無ければ None)。
fn list_items(a: &App) -> Option<Vec<String>> {
    a.active_list()
        .map(|l| l.items.iter().map(|i| i.text.clone()).collect())
}

#[test]
fn test_where_with_one_value_fills_the_new_note() {
    // [utest -> req~cell-edit.new-note-filter-values~1] 印のビューの `where = { status = ["later"] }`(値が1つ)のタブで新しいノートを
    // 作る → ノートに `status: later` が入り、作った行がそのタブに残る。
    let views = "[[table.base.views]]\ntype = \"table\"\nname = \"あとで\"\nwhere = { status = [\"later\"] }\norder = [\"status\"]\n";
    let (t, mut a) = boot(
        "fix_where1",
        &marker("", "", views),
        &[("T/a.md", "---\nstatus: later\n---\n")],
        "",
        ColorMode::None,
    );
    ch(&mut a, 'a');
    typing(&mut a, "新しい");
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Table, "{:?}\n{}", a.message, screen(&a));
    let body = std::fs::read_to_string(t.notes().join("T/新しい.md")).unwrap();
    assert!(body.lines().any(|l| l == "status: later"), "{body}");
    assert!(
        a.rows.iter().any(|r| a.src.label(r).ends_with("新しい.md")),
        "{}",
        screen(&a)
    );
}

#[test]
fn test_value_list_only_from_this_table() {
    // [utest -> req~cell-edit.value-list-open~3] 同じ根の表「サイト」だけが持つ status の値 archived は、表「T」の status の候補に無い。
    // T の値(todo が2件・doing が1件)は候補に出る。
    let m = format!(
        "{}\n[[table]]\nname = \"サイト\"\n\n[table.base]\nfilters = {{ and = ['file.inFolder(\"Sites\")'] }}\n\n[[table.base.views]]\ntype = \"table\"\nname = \"全部\"\n",
        marker("", "", "[[table.base.views]]\ntype = \"table\"\nname = \"全部\"\norder = [\"status\"]\n")
    );
    let (_t, mut a) = boot(
        "fix_vlist1",
        &m,
        &[
            ("T/a.md", "---\nstatus: todo\n---\n"),
            ("T/b.md", "---\nstatus: todo\n---\n"),
            ("T/c.md", "---\nstatus: doing\n---\n"),
            ("Sites/s.md", "---\nstatus: archived\n---\n"),
            ("Sites/u.md", "---\nstatus: archived\n---\n"),
        ],
        "",
        ColorMode::None,
    );
    select(&mut a, "a");
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit, "{:?}", a.message);
    let got = list_items(&a).unwrap_or_else(|| panic!("リストが無い:\n{}", screen(&a)));
    assert!(
        got.contains(&"todo".to_string()) && got.contains(&"doing".to_string()),
        "{got:?}"
    );
    assert!(!got.contains(&"archived".to_string()), "{got:?}");
}

#[test]
fn test_value_list_needs_a_repeated_value() {
    // [utest -> req~cell-edit.value-list-open~3] url が5行とも違う表で url のセルを直す → リストを出さず1行の入力。
    // くり返しのある列(kind)はリストを出す。
    let (_t, mut a) = boot(
        "fix_vlist2",
        &marker("", "", "[[table.base.views]]\ntype = \"table\"\nname = \"全部\"\norder = [\"url\", \"kind\"]\n"),
        &[
            ("T/a.md", "---\nurl: https://a.example.org/\nkind: tool\n---\n"),
            ("T/b.md", "---\nurl: https://b.example.org/\nkind: tool\n---\n"),
            ("T/c.md", "---\nurl: https://c.example.org/\nkind: doc\n---\n"),
            ("T/d.md", "---\nurl: https://d.example.org/\nkind: doc\n---\n"),
            ("T/e.md", "---\nurl: https://e.example.org/\nkind: doc\n---\n"),
        ],
        "",
        ColorMode::None,
    );
    select(&mut a, "a");
    col_named(&mut a, "url");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit, "{:?}", a.message);
    assert_eq!(list_items(&a), None, "{}", screen(&a));
    press(&mut a, KeyCode::Esc);
    col_named(&mut a, "kind");
    press(&mut a, KeyCode::Enter);
    let got = list_items(&a).unwrap_or_else(|| panic!("リストが無い:\n{}", screen(&a)));
    assert!(
        got.contains(&"tool".to_string()) && got.contains(&"doc".to_string()),
        "{got:?}"
    );
}

#[test]
fn test_list_items_only_from_this_table() {
    // [utest -> req~cell-edit.list-items-candidates~2] `[rules.tags.candidates]` を書かなくても、表「T」の tags の候補に、
    // 同じ根の表「サイト」のタグ「料理」は出ない。T のタグ「仕事」は出る。
    let m = format!(
        "{}\n[[table]]\nname = \"サイト\"\n\n[table.base]\nfilters = {{ and = ['file.inFolder(\"Sites\")'] }}\n\n[[table.base.views]]\ntype = \"table\"\nname = \"全部\"\n",
        marker("[types]\ntags = \"list\"", "", "[[table.base.views]]\ntype = \"table\"\nname = \"全部\"\norder = [\"tags\"]\n")
    );
    let (_t, mut a) = boot(
        "fix_list1",
        &m,
        &[
            ("T/a.md", "---\ntags:\n  - 仕事\n---\n"),
            ("Sites/s.md", "---\ntags:\n  - 料理\n---\n"),
        ],
        "",
        ColorMode::None,
    );
    select(&mut a, "a");
    col_named(&mut a, "tags");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::ListPick, "{:?}", a.message);
    let p = a.pick.as_ref().unwrap();
    let names: Vec<String> = p.cands.iter().map(|c| c.name.clone()).collect();
    assert!(names.contains(&"仕事".to_string()), "{names:?}");
    assert!(!names.contains(&"料理".to_string()), "{names:?}");
}

/// 日時の列 at の表(`at` の値ごとに1行)。
fn datetime_table(tag: &str, config: &str) -> (Tmp, App) {
    boot(
        tag,
        &marker(
            "[types]\nat = \"datetime\"",
            "kanban = { column = \"kind\", card = [\"at\"] }",
            "[[table.base.views]]\ntype = \"table\"\nname = \"全部\"\norder = [\"at\", \"kind\"]\n",
        ),
        &[
            ("T/会議.md", "---\nat: 2026-11-25T16:00\nkind: mtg\n---\n"),
            ("T/休み.md", "---\nat: 2026-11-26\nkind: off\n---\n"),
        ],
        config,
        ColorMode::None,
    )
}

#[test]
fn test_datetime_is_shown_with_a_space() {
    // [utest -> req~cell-view.datetime-display~1] `at: 2026-11-25T16:00` は表・詳細・カンバンの札で `2026-11-25 16:00`。
    // 日付だけの `2026-11-26` はそのまま。
    let (_t, mut a) = datetime_table("fix_dt1", "");
    let s = screen(&a);
    assert!(
        s.contains("2026-11-25 16:00") && !s.contains("2026-11-25T16:00"),
        "{s}"
    );
    assert!(s.contains("2026-11-26"), "{s}");
    select(&mut a, "会議");
    ch(&mut a, 'K');
    let s = screen(&a);
    assert!(
        s.contains("2026-11-25 16:00") && !s.contains("T16:00"),
        "{s}"
    );
    press(&mut a, KeyCode::Esc);
    ch(&mut a, 'B');
    assert_eq!(a.mode, Mode::Kanban, "{:?}", a.message);
    let s = screen(&a);
    assert!(
        s.contains("2026-11-25 16:00") && !s.contains("T16:00"),
        "{s}"
    );
}

#[test]
fn test_datetime_date_part_follows_the_date_format() {
    // [utest -> req~cell-view.datetime-display~1] 設定 `dates.format = "YYYY/MM/DD"` → `2026/11/25 16:00`。
    let (_t, a) = datetime_table("fix_dt2", "[dates]\nformat = \"YYYY/MM/DD\"\n");
    let s = screen(&a);
    assert!(s.contains("2026/11/25 16:00"), "{s}");
}

#[test]
fn test_datetime_edit_starts_from_the_note_value() {
    // [utest -> req~cell-view.datetime-display~1] `2026-11-25 16:00` のセルを直し始める → 欄はノートの値 `2026-11-25T16:00`。
    // 直さずに確定すると、ノートは1バイトも変わらない。
    let (t, mut a) = datetime_table("fix_dt3", "");
    let before = std::fs::read(t.notes().join("T/会議.md")).unwrap();
    select(&mut a, "会議");
    col_named(&mut a, "at");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit, "{:?}", a.message);
    assert_eq!(a.input.as_ref().unwrap().text, "2026-11-25T16:00");
    // カレンダーが出ていれば閉じてから確定する。
    press(&mut a, KeyCode::Enter);
    if a.mode == Mode::Edit {
        press(&mut a, KeyCode::Enter);
    }
    ctrl(&mut a, 's');
    if a.mode == Mode::Confirm {
        press(&mut a, KeyCode::Enter);
    }
    assert_eq!(std::fs::read(t.notes().join("T/会議.md")).unwrap(), before);
}

#[test]
fn test_detail_shows_link_names_like_the_cell() {
    // [utest -> req~navigation.detail-same-text~1] `related: ["[[設計]]", "[[画面]]"]` の行の詳細 → `[設計, 画面]`(`[[[` は出ない)。
    let (_t, mut a) = boot(
        "fix_detail1",
        &marker(
            "[types]\nrelated = \"list\"",
            "",
            "[[table.base.views]]\ntype = \"table\"\nname = \"全部\"\norder = [\"related\"]\n",
        ),
        &[
            ("T/設計.md", "---\nkind: a\n---\n"),
            ("T/画面.md", "---\nkind: a\n---\n"),
            (
                "T/問い.md",
                "---\nrelated:\n  - \"[[設計]]\"\n  - \"[[画面]]\"\n---\n",
            ),
        ],
        "",
        ColorMode::Rgb,
    );
    select(&mut a, "問い");
    ch(&mut a, 'K');
    let s = screen(&a);
    assert!(s.contains("[設計, 画面]"), "{s}");
    assert!(!s.contains("[[["), "{s}");
}

#[test]
fn test_child_count_survives_a_cut_name() {
    // [utest -> req~navigation.parent-child-count~2] ノートの欄に入らない長い名前の親 → 名前を `…` で切り、`(1)` は欄に残る。
    let long = "家の光熱費を年に3万円下げられるかを調べてまとめる";
    let file = format!("T/{long}.md");
    let child = format!("---\nparent: \"[[{long}]]\"\n---\n");
    let (_t, mut a) = boot(
        "fix_count1",
        &marker(
            "",
            "children = { key = \"parent\" }",
            "[[table.base.views]]\ntype = \"table\"\nname = \"全部\"\norder = [\"kind\"]\n",
        ),
        &[
            (file.as_str(), "---\nkind: q\n---\n"),
            ("T/電気.md", child.as_str()),
        ],
        "",
        ColorMode::None,
    );
    a.resize(44, 20);
    let s = screen(&a);
    let line = s
        .lines()
        .find(|l| l.contains("家の"))
        .unwrap_or_else(|| panic!("{s}"));
    assert!(line.contains('…') && line.contains("(1)"), "{line:?}\n{s}");
}

#[test]
fn test_kanban_lane_title_is_drawn_as_the_cell_part() {
    // [utest -> req~navigation.group-heading-parts~2] 札の形 chip と値の色 doing = #d6a85c → カンバンの doing のレーンの題の字の地は
    // #d6a85c で、札の数 (1) も出る。
    let (_t, mut a) = boot(
        "fix_lane1",
        &marker(
            "",
            "kanban = { column = \"status\" }",
            "[[table.base.views]]\ntype = \"table\"\nname = \"全部\"\norder = [\"status\"]\n",
        ),
        &[
            ("T/a.md", "---\nstatus: todo\n---\n"),
            ("T/b.md", "---\nstatus: doing\n---\n"),
            ("T/c.md", "---\nstatus: todo\n---\n"),
        ],
        "[look.style]\nstatus = \"chip\"\n\n[look.colors.values]\ndoing = \"#d6a85c\"\n",
        ColorMode::Rgb,
    );
    ch(&mut a, 'B');
    assert_eq!(a.mode, Mode::Kanban, "{:?}", a.message);
    let buf = buffer(&a);
    let (x, y) = (0..buf.area.height)
        .find_map(|y| {
            let line: String = (0..buf.area.width)
                .map(|x| buf[(x, y)].symbol().to_string())
                .collect();
            if !line.contains("(1)") || !line.contains("doing") {
                return None;
            }
            let at = line.find("doing")?;
            Some((line[..at].chars().count() as u16, y))
        })
        .unwrap_or_else(|| panic!("doing のレーンの題が無い:\n{}", screen(&a)));
    assert_eq!(
        buf[(x, y)].bg,
        Color::Rgb(0xd6, 0xa8, 0x5c),
        "{}",
        screen(&a)
    );
}
