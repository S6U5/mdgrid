//! 台帳のスターター(ST-1・ST-2・ST-3・ST-4)の単体の試験。`src/starter.rs` の中として
//! `#[cfg(test)] #[path = "test_starter_unit.rs"] mod tests;` で取り込む。実装を見ずに書いた。
//! 仕様: specs/starter/spec.md、記録: specs/_changes/2026-10-24-starter.md。
//!
//! 材料(作った値): ユーザーのスターター `kanban` を一時フォルダの設定の置き場 `config/starters/kanban/` に書く。
//!
//! | ファイル                                  | 中身                                                     |
//! |-------------------------------------------|----------------------------------------------------------|
//! | starter.toml                              | name en=Kanban ja=カンバン、description en/ja             |
//! | ledger/.mdgrid/workspace.toml             | open = "Board"、[[table]] Board → board                  |
//! | ledger/.mdgrid/templates/card.md          | `{name}`・`{date}`(雛形の変数。写すときは文字のまま)    |
//! | samples/board/First card.md               | `{date+3}`・`{date}`・`{time}`・`{date-10}`(見本の変数) |
//! | ledger.ja/ と samples.ja/(ja の変種のとき) | 印は open = "板"、雛形は fuda.md、見本は 最初の札.md       |
//!
//! 仮定(仕様に書かれていない細部):
//! - `StarterFile.rel` は台帳の根からの相対(`ledger/`・`samples/` の頭を除いたもの)。
//! - `create` の返すパスは、根からの相対に直せる(`root.join(rel)` か `rel` そのもの)。並びは問わない(揃えて比べる)。
//! - 書く先が1つでもあるときの Err の文に、そのファイルの名前が入る。
//! - 壊れた `starter.toml` の Err と警告は `starters/<名前>/starter.toml:` で始まる(CLI-3 の形)。

use super::*;
use crate::i18n::{scoped, Lang};
use std::path::{Path, PathBuf};

// ---- 材料 ----

const MANIFEST: &str = "[name]\nen = \"Kanban\"\nja = \"カンバン\"\n\n[description]\nen = \"Cards on a board\"\nja = \"板の上の札\"\n";
const MARKER_EN: &str = "open = \"Board\"\n\n[[table]]\nname = \"Board\"\npath = \"board\"\n";
const MARKER_JA: &str = "open = \"板\"\n\n[[table]]\nname = \"板\"\npath = \"board\"\n";
const TEMPLATE_EN: &str = "# {name}\n\nMade on {date}\n";
const TEMPLATE_JA: &str = "# {name}\n\n{date} に作った\n";
const SAMPLE_EN: &str =
    "---\ntitle: First card\ndue: \"{date+3}\"\n---\nWritten {date} at {time}, ten days after {date-10}\n";
const SAMPLE_JA: &str = "---\ntitle: 最初の札\ndue: \"{date+3}\"\n---\n{now} に書いた\n";

/// 試験ごとの一時フォルダ(実体のパス)。Drop で消す。
struct Tmp(PathBuf);

impl Tmp {
    fn new(name: &str) -> Tmp {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let d =
            std::env::temp_dir().join(format!("mdgrid-st-{name}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        Tmp(std::fs::canonicalize(&d).unwrap())
    }
}

impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn put(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

/// ユーザーのスターター `cfg/starters/<name>/` を書き、そのフォルダを返す。
/// `manifest` が None なら starter.toml を置かない。`ja` なら ledger.ja/ と samples.ja/ も置く。
fn user_starter(cfg: &Path, name: &str, manifest: Option<&str>, ja: bool) -> PathBuf {
    let dir = cfg.join("starters").join(name);
    if let Some(m) = manifest {
        put(&dir.join("starter.toml"), m);
    }
    put(&dir.join("ledger/.mdgrid/workspace.toml"), MARKER_EN);
    put(&dir.join("ledger/.mdgrid/templates/card.md"), TEMPLATE_EN);
    put(&dir.join("samples/board/First card.md"), SAMPLE_EN);
    if ja {
        put(&dir.join("ledger.ja/.mdgrid/workspace.toml"), MARKER_JA);
        put(
            &dir.join("ledger.ja/.mdgrid/templates/fuda.md"),
            TEMPLATE_JA,
        );
        put(&dir.join("samples.ja/board/最初の札.md"), SAMPLE_JA);
    }
    dir
}

/// 今日 2026-10-06 の 09:05(crate::clock::today_now と同じ形: 日数と地域の秒)。
fn now() -> (i64, i64) {
    let today = crate::types::parse_date("2026-10-06").unwrap();
    (today, today * 86_400 + 9 * 3600 + 5 * 60)
}

/// 返したパスを根からの相対にして揃える。
fn rels(root: &Path, made: &[PathBuf]) -> Vec<String> {
    let mut v: Vec<String> = made
        .iter()
        .map(|p| {
            p.strip_prefix(root)
                .unwrap_or(p.as_path())
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    v.sort();
    v
}

/// `root` の下の全部のファイル(根からの相対。揃える)。無ければ空。
fn files_under(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                out.push(p.strip_prefix(root).unwrap().to_string_lossy().into_owned());
            }
        }
    }
    out.sort();
    out
}

fn read(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

fn st_names(list: &[Starter]) -> Vec<&str> {
    list.iter().map(|s| s.name.as_str()).collect()
}

/// 部分(と言語の変種)で絞ったファイルの相対パス(揃える)。
fn file_rels(s: &Starter, ledger: bool, lang: Option<&str>) -> Vec<String> {
    let mut v: Vec<String> = s
        .files
        .iter()
        .filter(|f| matches!(f.part, Part::Ledger) == ledger)
        .filter(|f| f.lang.as_deref() == lang)
        .map(|f| f.rel.to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

fn strs(v: &[String]) -> Vec<&str> {
    v.iter().map(String::as_str).collect()
}

/// Err の文を取り出す(Ok なら落とす。`Starter` が Debug を持たなくてもよいように)。
fn must_fail<T>(r: Result<T, String>, what: &str) -> String {
    match r {
        Ok(_) => panic!("{what}: Ok が返った"),
        Err(e) => e,
    }
}

// ---- ST-1: スターターの形 ----

#[test]
fn test_st_1_reads_manifest_ledger_and_samples() {
    // [ST-1] starter.toml・ledger/・samples/ を持つユーザーのスターターが読め、名前と説明は言語ごとに、
    // ファイルは部分(印と雛形 / 見本)と台帳の根からの相対パスと中身をそのまま持つ。
    let t = Tmp::new("read");
    let dir = user_starter(&t.0.join("config"), "kanban", Some(MANIFEST), false);
    let s = read_user(&dir).unwrap_or_else(|e| panic!("読めない: {e}"));
    assert_eq!(s.name, "kanban");
    assert!(matches!(s.origin, Origin::User(_)), "ユーザーのもの");
    assert_eq!(s.title.get("en").map(String::as_str), Some("Kanban"));
    assert_eq!(s.title.get("ja").map(String::as_str), Some("カンバン"));
    assert_eq!(
        describe(&s, Lang::En),
        ("Kanban".to_string(), "Cards on a board".to_string())
    );
    assert_eq!(
        describe(&s, Lang::Ja),
        ("カンバン".to_string(), "板の上の札".to_string())
    );
    assert_eq!(
        strs(&file_rels(&s, true, None)),
        [".mdgrid/templates/card.md", ".mdgrid/workspace.toml"],
        "印と雛形は台帳の根からの相対"
    );
    assert_eq!(
        strs(&file_rels(&s, false, None)),
        ["board/First card.md"],
        "見本は台帳の根からの相対"
    );
    assert!(
        s.files.iter().all(|f| f.lang.is_none()),
        "言語の変種は無い: {:?}",
        s.files.iter().map(|f| f.lang.clone()).collect::<Vec<_>>()
    );
    let marker = s
        .files
        .iter()
        .find(|f| f.rel.as_path() == Path::new(".mdgrid/workspace.toml"))
        .unwrap();
    assert_eq!(marker.text, MARKER_EN, "印の中身はそのまま");
    let sample = s
        .files
        .iter()
        .find(|f| f.rel.as_path() == Path::new("board/First card.md"))
        .unwrap();
    assert_eq!(sample.text, SAMPLE_EN, "読んだだけでは見本の変数を埋めない");
}

#[test]
fn test_st_1_no_manifest_means_folder_name_and_empty_description() {
    // [ST-1] starter.toml が無ければ、名前はフォルダの名前、説明は空(どの言語でも)。読めはする。
    let t = Tmp::new("nomanifest");
    let dir = user_starter(&t.0.join("config"), "kanban", None, false);
    let s = read_user(&dir).unwrap_or_else(|e| panic!("読めない: {e}"));
    assert_eq!(s.name, "kanban");
    for lang in Lang::ALL {
        assert_eq!(
            describe(&s, *lang),
            ("kanban".to_string(), String::new()),
            "{}",
            lang.code()
        );
    }
}

#[test]
fn test_st_1_missing_language_falls_back_to_en_then_folder_name() {
    // [ST-1] その言語の名前・説明が無ければ en のもの、en も無ければフォルダの名前と空の説明。
    let t = Tmp::new("fallback");
    let cfg = t.0.join("config");
    let en_only = user_starter(
        &cfg,
        "enonly",
        Some("[name]\nen = \"Only English\"\n\n[description]\nen = \"No Japanese\"\n"),
        false,
    );
    let s = read_user(&en_only).unwrap_or_else(|e| panic!("読めない: {e}"));
    assert_eq!(
        describe(&s, Lang::Ja),
        ("Only English".to_string(), "No Japanese".to_string()),
        "ja が無ければ en"
    );
    let ja_only = user_starter(&cfg, "jaonly", Some("[name]\nja = \"名前だけ\"\n"), false);
    let s = read_user(&ja_only).unwrap_or_else(|e| panic!("読めない: {e}"));
    assert_eq!(
        describe(&s, Lang::Ja),
        ("名前だけ".to_string(), String::new()),
        "[description] が無ければ説明は空"
    );
    assert_eq!(
        describe(&s, Lang::En),
        ("jaonly".to_string(), String::new()),
        "en も無ければフォルダの名前"
    );
}

#[test]
fn test_st_1_lang_variant_replaces_whole_ledger_and_samples() {
    // [ST-1] ledger.ja/・samples.ja/ があれば、ja のときはその中身だけを使い、ledger/・samples/ は重ねない。
    // en(接尾なし)は ledger/・samples/ だけ。
    let t = Tmp::new("variant");
    let dir = user_starter(&t.0.join("config"), "kanban", Some(MANIFEST), true);
    let s = read_user(&dir).unwrap_or_else(|e| panic!("読めない: {e}"));
    assert_eq!(
        strs(&file_rels(&s, true, Some("ja"))),
        [".mdgrid/templates/fuda.md", ".mdgrid/workspace.toml"],
        "ledger.ja/ は lang = ja の印と雛形"
    );
    assert_eq!(
        strs(&file_rels(&s, false, Some("ja"))),
        ["board/最初の札.md"],
        "samples.ja/ は lang = ja の見本"
    );
    assert_eq!(
        strs(&file_rels(&s, true, None)),
        [".mdgrid/templates/card.md", ".mdgrid/workspace.toml"]
    );
    // ja で作る → ledger.ja/ と samples.ja/ の中身だけ。
    let root_ja = t.0.join("ja");
    let made = create(&root_ja, &s, Lang::Ja, true, now()).unwrap_or_else(|e| panic!("{e}"));
    let want_ja = [
        ".mdgrid/templates/fuda.md",
        ".mdgrid/workspace.toml",
        "board/最初の札.md",
    ];
    assert_eq!(strs(&rels(&root_ja, &made)), want_ja);
    assert_eq!(
        strs(&files_under(&root_ja)),
        want_ja,
        "ledger/ の雛形も見本も重ねない"
    );
    assert_eq!(read(&root_ja, ".mdgrid/workspace.toml"), MARKER_JA);
    assert_eq!(read(&root_ja, ".mdgrid/templates/fuda.md"), TEMPLATE_JA);
    // en で作る → ledger/ と samples/ の中身だけ。
    let root_en = t.0.join("en");
    let made = create(&root_en, &s, Lang::En, true, now()).unwrap_or_else(|e| panic!("{e}"));
    let want_en = [
        ".mdgrid/templates/card.md",
        ".mdgrid/workspace.toml",
        "board/First card.md",
    ];
    assert_eq!(strs(&rels(&root_en, &made)), want_en);
    assert_eq!(strs(&files_under(&root_en)), want_en);
    assert_eq!(read(&root_en, ".mdgrid/workspace.toml"), MARKER_EN);
}

#[test]
fn test_st_1_no_variant_for_language_uses_plain_ledger() {
    // [ST-1] ledger.ja/ が無ければ、ja で作っても ledger/・samples/ の中身(英語が正本)。
    let t = Tmp::new("novariant");
    let dir = user_starter(&t.0.join("config"), "kanban", Some(MANIFEST), false);
    let s = read_user(&dir).unwrap_or_else(|e| panic!("読めない: {e}"));
    let root = t.0.join("ja");
    let made = create(&root, &s, Lang::Ja, true, now()).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        strs(&rels(&root, &made)),
        [
            ".mdgrid/templates/card.md",
            ".mdgrid/workspace.toml",
            "board/First card.md"
        ]
    );
    assert_eq!(read(&root, ".mdgrid/workspace.toml"), MARKER_EN);
}

// ---- ST-2: 組み込み ----

#[test]
fn test_st_2_builtin_four_in_name_order_with_every_language() {
    // [ST-2] 組み込みは bookmarks・people・reading・tasks の4つで名前の順。どれも Origin::Builtin で、名前と説明を
    // 文言の台帳の全言語に持ち、印 .mdgrid/workspace.toml と雛形(.mdgrid/templates/)と見本を持つ。
    let all = builtin();
    assert_eq!(st_names(&all), ["bookmarks", "people", "reading", "tasks"]);
    for s in &all {
        assert!(matches!(s.origin, Origin::Builtin), "{}: 組み込み", s.name);
        for lang in Lang::ALL {
            let code = lang.code();
            assert!(
                s.title.contains_key(code),
                "{}: [name] に {code} が無い",
                s.name
            );
            let (name, desc) = describe(s, *lang);
            assert!(!name.is_empty(), "{}: {code} の名前が空", s.name);
            assert!(!desc.is_empty(), "{}: {code} の説明が空", s.name);
        }
        let ledger = file_rels(s, true, None);
        assert!(
            ledger.iter().any(|r| r == ".mdgrid/workspace.toml"),
            "{}: 印が無い: {ledger:?}",
            s.name
        );
        assert!(
            ledger
                .iter()
                .any(|r| r.starts_with(".mdgrid/templates/") && r.ends_with(".md")),
            "{}: 雛形が無い: {ledger:?}",
            s.name
        );
        assert!(
            s.files.iter().any(|f| matches!(f.part, Part::Sample)),
            "{}: 見本が無い",
            s.name
        );
    }
}

#[test]
fn test_st_2_builtin_every_language_creates_a_ledger_that_reads_without_warnings() {
    // [ST-2] 組み込みの全部 × 全言語を一時フォルダに作る(見本あり・なし)と、印の読み込み(read_marker)・
    // open(marker_open)・木の設定(marker_tree)の警告が空で、open が印の表(根の中)を指す。
    // 返したパスは書いたファイルと一致し、見本ありなら .mdgrid/ の外にノートがある。
    let t = Tmp::new("builtin");
    for s in &builtin() {
        for lang in Lang::ALL {
            let _g = scoped(*lang);
            for with_samples in [true, false] {
                let tag = format!("{}-{}-{with_samples}", s.name, lang.code());
                let root = t.0.join(&tag);
                let made = create(&root, s, *lang, with_samples, now())
                    .unwrap_or_else(|e| panic!("{tag}: 作れない: {e}"));
                let made = rels(&root, &made);
                assert!(
                    made.iter().any(|r| r == ".mdgrid/workspace.toml"),
                    "{tag}: 印を作る: {made:?}"
                );
                assert_eq!(
                    made,
                    files_under(&root),
                    "{tag}: 返したパスと書いたファイル"
                );
                assert_eq!(
                    made.iter()
                        .any(|r| r.ends_with(".md") && !r.starts_with(".mdgrid/")),
                    with_samples,
                    "{tag}: 見本は with_samples のときだけ: {made:?}"
                );
                let (ws, warns) = crate::workspace::read_marker(&root);
                assert!(warns.is_empty(), "{tag}: read_marker の警告: {warns:?}");
                assert!(!ws.tables.is_empty(), "{tag}: 印の表が無い");
                let (open, warns) = crate::workspace::marker_open(&root);
                assert!(warns.is_empty(), "{tag}: marker_open の警告: {warns:?}");
                let open = open.unwrap_or_else(|| panic!("{tag}: open が表を指さない"));
                assert!(
                    ws.tables.iter().any(|tb| tb.name == open.name),
                    "{tag}: open の表 {} が印の表に無い",
                    open.name
                );
                assert!(
                    open.path.starts_with(&root),
                    "{tag}: open の表は根の中: {}",
                    open.path.display()
                );
                let (_, warns) = crate::workspace::marker_tree(&open.path);
                assert!(warns.is_empty(), "{tag}: marker_tree の警告: {warns:?}");
            }
        }
    }
}

#[test]
fn test_st_2_builtin_marker_has_no_name_so_the_folder_names_the_workspace() {
    // [ST-2] 組み込みの印に name は書かない(仕様の前提): 作った先のフォルダの名前がワークスペースの名前になる(WS-7)。
    let t = Tmp::new("noname");
    for s in &builtin() {
        let root = t.0.join(format!("my-{}", s.name));
        create(&root, s, Lang::En, false, now()).unwrap_or_else(|e| panic!("{}: {e}", s.name));
        let (ws, _) = crate::workspace::read_marker(&root);
        assert_eq!(ws.name, format!("my-{}", s.name), "{}", s.name);
    }
}

// ---- ST-3: ユーザーのスターターと一覧 ----

#[test]
fn test_st_3_list_user_first_in_name_order_then_builtin_and_same_name_hides_builtin() {
    // [ST-3] 一覧はユーザーのものを名前の順に先に、組み込みをその後に。同じ名前(tasks)はユーザーのものだけ。
    let t = Tmp::new("list");
    let cfg = t.0.join("config");
    user_starter(&cfg, "zeta", Some(MANIFEST), false);
    user_starter(&cfg, "alpha", None, false);
    user_starter(
        &cfg,
        "tasks",
        Some("[name]\nen = \"My tasks\"\nja = \"自分のタスク\"\n\n[description]\nen = \"Mine\"\nja = \"自分の台帳\"\n"),
        false,
    );
    let (got, warns) = list(Some(&cfg));
    assert!(warns.is_empty(), "{warns:?}");
    assert_eq!(
        st_names(&got),
        ["alpha", "tasks", "zeta", "bookmarks", "people", "reading"]
    );
    let tasks: Vec<&Starter> = got.iter().filter(|s| s.name == "tasks").collect();
    assert_eq!(tasks.len(), 1, "tasks は1つ");
    assert!(matches!(tasks[0].origin, Origin::User(_)), "ユーザーのもの");
    assert_eq!(
        describe(tasks[0], Lang::Ja),
        ("自分のタスク".to_string(), "自分の台帳".to_string())
    );
    assert!(
        got.iter().filter(|s| s.name != "tasks").all(|s| {
            s.name == "alpha" || s.name == "zeta" || matches!(s.origin, Origin::Builtin)
        }),
        "ほかの組み込みは組み込みのまま"
    );
}

#[test]
fn test_st_3_list_without_config_dir_is_builtin_only() {
    // [ST-3] 設定の置き場が無い(None・無いフォルダ・starters/ の無いフォルダ)なら組み込みだけで、警告なし。
    let t = Tmp::new("nodir");
    let builtin_names: Vec<String> = builtin().iter().map(|s| s.name.clone()).collect();
    for (what, dir) in [
        ("None", None),
        ("無いフォルダ", Some(t.0.join("nowhere"))),
        ("starters/ の無いフォルダ", Some(t.0.clone())),
    ] {
        let (got, warns) = list(dir.as_deref());
        assert!(warns.is_empty(), "{what}: {warns:?}");
        assert_eq!(
            got.iter().map(|s| s.name.clone()).collect::<Vec<_>>(),
            builtin_names,
            "{what}"
        );
        assert!(
            got.iter().all(|s| matches!(s.origin, Origin::Builtin)),
            "{what}"
        );
    }
}

#[test]
fn test_st_3_broken_manifest_warns_in_cli_3_shape_and_is_skipped() {
    // [ST-3] 壊れた starter.toml は `starters/kanban/starter.toml:` で始まる警告1つで飛ばし、ほかは出る。
    // read_user も同じ形の Err。
    let t = Tmp::new("broken");
    let cfg = t.0.join("config");
    let kanban = user_starter(&cfg, "kanban", Some("[name\nen = \"Kanban\"\n"), false);
    user_starter(&cfg, "okay", Some(MANIFEST), false);
    let e = must_fail(read_user(&kanban), "壊れた starter.toml は Err");
    assert!(
        e.starts_with("starters/kanban/starter.toml:"),
        "Err は CLI-3 の形: {e}"
    );
    let (got, warns) = list(Some(&cfg));
    assert_eq!(warns.len(), 1, "{warns:?}");
    assert!(
        warns[0].starts_with("starters/kanban/starter.toml:"),
        "警告は CLI-3 の形: {}",
        warns[0]
    );
    assert_eq!(
        st_names(&got),
        ["okay", "bookmarks", "people", "reading", "tasks"],
        "壊れたものは出ず、ほかは出る"
    );
}

#[test]
fn test_st_3_starter_without_marker_warns_and_is_skipped() {
    // [ST-3] ledger/.mdgrid/workspace.toml の無いスターターは読めないので、警告して飛ばす(名前が警告に出る)。
    let t = Tmp::new("nomarker");
    let cfg = t.0.join("config");
    let dir = cfg.join("starters/nomarker");
    put(&dir.join("starter.toml"), MANIFEST);
    put(&dir.join("samples/board/First card.md"), SAMPLE_EN);
    put(&dir.join("ledger/.mdgrid/templates/card.md"), TEMPLATE_EN);
    assert!(read_user(&dir).is_err(), "印が無ければ Err");
    let (got, warns) = list(Some(&cfg));
    assert_eq!(warns.len(), 1, "{warns:?}");
    assert!(warns[0].contains("nomarker"), "{}", warns[0]);
    assert!(
        !st_names(&got).contains(&"nomarker"),
        "{:?}",
        st_names(&got)
    );
    assert_eq!(got.len(), builtin().len());
}

#[test]
fn test_st_3_find_by_name() {
    // [ST-3] find は名前で引く。無い名前は None。同じ名前はユーザーのもの。
    let t = Tmp::new("find");
    let cfg = t.0.join("config");
    user_starter(&cfg, "tasks", Some(MANIFEST), false);
    let (got, _) = list(Some(&cfg));
    assert_eq!(
        find(&got, "reading").map(|s| s.name.as_str()),
        Some("reading")
    );
    assert!(find(&got, "nope").is_none());
    assert!(find(&got, "").is_none());
    let tasks = find(&got, "tasks").expect("tasks");
    assert!(matches!(tasks.origin, Origin::User(_)));
}

// ---- ST-4: 作る ----

#[test]
fn test_st_4_create_copies_ledger_as_is_without_samples() {
    // [ST-4] with_samples が偽なら ledger/ の中身(印と雛形)だけを同じ相対パスで書き、雛形の {date}・{name} は文字のまま。
    let t = Tmp::new("create");
    let dir = user_starter(&t.0.join("config"), "kanban", Some(MANIFEST), false);
    let s = read_user(&dir).unwrap_or_else(|e| panic!("読めない: {e}"));
    let root = t.0.join("board");
    let made = create(&root, &s, Lang::En, false, now()).unwrap_or_else(|e| panic!("{e}"));
    let want = [".mdgrid/templates/card.md", ".mdgrid/workspace.toml"];
    assert_eq!(strs(&rels(&root, &made)), want);
    assert_eq!(strs(&files_under(&root)), want, "見本は書かない");
    assert_eq!(
        read(&root, ".mdgrid/workspace.toml"),
        MARKER_EN,
        "印はそのまま"
    );
    assert_eq!(
        read(&root, ".mdgrid/templates/card.md"),
        TEMPLATE_EN,
        "雛形の変数は文字のまま"
    );
}

#[test]
fn test_st_4_create_with_samples_fills_sample_vars_only() {
    // [ST-4] with_samples が真なら見本も書き、見本の {date}・{date+3}・{date-10}・{time} は CE-32 の変数として埋める。
    // 雛形は同じ呼び出しでも文字のまま。
    let t = Tmp::new("samples");
    let dir = user_starter(&t.0.join("config"), "kanban", Some(MANIFEST), false);
    let s = read_user(&dir).unwrap_or_else(|e| panic!("読めない: {e}"));
    let root = t.0.join("board");
    let made = create(&root, &s, Lang::En, true, now()).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        strs(&rels(&root, &made)),
        [
            ".mdgrid/templates/card.md",
            ".mdgrid/workspace.toml",
            "board/First card.md"
        ]
    );
    assert_eq!(
        read(&root, "board/First card.md"),
        "---\ntitle: First card\ndue: \"2026-10-09\"\n---\nWritten 2026-10-06 at 09:05, ten days after 2026-09-26\n"
    );
    assert_eq!(read(&root, ".mdgrid/templates/card.md"), TEMPLATE_EN);
    assert_eq!(read(&root, ".mdgrid/workspace.toml"), MARKER_EN);
}

#[test]
fn test_st_4_create_refuses_when_any_target_exists_and_writes_nothing() {
    // [ST-4] 書く先に同じパスのファイルが1つでもあれば Err(そのファイルの名前が入る)で、どれも書かず、あるものも変えない。
    let t = Tmp::new("exists");
    let dir = user_starter(&t.0.join("config"), "kanban", Some(MANIFEST), false);
    let s = read_user(&dir).unwrap_or_else(|e| panic!("読めない: {e}"));
    // 見本の1つだけが先にある。
    let root = t.0.join("one");
    put(&root.join("board/First card.md"), "keep\n");
    let e = must_fail(create(&root, &s, Lang::En, true, now()), "見本があれば Err");
    assert!(e.contains("First card.md"), "理由にそのファイル: {e}");
    assert_eq!(
        strs(&files_under(&root)),
        ["board/First card.md"],
        "何も書かない"
    );
    assert_eq!(
        read(&root, "board/First card.md"),
        "keep\n",
        "あるものは変えない"
    );
    // 印だけが先にある(WS-7 と同じく上書きしない)。
    let root = t.0.join("marker");
    put(&root.join(".mdgrid/workspace.toml"), "name = \"mine\"\n");
    let e = must_fail(create(&root, &s, Lang::En, false, now()), "印があれば Err");
    assert!(e.contains("workspace.toml"), "理由にそのファイル: {e}");
    assert_eq!(strs(&files_under(&root)), [".mdgrid/workspace.toml"]);
    assert_eq!(read(&root, ".mdgrid/workspace.toml"), "name = \"mine\"\n");
    // 2回目(全部ある)も Err で変わらない。
    let root = t.0.join("twice");
    create(&root, &s, Lang::En, true, now()).unwrap_or_else(|e| panic!("{e}"));
    let before = files_under(&root);
    assert!(create(&root, &s, Lang::En, true, now()).is_err());
    assert!(create(&root, &s, Lang::En, false, now()).is_err());
    assert_eq!(files_under(&root), before);
    assert_eq!(read(&root, ".mdgrid/workspace.toml"), MARKER_EN);
}

#[test]
fn test_st_4_create_without_samples_ignores_existing_sample_path() {
    // [ST-4] 書く先だけを見る: with_samples が偽なら、見本のパスにファイルがあっても印と雛形は書ける。
    let t = Tmp::new("skip-sample");
    let dir = user_starter(&t.0.join("config"), "kanban", Some(MANIFEST), false);
    let s = read_user(&dir).unwrap_or_else(|e| panic!("読めない: {e}"));
    let root = t.0.join("board");
    put(&root.join("board/First card.md"), "keep\n");
    let made = create(&root, &s, Lang::En, false, now()).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        strs(&rels(&root, &made)),
        [".mdgrid/templates/card.md", ".mdgrid/workspace.toml"]
    );
    assert_eq!(read(&root, "board/First card.md"), "keep\n");
}

#[test]
fn test_st_4_create_makes_a_missing_root() {
    // [ST-4] 根のフォルダが無ければ(深くても)作る。
    let t = Tmp::new("mkroot");
    let dir = user_starter(&t.0.join("config"), "kanban", Some(MANIFEST), false);
    let s = read_user(&dir).unwrap_or_else(|e| panic!("読めない: {e}"));
    let root = t.0.join("new/deep/books");
    assert!(!root.exists());
    let made = create(&root, &s, Lang::En, true, now()).unwrap_or_else(|e| panic!("{e}"));
    assert!(root.is_dir());
    assert_eq!(made.len(), 3);
    assert!(root.join(".mdgrid/workspace.toml").is_file());
    assert!(root.join("board/First card.md").is_file());
}
