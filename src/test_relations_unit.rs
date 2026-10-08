//! リレーションの核(REL-1・REL-2・REL-3・REL-5・REL-6・REL-10)。

use super::*;

/// 一時フォルダ(落ちても消える)。
struct Tmp(PathBuf);

impl Tmp {
    fn new(name: &str) -> Tmp {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let d =
            std::env::temp_dir().join(format!("mdgrid-rel-{name}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        Tmp(std::fs::canonicalize(&d).unwrap())
    }

    fn write(&self, rel: &str, text: &str) -> PathBuf {
        let p = self.0.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, text).unwrap();
        p
    }
}

impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// projects・tasks・members の小さなワークスペース。
fn workspace(name: &str) -> Tmp {
    let t = Tmp::new(name);
    t.write("projects/mdgrid.md", "---\nstatus: active\n---\n");
    t.write("projects/Other Project.md", "---\nstatus: idea\n---\n");
    t.write("members/Alice.md", "---\nrole: dev\n---\n");
    t.write("members/Bob.md", "---\nrole: pm\n---\n");
    t.write(
        "tasks/Build relation view.md",
        "---\nproject: \"[[mdgrid]]\"\nassignee: \"[[Alice]]\"\ntags:\n  - \"[[tui]]\"\n---\n",
    );
    t.write(
        "tasks/Add backlinks.md",
        "---\nproject: \"[mdgrid](../projects/mdgrid.md)\"\nassignee: ../members/Bob\n---\n",
    );
    t.write(
        "tasks/Write docs.md",
        "---\nproject: \"[[nothing]]\"\nstatus: done\n---\n",
    );
    t.write("tui.md", "---\n---\n");
    t
}

fn link(s: &str) -> Link {
    parse(s).unwrap_or_else(|| panic!("リンクでない: {s}"))
}

#[test]
fn test_rel_1_forms_and_resolution() {
    // [REL-1] 3つの書き方を読む。行き先は相対 → 根からのパス → 名前の順。
    let t = workspace("forms");
    let root = Notes::scan(&t.0);
    let note = t.0.join("tasks/Build relation view.md");
    let mdgrid = t.0.join("projects/mdgrid.md");
    assert_eq!(link("[[mdgrid]]").form, Form::Wiki);
    assert_eq!(
        resolve(&link("[[mdgrid]]"), &note, &[&root]),
        Some(mdgrid.clone())
    );
    assert_eq!(
        resolve(&link("[[projects/mdgrid|MD]]"), &note, &[&root]),
        Some(mdgrid.clone())
    );
    assert_eq!(link("[[projects/mdgrid|MD]]").alias.as_deref(), Some("MD"));
    assert_eq!(
        resolve(&link("[[mdgrid#見出し]]"), &note, &[&root]),
        Some(mdgrid.clone())
    );
    let md = link("[x](../projects/mdgrid.md)");
    assert_eq!(md.form, Form::Md);
    assert_eq!(resolve(&md, &note, &[&root]), Some(mdgrid.clone()));
    assert_eq!(
        resolve(
            &link("[x](../projects/Other%20Project.md)"),
            &note,
            &[&root]
        ),
        Some(t.0.join("projects/Other Project.md"))
    );
    // ただのパス: 相対(.md は省いてよい)と、根からのパス。
    assert_eq!(
        resolve(&link("../projects/mdgrid"), &note, &[&root]),
        Some(mdgrid.clone())
    );
    assert_eq!(
        resolve(&link("projects/mdgrid.md"), &note, &[&root]),
        Some(mdgrid.clone())
    );
    assert_eq!(link("projects/mdgrid.md").form, Form::Path { ext: true });
}

#[test]
fn test_rel_1_plain_words_are_not_links() {
    // [REL-1] どのノートも指さない文字・1語・外のアドレスはリンクにしない。
    let t = workspace("words");
    t.write("done.md", "---\n---\n");
    let root = Notes::scan(&t.0);
    let note = t.0.join("tasks/Write docs.md");
    assert!(
        parse("done").is_none(),
        "1語はパスとみなさない(done.md があっても)"
    );
    assert!(parse("https://example.com/a.md").is_none());
    assert!(parse("[x](https://example.com)").is_none());
    assert!(parse("").is_none());
    assert!(parse("a\nb/c").is_none());
    // パスの形でも、在るノートを指さなければ解けない。
    assert_eq!(resolve(&link("nowhere/x"), &note, &[&root]), None);
}

#[test]
fn test_rel_1_target_outside_root() {
    // [REL-1] 行き先は開いた表のフォルダの外(ほかの表)にあってもよい。
    let t = workspace("outside");
    let tasks = Notes::scan(&t.0.join("tasks"));
    let projects = Notes::scan(&t.0.join("projects"));
    let note = t.0.join("tasks/Build relation view.md");
    // 相対のパスはフォルダの外へ解ける。
    assert_eq!(
        resolve(&link("../projects/mdgrid"), &note, &[&tasks]),
        Some(t.0.join("projects/mdgrid.md"))
    );
    // 名前は、渡した表(ほかの表を含む)で探す。
    assert_eq!(resolve(&link("[[mdgrid]]"), &note, &[&tasks]), None);
    assert_eq!(
        resolve(&link("[[mdgrid]]"), &note, &[&tasks, &projects]),
        Some(t.0.join("projects/mdgrid.md"))
    );
}

#[test]
fn test_rel_2_display_names() {
    // [REL-2][REL-10] 見せる名前は表示の文字か行き先の名前。行き先が無ければ書いた名前。
    let t = workspace("display");
    let mdgrid = t.0.join("projects/mdgrid.md");
    assert_eq!(
        display(&link("[[projects/mdgrid|MD]]"), Some(&mdgrid)),
        "MD"
    );
    assert_eq!(
        display(&link("../projects/mdgrid"), Some(&mdgrid)),
        "mdgrid"
    );
    assert_eq!(display(&link("[[nothing]]"), None), "nothing");
    // 行き先が無ければ書いた行き先のまま。Markdown のリンクは表示の文字でなく行き先の名前。
    assert_eq!(display(&link("[[a/nothing]]"), None), "a/nothing");
    assert_eq!(display(&link("a/b/nothing.md"), None), "a/b/nothing");
    assert_eq!(
        display(&link("[MD Grid](../projects/mdgrid.md)"), Some(&mdgrid)),
        "mdgrid"
    );
}

#[test]
fn test_rel_3_format_matches_form() {
    // [REL-3] 書く形は列の形に合わせる。名前が探す表の全部で1つなら [[名前]]、重なれば相対のパス。
    let t = workspace("format");
    let note = t.0.join("tasks/Add backlinks.md");
    let mdgrid = t.0.join("projects/mdgrid.md");
    let other = t.0.join("projects/Other Project.md");
    assert_eq!(format(Form::Wiki, &mdgrid, &note, true), "[[mdgrid]]");
    assert_eq!(
        format(Form::Md, &other, &note, true),
        "[Other Project](../projects/Other%20Project.md)"
    );
    assert_eq!(
        format(Form::Path { ext: false }, &mdgrid, &note, true),
        "../projects/mdgrid"
    );
    assert_eq!(
        format(Form::Path { ext: true }, &mdgrid, &note, true),
        "../projects/mdgrid.md"
    );
    // 別の表に同じ名前があれば、[[…]] はノートのフォルダからの相対のパス(取り違えない)。
    t.write("tasks/archive/mdgrid.md", "---\n---\n");
    let tasks = Notes::scan(&t.0.join("tasks"));
    let projects = Notes::scan(&t.0.join("projects"));
    assert_eq!(count_name("mdgrid", &[&tasks, &projects]), 2);
    assert_eq!(
        format(Form::Wiki, &mdgrid, &note, false),
        "[[../projects/mdgrid]]"
    );
    // 書いた値は同じ行き先に解ける(今の表を先に見ても)。
    for f in [Form::Wiki, Form::Md, Form::Path { ext: false }] {
        let v = format(f, &mdgrid, &note, false);
        assert_eq!(
            resolve(&link(&v), &note, &[&tasks, &projects]),
            Some(mdgrid.clone()),
            "{v}"
        );
    }
}

#[test]
fn test_rel_3_link_column() {
    // [REL-3] 過半数がリンクの列はリンクの列。形と行き先のフォルダはいちばん多いもの。
    let t = workspace("column");
    let root = Notes::scan(&t.0);
    let n = |r: &str| t.0.join(r);
    let s = |v: &str| Value::Str(v.into());
    let vals = vec![
        (n("tasks/Build relation view.md"), s("[[mdgrid]]")),
        (n("tasks/Add backlinks.md"), s("[[Other Project]]")),
        (n("tasks/Write docs.md"), s("[[nothing]]")),
    ];
    let (form, dir) = link_column(&vals, &[&root]).unwrap();
    assert_eq!(form, Form::Wiki);
    assert_eq!(dir, Some(t.0.join("projects")));
    let words = vec![
        (n("tasks/Build relation view.md"), s("todo")),
        (n("tasks/Add backlinks.md"), s("done")),
        (n("tasks/Write docs.md"), s("[[mdgrid]]")),
    ];
    assert!(link_column(&words, &[&root]).is_none(), "過半数でない");
    assert_eq!(
        notes_in(&t.0.join("projects")),
        vec![
            t.0.join("projects/mdgrid.md"),
            t.0.join("projects/Other Project.md")
        ]
    );
}

fn tables(t: &Tmp) -> Vec<Table> {
    vec![
        Table::new("Tasks", &t.0.join("tasks")),
        Table::new("Projects", &t.0.join("projects")),
        Table::new("Members", &t.0.join("members")),
    ]
}

#[test]
fn test_rel_5_backlinks() {
    // [REL-5] つながった行: どの表のどの列から指しているか。3つの書き方のどれでも。
    let t = workspace("back");
    let mut got = backlinks(&t.0.join("projects/mdgrid.md"), &tables(&t));
    got.sort_by(|a, b| a.note.cmp(&b.note));
    let want: Vec<(String, Option<String>, String)> = got
        .iter()
        .map(|b| (file_stem(&b.note), b.table.clone(), b.column.clone()))
        .collect();
    assert_eq!(
        want,
        vec![
            (
                "Add backlinks".into(),
                Some("Tasks".into()),
                "project".into()
            ),
            (
                "Build relation view".into(),
                Some("Tasks".into()),
                "project".into()
            ),
        ]
    );
    let bob = backlinks(&t.0.join("members/Bob.md"), &tables(&t));
    assert_eq!(bob.len(), 1);
    assert_eq!(bob[0].column, "assignee");
}

#[test]
fn test_rel_6_edges() {
    // [REL-6] 表どうしのつながりを設定なしで見つける。リストは多対多。どの表にも無い行き先は数えない。
    let t = workspace("edges");
    let mut e = edges(&tables(&t));
    e.sort_by(|a, b| (&a.from, &a.column).cmp(&(&b.from, &b.column)));
    assert_eq!(
        e,
        vec![
            Edge {
                from: "Tasks".into(),
                column: "assignee".into(),
                to: "Members".into(),
                many: false
            },
            Edge {
                from: "Tasks".into(),
                column: "project".into(),
                to: "Projects".into(),
                many: false
            },
        ],
        "tags の [[tui]] はどの表にも無いので数えない"
    );
    // リストの列は多対多。
    t.write(
        "projects/Website.md",
        "---\nmembers:\n  - \"[[Alice]]\"\n  - \"[[Bob]]\"\n---\n",
    );
    assert!(edges(&tables(&t)).contains(&Edge {
        from: "Projects".into(),
        column: "members".into(),
        to: "Members".into(),
        many: true
    }));
    std::fs::remove_file(t.0.join("projects/Website.md")).unwrap();
    // 表の形にそろわないノート(フロントマターが無い・壊れている)があっても止まらない。
    t.write("tasks/broken.md", "---\nproject: [unclosed\n");
    t.write("tasks/plain.md", "ただの本文\n");
    assert_eq!(edges(&tables(&t)).len(), 2);
}

#[test]
fn test_rel_1_percent_with_multibyte_does_not_panic() {
    // [REL-1] `%` のあとが多バイトの文字・途中で切れた `%` でも落ちない(壊れた値も読める)。
    for v in [
        "[x](a%あb.md)",
        "[x](%E3%81)",
        "[x](a%)",
        "[x](%%%)",
        "[x](%zz/b.md)",
    ] {
        let l = parse(v).unwrap();
        assert_eq!(l.form, Form::Md);
    }
    assert_eq!(parse("[x](a%20b.md)").unwrap().target, "a b.md");
}

#[test]
fn test_rel_1_registered_base_uses_vault_root() {
    // [REL-1] 登録した `.base` の表は、開いたときと同じく保管庫の根(.obsidian のある上のフォルダ)を探す。
    let t = workspace("base");
    std::fs::create_dir_all(t.0.join(".obsidian")).unwrap();
    t.write("Bases/tasks.base", "views: []\n");
    let tb = Table::new("Tasks", &t.0.join("Bases/tasks.base"));
    assert_eq!(tb.dir, t.0);
    let plain = Tmp::new("base-plain");
    plain.write("b/x.base", "views: []\n");
    assert_eq!(
        Table::new("X", &plain.0.join("b/x.base")).dir,
        plain.0.join("b")
    );
}
