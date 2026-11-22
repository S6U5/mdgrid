//! セッションのファイル(sessions.toml)の読み書きの試験(cli.session-*)。期待値は要求の受け入れの例から取った。

use super::*;
use crate::i18n::{self, Lang};

fn sample() -> Session {
    Session {
        name: "進行中".into(),
        folders: vec![PathBuf::from("/data/notes/tasks")],
        base: None,
        view: None,
        columns: vec!["title".into(), "status".into(), "due".into()],
        hidden: vec!["priority".into()],
        sort: vec![("due".into(), Dir::Asc)],
        filters: vec![
            Filter::Value {
                key: "status".into(),
                value: "進行中".into(),
                by_keys: false,
                list: false,
            },
            Filter::Text("会議".into()),
            Filter::Expr("due < today()".into()),
            Filter::Cond("soon".into()),
            Filter::Show("archived".into()),
        ],
        readonly: true,
    }
}

#[test]
fn test_session_round_trip_keeps_every_field() {
    // [utest -> req~cli.session-content-fields~2] フォルダ・ビュー・絞り込み(検索・同じ値・式・数字のキーの条件・
    // 止めたいつも隠す条件)・並び・列の並びと隠す列・読むだけを持ち、書いて読み直すと同じ。値は持たない。
    let mut s = sample();
    s.base = Some(PathBuf::from("/data/notes/tasks.base"));
    s.view = Some("表".into());
    let text = to_toml(std::slice::from_ref(&s));
    let (got, warns) = parse(&text).unwrap();
    assert!(warns.is_empty(), "{warns:?}");
    assert_eq!(got, vec![s]);
    // ノートの値(会議のメモのタイトルなど)は文に無い。
    assert!(!text.contains("会議のメモ"));
}

#[test]
fn test_session_store_table_and_unknown_item_warns() {
    // [utest -> req~cli.session-store~1] `[sessions.進行中]` に置き、手で足した `color = "red"` は警告にとどめる。
    // [utest -> req~cli.config-file-warning-form~2] 警告は「ファイルの名前: 項目の道筋: 理由」の形。
    let _ja = i18n::scoped(Lang::Ja);
    let text = to_toml(&[sample()]);
    let t: toml::Table = text.parse().unwrap();
    assert!(t["sessions"].as_table().unwrap().contains_key("進行中"));
    let text = text.replacen("readonly = true\n", "readonly = true\ncolor = \"red\"\n", 1);
    let (got, warns) = parse(&text).unwrap();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].columns, ["title", "status", "due"]);
    assert_eq!(
        warns,
        ["sessions.toml: sessions.進行中.color: 知らない項目なので無視した"]
    );
}

#[test]
fn test_session_store_wrong_types_warn() {
    // [utest -> req~cli.session-store~1] 型の違う値・知らない絞り込みの種類も警告にして、ほかは読む。
    let _ja = i18n::scoped(Lang::Ja);
    let text = "[sessions.a]\nfolders = [\"/x\"]\nreadonly = \"yes\"\n\
                [[sessions.a.filters]]\nkind = \"regex\"\n\
                [[sessions.a.sort]]\nkey = \"due\"\ndir = \"up\"\n";
    let (got, mut warns) = parse(text).unwrap();
    assert_eq!(got[0].folders, [PathBuf::from("/x")]);
    assert!(!got[0].readonly);
    assert!(got[0].filters.is_empty() && got[0].sort.is_empty());
    warns.sort();
    assert_eq!(warns.len(), 3, "{warns:?}");
    assert!(warns[0].starts_with("sessions.toml: sessions.a.filters[1].kind:"));
    assert!(warns[1].starts_with("sessions.toml: sessions.a.readonly:"));
    assert!(warns[2].starts_with("sessions.toml: sessions.a.sort[1].dir:"));
}

#[test]
fn test_session_broken_file_is_an_error() {
    // [utest -> req~cli.session-errors~2] 壊れた sessions.toml は理由1行(行の番号つき)。
    let _ja = i18n::scoped(Lang::Ja);
    let e = parse("[sessions.a\nfolders = 1\n").unwrap_err();
    assert!(e.starts_with("sessions.toml が読めない(1 行目): "), "{e}");
    assert!(!e.contains('\n'));
}

#[test]
fn test_session_save_keeps_hand_written_text() {
    // [utest -> req~cli.session-screen-overwrite~1] 同じ名前は置き換え、ほかのセッションと手で書いたコメントは残す。
    let hand = "# 手で書いたコメント\n[sessions.other]\nfolders = [\"/o\"]  # 横のコメント\n\n\
                [sessions.\"進行中\"]\nfolders = [\"/old\"]\n\n[[sessions.\"進行中\".filters]]\nkind = \"text\"\nvalue = \"古い\"\n";
    let new = upsert(hand, &sample()).unwrap();
    assert!(new.contains("# 手で書いたコメント"), "{new}");
    assert!(new.contains("# 横のコメント"), "{new}");
    assert!(!new.contains("古い") && !new.contains("/old"), "{new}");
    let (got, _) = parse(&new).unwrap();
    assert_eq!(got.len(), 2);
    assert_eq!(got.iter().find(|s| s.name == "進行中"), Some(&sample()));
    assert_eq!(
        got.iter().find(|s| s.name == "other").unwrap().folders,
        [PathBuf::from("/o")]
    );
}

#[test]
fn test_session_save_adds_after_the_others() {
    // [utest -> req~cli.session-screen-save~1] 無い名前は後ろに足す(前のセッションの文はそのまま)。
    let first = to_toml(&[Session {
        name: "a".into(),
        folders: vec![PathBuf::from("/a")],
        ..Session::default()
    }]);
    let new = upsert(&first, &sample()).unwrap();
    assert!(new.starts_with(&first), "{new}");
    let (got, _) = parse(&new).unwrap();
    let names: Vec<&str> = got.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["a", "進行中"]);
}

#[test]
fn test_session_save_refuses_a_broken_file() {
    // [utest -> req~cli.session-store~1] 壊れたファイルは書き換えない(手で書いた中身を消さない)。
    assert!(upsert("[sessions.a\n", &sample()).is_err());
}

#[test]
fn test_session_list_text() {
    // [utest -> req~cli.session-screen-list~2] 1つ1行で「名前<TAB>開くフォルダ」。無ければ何も出さない。
    let s = sample();
    assert_eq!(list_text(&[s]), "進行中\t/data/notes/tasks\n");
    assert_eq!(list_text(&[]), "");
}

#[test]
fn test_session_home_is_written_with_tilde() {
    // [utest -> req~cli.session-screen-list~2] ホームの下のフォルダは `~/…` で書き、読むとホームに戻す。
    let Some(home) = crate::config::home_dir() else {
        return;
    };
    let s = Session {
        name: "h".into(),
        folders: vec![home.join("notes/tasks")],
        ..Session::default()
    };
    let text = to_toml(std::slice::from_ref(&s));
    assert!(text.contains("folders = [\"~/notes/tasks\"]"), "{text}");
    assert_eq!(list_text(std::slice::from_ref(&s)), "h\t~/notes/tasks\n");
    assert_eq!(parse(&text).unwrap().0, vec![s]);
}

#[test]
fn test_session_open_paths_prefer_the_base() {
    // [utest -> req~cli.session-content-fields~2] `.base` があればそれを開き、無ければフォルダの並び。
    let mut s = sample();
    assert_eq!(s.open_paths(), [PathBuf::from("/data/notes/tasks")]);
    s.base = Some(PathBuf::from("/data/notes/tasks.base"));
    assert_eq!(s.open_paths(), [PathBuf::from("/data/notes/tasks.base")]);
}

#[test]
fn test_session_name_check() {
    // [utest -> req~cli.session-screen-save~1] 空とタブ・改行の名前は保存しない(`--sessions` の1行を壊さない)。
    assert_eq!(check_name(" 進行中 ").unwrap(), "進行中");
    assert!(check_name("  ").is_err());
    assert!(check_name("a\tb").is_err());
}
