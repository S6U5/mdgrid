//! 新しいノートの窓の欄を、セルを直す窓と同じ候補・キー・入れ方にする(2026-11-25 の実地の操作で見つかった、
//! 窓の欄だけ別の候補・キー・入力を使っていた種類)。期待値は specs/requirements/cell-edit.md の
//! cell-edit.new-note-form-link-candidates・-same-input・-move・-create・-esc-popup・-error-clear・
//! cell-edit.template-vars-name-head の受け入れの例から。今日は 2026-10-02 に固定する。

use super::keymap::Mode;
use super::listpick::{Mark, PickRow};
use super::startup::Startup;
use super::test_screen::{ch, ctrl, press, screen, typing, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::source::markdown::Markdown;
use mdgrid::types;
use ratatui::crossterm::event::KeyCode;

fn boot(tmp: &Tmp, toml: &str) -> App {
    let (cfg, warnings): (Config, _) = mdgrid::config::parse(toml).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(100, 30);
    app.start(Startup {
        config: cfg,
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: tmp.notes(),
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app.today = types::parse_date("2026-10-02").unwrap();
    app
}

fn write(tmp: &Tmp, rel: &str, text: &str) {
    let p = tmp.notes().join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

fn read(tmp: &Tmp, rel: &str) -> String {
    std::fs::read_to_string(tmp.notes().join(rel)).unwrap_or_default()
}

/// 名前を打って Enter(最初の聞く欄へ)。
fn name_enter(a: &mut App, name: &str) {
    ch(a, 'a');
    assert!(a.note.flow.is_some(), "窓が開く:\n{}", screen(a));
    typing(a, name);
    press(a, KeyCode::Enter);
}

/// 今の欄の候補のリストの項目の文字(リストが無ければ空)。
fn list_texts(a: &App) -> Vec<String> {
    a.input
        .as_ref()
        .and_then(|i| i.list.as_ref())
        .map(|l| l.items.iter().map(|it| it.text.clone()).collect())
        .unwrap_or_default()
}

/// 今の欄の候補のリストで、文字が `text` の項目を ↑↓ で選ぶ(先頭まで ↑ で上がってから ↓ で探す)。
fn select_item(a: &mut App, text: &str) {
    let n = list_texts(a).len();
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
    panic!("候補に {text} が無い: {:?}", list_texts(a));
}

/// リストの値の付け外しの窓で、見せる名前が `name` の候補の行を選ぶ。
fn pick_select(a: &mut App, name: &str) {
    let p = a.pick.as_mut().expect("付け外しの窓が開いている");
    let rows = p.rows();
    let at = rows
        .iter()
        .position(|r| matches!(r, PickRow::Cand(i) if p.cands[*i].shown() == name))
        .unwrap_or_else(|| panic!("候補に {name} が無い"));
    p.sel = at;
}

fn pick_mark(a: &App, name: &str) -> Mark {
    let p = a.pick.as_ref().expect("付け外しの窓が開いている");
    p.cands
        .iter()
        .find(|c| c.shown() == name)
        .map_or_else(|| panic!("候補に {name} が無い"), |c| c.mark)
}

fn pick_names(a: &App) -> Vec<String> {
    let p = a.pick.as_ref().expect("付け外しの窓が開いている");
    p.cands.iter().map(|c| c.shown().to_string()).collect()
}

/// 窓の欄の行(縦の線 `│` で始まり、見出しが `label` の行)。
fn field_line(a: &App, label: &str) -> String {
    let s = screen(a);
    s.lines()
        .find(|l| {
            l.split('│').skip(1).any(|rest| {
                let rest = rest.strip_prefix(' ').unwrap_or(rest);
                let rest = rest.strip_prefix("> ").unwrap_or(rest).trim_start();
                rest.starts_with(label)
            })
        })
        .unwrap_or_else(|| panic!("欄 {label} の行が無い:\n{s}"))
        .to_string()
}

fn step(a: &App) -> usize {
    a.note.flow.as_ref().expect("窓が開いている").step
}

// ---- リンクの列の候補(cell-edit.new-note-form-link-candidates) ----

/// Library に本が2冊、Quotes の抜き書きの source はどれも霧の灯台守を指す(雨の日の数学を指すものは無い)。
fn reading(tag: &str) -> Tmp {
    let tmp = Tmp::new(tag);
    write(&tmp, "Library/霧の灯台守.md", "---\nkind: book\n---\n");
    write(&tmp, "Library/雨の日の数学.md", "---\nkind: book\n---\n");
    write(
        &tmp,
        "Quotes/q1.md",
        "---\nsource: \"[[霧の灯台守]]\"\npage: 3\n---\n",
    );
    write(
        &tmp,
        "Quotes/q2.md",
        "---\nsource: \"[[霧の灯台守]]\"\npage: 9\n---\n",
    );
    tmp
}

const READING: &str = "[new_note]\nfolder = \"Quotes\"\nask = [\"source\", \"page\"]\n";

#[test]
fn test_link_field_candidates_are_notes_of_the_target_folder() {
    // [utest -> req~cell-edit.new-note-form-link-candidates~1] source の欄の候補は、Library のノートの名前(まだどの抜き書きからも
    // 指されていない「雨の日の数学」も並ぶ)。`[[…]]` の生の形の候補は無い。
    let tmp = reading("nfs_link1");
    let mut a = boot(&tmp, READING);
    name_enter(&mut a, "新しい抜き書き");
    assert_eq!(step(&a), 1, "source の欄へ:\n{}", screen(&a));
    let got = list_texts(&a);
    assert!(got.contains(&"霧の灯台守".to_string()), "{got:?}");
    assert!(got.contains(&"雨の日の数学".to_string()), "{got:?}");
    assert!(!got.iter().any(|t| t.contains("[[")), "{got:?}");
}

#[test]
fn test_link_field_writes_link_and_shows_name() {
    // [utest -> req~cell-edit.new-note-form-link-candidates~1] 候補の「雨の日の数学」を選んで決めると、欄には名前が見え、
    // 作ったノートに `source: "[[雨の日の数学]]"`。
    let tmp = reading("nfs_link2");
    let mut a = boot(&tmp, READING);
    name_enter(&mut a, "新しい抜き書き");
    select_item(&mut a, "雨の日の数学");
    press(&mut a, KeyCode::Enter);
    assert_eq!(step(&a), 2, "page の欄へ:\n{}", screen(&a));
    let line = field_line(&a, "source");
    assert!(line.contains("雨の日の数学"), "{line}");
    assert!(!line.contains("[["), "{line}");
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Table, "{:?}", a.message);
    let text = read(&tmp, "Quotes/新しい抜き書き.md");
    assert!(text.contains("source: \"[[雨の日の数学]]\""), "{text:?}");
}

#[test]
fn test_link_list_field_candidates_are_notes_of_the_target_folder() {
    // [utest -> req~cell-edit.new-note-form-link-candidates~1] リンクのリストの列(question)の付け外しの窓の候補は、Questions の
    // 問いの名前。どのニュースからも指されていない問いも並ぶ。
    let tmp = Tmp::new("nfs_link3");
    write(&tmp, "Questions/問いA.md", "---\nstatus: open\n---\n");
    write(&tmp, "Questions/問いB.md", "---\nstatus: open\n---\n");
    write(
        &tmp,
        "News/n1.md",
        "---\nquestion:\n  - \"[[問いA]]\"\n---\n",
    );
    let mut a = boot(
        &tmp,
        "[types]\nquestion = \"list\"\n[new_note]\nfolder = \"News\"\nask = [\"question\"]\n",
    );
    name_enter(&mut a, "ニュース");
    assert_eq!(a.mode, Mode::ListPick, "{}", screen(&a));
    let names = pick_names(&a);
    assert!(names.contains(&"問いA".to_string()), "{names:?}");
    assert!(names.contains(&"問いB".to_string()), "{names:?}");
    assert!(!names.iter().any(|n| n.contains("[[")), "{names:?}");
}

// ---- セルと同じ候補と入れ方(cell-edit.new-note-form-same-input) ----

#[test]
fn test_checkbox_field_offers_true_false_none() {
    // [utest -> req~cell-edit.new-note-form-same-input~1] チェックボックスの列の欄は、セルに Tab で着いたときと同じ true・false・なし の候補。
    let tmp = Tmp::new("nfs_check");
    tmp.write("a.md", "---\nfav: true\n---\n");
    let mut a = boot(
        &tmp,
        "[types]\nfav = \"checkbox\"\n[new_note]\nask = [\"fav\"]\n",
    );
    name_enter(&mut a, "x");
    assert_eq!(
        list_texts(&a),
        vec!["true".to_string(), "false".to_string(), "なし".to_string()],
        "{}",
        screen(&a)
    );
}

#[test]
fn test_typing_replaces_preset_date() {
    // [utest -> req~cell-edit.new-note-form-same-input~1] 前もって 7日後の日付が入った due の欄で `2026-12-01` と打つと、置き換わる。
    let tmp = Tmp::new("nfs_date");
    tmp.write("a.md", "---\ndue: 2026-09-20\n---\n");
    let mut a = boot(
        &tmp,
        "[types]\ndue = \"date\"\n[new_note]\nask = [\"due\"]\n\n[new_note.set]\ndue = \"{date+7}\"\n",
    );
    name_enter(&mut a, "x");
    assert_eq!(a.input.as_ref().unwrap().text, "2026-10-09");
    typing(&mut a, "2026-12-01");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table, "{:?}\n{}", a.message, screen(&a));
    assert!(
        read(&tmp, "x.md").contains("\ndue: 2026-12-01\n"),
        "{}",
        read(&tmp, "x.md")
    );
}

#[test]
fn test_typing_replaces_preset_value_with_candidates() {
    // [utest -> req~cell-edit.new-note-form-same-input~1] 前もって todo が入った候補のある status の欄で `done` と打つと、置き換わる。
    let tmp = Tmp::new("nfs_status");
    tmp.write("a.md", "---\nstatus: todo\n---\n");
    let mut a = boot(
        &tmp,
        "[rules.status]\nvalues = [\"todo\", \"done\"]\n[new_note]\nask = [\"status\"]\n\n[new_note.set]\nstatus = \"todo\"\n",
    );
    name_enter(&mut a, "x");
    typing(&mut a, "done");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table, "{:?}\n{}", a.message, screen(&a));
    assert!(
        read(&tmp, "x.md").contains("\nstatus: done\n"),
        "{}",
        read(&tmp, "x.md")
    );
}

// ---- リストの欄のキー(cell-edit.new-note-form-move・cell-edit.new-note-form-create) ----

fn tagged(tag: &str) -> Tmp {
    let tmp = Tmp::new(tag);
    tmp.write("a.md", "---\ntags:\n  - 仕事\npriority: 1\n---\n");
    tmp.write("b.md", "---\ntags:\n  - 家\npriority: 2\n---\n");
    tmp
}

const TAGS: &str = "[types]\ntags = \"list\"\npriority = \"number\"\n";

#[test]
fn test_list_field_enter_toggles_and_tab_moves_on() {
    // [utest -> req~cell-edit.new-note-form-move~3] リストの欄で Enter → チェックが付き、欄のまま。Tab → 次の欄へ移り、欄に「仕事」が見える。
    let tmp = tagged("nfs_keys1");
    let mut a = boot(
        &tmp,
        &format!("{TAGS}[new_note]\nask = [\"tags\", \"priority\"]\n"),
    );
    name_enter(&mut a, "メモ");
    assert_eq!(a.mode, Mode::ListPick, "{}", screen(&a));
    pick_select(&mut a, "仕事");
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        a.mode,
        Mode::ListPick,
        "Enter で窓は閉じない:\n{}",
        screen(&a)
    );
    assert_eq!(step(&a), 1, "tags の欄のまま");
    assert_eq!(pick_mark(&a, "仕事"), Mark::All);
    assert!(!tmp.notes().join("メモ.md").exists());
    press(&mut a, KeyCode::Tab);
    assert_eq!(step(&a), 2, "priority の欄へ:\n{}", screen(&a));
    let line = field_line(&a, "tags");
    assert!(line.contains("仕事"), "{line}");
}

#[test]
fn test_list_field_decide_row_moves_on() {
    // [utest -> req~cell-edit.new-note-form-move~3] リストの欄の窓の一番下に「[ 決める ]」の行があり、その行の Enter で決めて次の欄へ。
    let tmp = tagged("nfs_keys2");
    let mut a = boot(
        &tmp,
        &format!("{TAGS}[new_note]\nask = [\"tags\", \"priority\"]\n"),
    );
    name_enter(&mut a, "メモ");
    pick_select(&mut a, "家");
    press(&mut a, KeyCode::Enter);
    assert!(screen(&a).contains("[ 決める ]"), "{}", screen(&a));
    let n = a.pick.as_ref().unwrap().rows().len();
    a.pick.as_mut().unwrap().sel = n;
    press(&mut a, KeyCode::Enter);
    assert_eq!(step(&a), 2, "priority の欄へ:\n{}", screen(&a));
    assert!(field_line(&a, "tags").contains("家"));
}

#[test]
fn test_list_field_shift_tab_goes_back() {
    // [utest -> req~cell-edit.new-note-form-move~3] リストの欄で Shift+Tab → 名前の欄へ移る。
    let tmp = tagged("nfs_keys3");
    let mut a = boot(
        &tmp,
        &format!("{TAGS}[new_note]\nask = [\"tags\", \"priority\"]\n"),
    );
    name_enter(&mut a, "メモ");
    assert_eq!(a.mode, Mode::ListPick);
    press(&mut a, KeyCode::BackTab);
    assert_eq!(step(&a), 0, "名前の欄へ:\n{}", screen(&a));
    assert_eq!(a.mode, Mode::Edit);
    assert_eq!(a.input.as_ref().unwrap().text, "メモ");
}

#[test]
fn test_last_list_field_enter_does_not_create() {
    // [utest -> req~cell-edit.new-note-form-create~2] 最後の欄がリストの欄なら、Enter ではノートを作らずチェックを付ける。Tab で作る。
    let tmp = tagged("nfs_keys4");
    let mut a = boot(&tmp, &format!("{TAGS}[new_note]\nask = [\"tags\"]\n"));
    name_enter(&mut a, "メモ");
    pick_select(&mut a, "仕事");
    press(&mut a, KeyCode::Enter);
    assert!(!tmp.notes().join("メモ.md").exists(), "Enter では作らない");
    assert_eq!(pick_mark(&a, "仕事"), Mark::All);
    press(&mut a, KeyCode::Tab);
    assert_eq!(a.mode, Mode::Table, "{:?}\n{}", a.message, screen(&a));
    assert_eq!(read(&tmp, "メモ.md"), "---\ntags:\n  - 仕事\n---\n");
}

// ---- 候補の窓の Esc(cell-edit.new-note-form-esc-popup・cell-edit.new-note-form-esc) ----

#[test]
fn test_esc_closes_value_candidates_only() {
    // [utest -> req~cell-edit.new-note-form-esc-popup~1, req~cell-edit.new-note-form-esc~2] status の欄の候補が出ているときの Esc は
    // 候補だけを閉じ、窓と打った名前・url は残る。続けて Esc で窓が閉じ、ノートは作られない。
    let tmp = Tmp::new("nfs_esc1");
    tmp.write("a.md", "---\nstatus: todo\nurl: https://a.example\n---\n");
    let mut a = boot(
        &tmp,
        "[rules.status]\nvalues = [\"todo\", \"done\"]\n[new_note]\nask = [\"url\", \"status\"]\n",
    );
    name_enter(&mut a, "記事");
    typing(&mut a, "https://x.example/1");
    press(&mut a, KeyCode::Enter);
    assert_eq!(step(&a), 2, "{}", screen(&a));
    assert!(
        !list_texts(&a).is_empty(),
        "候補が出ている:\n{}",
        screen(&a)
    );
    press(&mut a, KeyCode::Esc);
    assert!(a.note.flow.is_some(), "窓は開いたまま");
    assert_eq!(a.mode, Mode::Edit, "status の欄の入力は開いたまま");
    assert_eq!(step(&a), 2);
    assert!(a.list_view().is_none(), "候補は閉じた:\n{}", screen(&a));
    let s = screen(&a);
    assert!(
        s.contains("記事") && s.contains("https://x.example/1"),
        "{s}"
    );
    press(&mut a, KeyCode::Esc);
    assert!(a.note.flow.is_none(), "窓が閉じる:\n{}", screen(&a));
    assert_eq!(a.mode, Mode::Table);
    assert!(!tmp.notes().join("記事.md").exists());
}

#[test]
fn test_esc_closes_calendar_only() {
    // [utest -> req~cell-edit.new-note-form-esc-popup~1] due の欄のカレンダーが出ているときの Esc はカレンダーだけを閉じ、入力は開いたまま。
    let tmp = Tmp::new("nfs_esc2");
    tmp.write("a.md", "---\ndue: 2026-09-20\n---\n");
    let mut a = boot(
        &tmp,
        "[types]\ndue = \"date\"\n[new_note]\nask = [\"due\"]\n",
    );
    name_enter(&mut a, "x");
    assert!(
        a.input.as_ref().is_some_and(|i| i.cal.is_some()),
        "カレンダーが出ている:\n{}",
        screen(&a)
    );
    press(&mut a, KeyCode::Esc);
    assert!(a.note.flow.is_some(), "窓は開いたまま:\n{}", screen(&a));
    assert_eq!(a.mode, Mode::Edit);
    assert!(a.input.as_ref().is_some_and(|i| i.cal.is_none()));
    typing(&mut a, "2026-10-05");
    press(&mut a, KeyCode::Enter);
    assert!(read(&tmp, "x.md").contains("\ndue: 2026-10-05\n"));
}

#[test]
fn test_esc_closes_list_pick_only_and_enter_reopens() {
    // [utest -> req~cell-edit.new-note-form-esc-popup~1] tags の付け外しの窓の Esc は、付けたチェックを入れずにその窓だけを閉じ、
    // 新しいノートの窓は tags の欄のまま。Enter で付け外しの窓がまた開き、Tab なら次の欄へ。
    let tmp = tagged("nfs_esc3");
    let mut a = boot(
        &tmp,
        &format!("{TAGS}[new_note]\nask = [\"tags\", \"priority\"]\n"),
    );
    name_enter(&mut a, "メモ");
    pick_select(&mut a, "仕事");
    ch(&mut a, ' ');
    assert_eq!(pick_mark(&a, "仕事"), Mark::All);
    press(&mut a, KeyCode::Esc);
    assert!(a.note.flow.is_some(), "窓は開いたまま:\n{}", screen(&a));
    assert_eq!(step(&a), 1, "tags の欄のまま");
    assert!(
        !screen(&a).contains("[ 決める ]"),
        "付け外しの窓は閉じた:\n{}",
        screen(&a)
    );
    press(&mut a, KeyCode::Enter);
    assert!(
        screen(&a).contains("[ 決める ]"),
        "また開く:\n{}",
        screen(&a)
    );
    assert_eq!(
        pick_mark(&a, "仕事"),
        Mark::None,
        "付けたチェックは入っていない"
    );
    press(&mut a, KeyCode::Esc);
    press(&mut a, KeyCode::Tab);
    assert_eq!(step(&a), 2, "priority の欄へ:\n{}", screen(&a));
}

// ---- 理由の印を消す(cell-edit.new-note-form-error-clear) ----

fn has_mark(line: &str) -> bool {
    line.contains(" ! ")
}

#[test]
fn test_required_mark_clears_when_filled() {
    // [utest -> req~cell-edit.new-note-form-error-clear~1] 必須の due を空のまま Ctrl+S で出た印は、due に値を入れて決めたら消える。
    let tmp = Tmp::new("nfs_err1");
    tmp.write("a.md", "---\ndue: 2026-09-20\npriority: 1\n---\n");
    let mut a = boot(
        &tmp,
        "[types]\ndue = \"date\"\npriority = \"number\"\n[new_note]\nask = [\"due\", \"priority\"]\nrequired = [\"due\"]\n",
    );
    ch(&mut a, 'a');
    typing(&mut a, "x");
    ctrl(&mut a, 's');
    assert_eq!(step(&a), 1, "due の欄へ:\n{}", screen(&a));
    // 今の欄から離れると印が見える(いまの欄は入力の欄なので、priority へ移ってから見る)。
    typing(&mut a, "2026-10-05");
    press(&mut a, KeyCode::Enter);
    assert_eq!(step(&a), 2, "{}", screen(&a));
    let line = field_line(&a, "due");
    assert!(!has_mark(&line), "{line}");
}

#[test]
fn test_required_mark_stays_while_empty() {
    // [utest -> req~cell-edit.new-note-form-error-clear~1] 必須の due を空のまま決めても、印は残る。
    let tmp = Tmp::new("nfs_err2");
    tmp.write("a.md", "---\ndue: 2026-09-20\npriority: 1\n---\n");
    let mut a = boot(
        &tmp,
        "[types]\ndue = \"date\"\npriority = \"number\"\n[new_note]\nask = [\"due\", \"priority\"]\nrequired = [\"due\"]\n",
    );
    ch(&mut a, 'a');
    typing(&mut a, "x");
    ctrl(&mut a, 's');
    assert_eq!(step(&a), 1);
    press(&mut a, KeyCode::Enter);
    assert_eq!(step(&a), 2, "{}", screen(&a));
    let line = field_line(&a, "due");
    assert!(has_mark(&line), "{line}");
}

#[test]
fn test_bad_name_mark_clears_when_fixed() {
    // [utest -> req~cell-edit.new-note-form-error-clear~1] 名前「c」を決めたあとにほかの道具で c.md ができ、作るときに名前の欄へ
    // 戻って出た印は、名前を「d」に直して決めたら消える。
    let tmp = Tmp::new("nfs_err3");
    tmp.write("a.md", "---\npriority: 1\n---\n");
    let mut a = boot(
        &tmp,
        "[types]\npriority = \"number\"\n[new_note]\nask = [\"priority\"]\n",
    );
    ch(&mut a, 'a');
    typing(&mut a, "c");
    press(&mut a, KeyCode::Enter);
    assert_eq!(step(&a), 1);
    tmp.write("c.md", "---\npriority: 9\n---\n");
    ctrl(&mut a, 's');
    assert_eq!(step(&a), 0, "名前の欄へ戻る:\n{}", screen(&a));
    assert!(a.message.is_some(), "名前の欄に理由");
    press(&mut a, KeyCode::Backspace);
    typing(&mut a, "d");
    press(&mut a, KeyCode::Enter);
    assert_eq!(step(&a), 1, "{}", screen(&a));
    let line = field_line(&a, "名前");
    assert!(line.contains('d') && !has_mark(&line), "{line}");
    assert_eq!(read(&tmp, "c.md"), "---\npriority: 9\n---\n");
}

// ---- 頭を隠すときの {name}(cell-edit.template-vars-name-head) ----

const HEAD_BODY: &str = "[display]\nnote_name = '^\\d{8}_'\n[new_note]\nname = \"{date:YYYYMMDD}_\"\nbody = \"templates/t.md\"\n";

#[test]
fn test_name_variable_has_no_hidden_head() {
    // [utest -> req~cell-edit.template-vars-name-head~1] name_head = "hide" で「テスト」と打って作ると、本文の雛形の `# {name}` は `# テスト`。
    let tmp = Tmp::new("nfs_name1");
    write(&tmp, "templates/t.md", "# {name}\n");
    let mut a = boot(&tmp, &format!("{HEAD_BODY}name_head = \"hide\"\n"));
    ch(&mut a, 'a');
    typing(&mut a, "テスト");
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Table, "{:?}", a.message);
    let text = read(&tmp, "20261002_テスト.md");
    assert!(text.ends_with("# テスト\n"), "{text:?}");
}

#[test]
fn test_name_variable_keeps_shown_head() {
    // [utest -> req~cell-edit.template-vars-name-head~1] name_head を書かなければ、名前の欄の `20261002_` ごと `{name}` に入る(今までどおり)。
    let tmp = Tmp::new("nfs_name2");
    write(&tmp, "templates/t.md", "# {name}\n");
    let mut a = boot(&tmp, HEAD_BODY);
    ch(&mut a, 'a');
    assert_eq!(a.input.as_ref().unwrap().text, "20261002_");
    typing(&mut a, "テスト");
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Table, "{:?}", a.message);
    let text = read(&tmp, "20261002_テスト.md");
    assert!(text.ends_with("# 20261002_テスト\n"), "{text:?}");
}
