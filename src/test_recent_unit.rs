//! 最近開いた表(cli.recent-*)の覚え方と読み方。期待値は specs/requirements/cli.md の受け入れの例から取った
//! (画面で開いて覚えるのは、本物の実行ファイルの試験 tests/test_recent_e2e.rs で見る)。

use super::*;

fn tmp(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let d = std::env::temp_dir().join(format!("mdgrid-rec-{name}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    std::fs::canonicalize(&d).unwrap()
}

fn folder(d: &Path, name: &str) -> PathBuf {
    let p = d.join(name);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn names(list: &[Recent]) -> Vec<String> {
    list.iter().map(|r| workspace::stem_of(&r.path)).collect()
}

#[test]
fn test_recent_newest_first_without_repeats() {
    // [cli.recent-record] tasks → projects → tasks の順に開く → tasks・projects の順で、tasks は1件だけ。ビューも覚える。
    let d = tmp("order");
    let state = d.join("state");
    let (tasks, projects) = (folder(&d, "tasks"), folder(&d, "projects"));
    record(&state, &tasks, None, "2026-11-21 09:00").unwrap();
    record(&state, &projects, None, "2026-11-21 09:10").unwrap();
    record(&state, &tasks, Some("進行中"), "2026-11-21 09:30").unwrap();
    let list = load(&state);
    assert_eq!(names(&list), vec!["tasks", "projects"]);
    assert_eq!(list[0].view.as_deref(), Some("進行中"));
    assert_eq!(list[0].at, "2026-11-21 09:30");
}

#[test]
fn test_recent_keeps_twenty() {
    // [cli.recent-record] 21 個目の表を開く → いちばん古い1件が消える。
    let d = tmp("limit");
    let state = d.join("state");
    for k in 1..=21 {
        let p = folder(&d, &format!("t{k:02}"));
        record(&state, &p, None, "2026-11-21 09:00").unwrap();
    }
    let list = load(&state);
    assert_eq!(list.len(), LIMIT);
    assert_eq!(names(&list).first().map(String::as_str), Some("t21"));
    assert!(!names(&list).contains(&"t01".to_string()));
}

#[test]
fn test_recent_missing_path_is_dropped() {
    // [utest -> req~cli.recent-missing~1] 覚えた projects のフォルダを消してから読む → projects が無く、覚えた並びからも外れる。
    let d = tmp("missing");
    let state = d.join("state");
    let (tasks, projects) = (folder(&d, "tasks"), folder(&d, "projects"));
    record(&state, &tasks, None, "2026-11-21 09:00").unwrap();
    record(&state, &projects, None, "2026-11-21 09:10").unwrap();
    std::fs::remove_dir_all(&projects).unwrap();
    assert_eq!(names(&present(&state)), vec!["tasks"]);
    assert_eq!(names(&load(&state)), vec!["tasks"], "覚えた並びから外す");
}

#[test]
fn test_recent_line_form() {
    // [utest -> req~cli.recent-cli~2] 「パス(タブ)ビューの名前(タブ)最後に開いた日時」の1行。ビューが無ければ空の欄。
    let r = Recent {
        path: PathBuf::from("/home/u/notes/tasks.base"),
        view: Some("進行中".into()),
        at: "2026-11-21 09:30".into(),
    };
    assert_eq!(
        r.line(),
        "/home/u/notes/tasks.base\t進行中\t2026-11-21 09:30"
    );
    let r = Recent { view: None, ..r };
    assert_eq!(r.line(), "/home/u/notes/tasks.base\t\t2026-11-21 09:30");
}

#[test]
fn test_recent_stamp_is_local_minutes() {
    // 日時は地域の時刻の分まで(2026-11-21 00:30 UTC は +09:00 で 09:30)。
    let utc = 20_778 * 86_400 + 30 * 60;
    assert_eq!(stamp(utc, 9 * 3600), "2026-11-21 09:30");
    assert_eq!(stamp(utc, 0), "2026-11-21 00:30");
}

#[test]
fn test_recent_unreadable_file_is_empty() {
    // 壊れたファイルは無いのと同じ(見た目の状態と同じく、消えても困らない)。
    let d = tmp("broken");
    std::fs::write(d.join(FILE_NAME), "[[table]\n").unwrap();
    assert!(load(&d).is_empty());
}
