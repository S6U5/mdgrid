//! タスク 1(oss-config)の受け入れ: エディタの決め方(SR-8)と、設定の `editor` の読み取り(CLI-3)。
//! 実装を見ずに書いた。環境変数は書き換えない(並列の試験でぶつかるため)。
//!
//! 仮定した公開の形(最小):
//!
//! ```ignore
//! // src/config.rs
//! pub struct Config { /* … */ pub editor: Option<String>, /* … */ }
//! /// 設定の editor > $VISUAL > $EDITOR の順で空でない最初のもの。どれも無ければ "vi"。
//! /// 呼ぶ側(main)が環境変数を読んで渡す純関数。
//! pub fn resolve_editor(config: Option<&str>, visual: Option<&str>, editor: Option<&str>) -> String;
//! ```

use mdgrid::config::{parse, resolve_editor, Config};

#[test]
fn test_sr_8_config_editor_wins_over_env() {
    // [SR-8] 設定に editor = "nvim" と EDITOR=vim → nvim。
    assert_eq!(resolve_editor(Some("nvim"), None, Some("vim")), "nvim");
    // 設定は VISUAL にも勝つ。
    assert_eq!(
        resolve_editor(Some("nvim"), Some("hx"), Some("vim")),
        "nvim"
    );
}

#[test]
fn test_sr_8_visual_before_editor() {
    // [SR-8] 設定が無く VISUAL=hx・EDITOR=vim → hx。
    assert_eq!(resolve_editor(None, Some("hx"), Some("vim")), "hx");
}

#[test]
fn test_sr_8_editor_env_when_no_config_or_visual() {
    // [SR-8] 設定も VISUAL も無い → EDITOR。
    assert_eq!(resolve_editor(None, None, Some("vim")), "vim");
}

#[test]
fn test_sr_8_none_falls_back_to_vi() {
    // [SR-8] どれも無い → vi。
    assert_eq!(resolve_editor(None, None, None), "vi");
}

#[test]
fn test_sr_8_empty_is_same_as_absent() {
    // [SR-8] 空は無いのと同じ。設定の editor = "" は無いのと同じ。
    assert_eq!(resolve_editor(Some(""), Some("hx"), Some("vim")), "hx");
    assert_eq!(resolve_editor(Some(""), None, Some("vim")), "vim");
    assert_eq!(resolve_editor(Some(""), Some(""), Some("vim")), "vim");
    assert_eq!(resolve_editor(None, Some(""), Some("")), "vi");
    assert_eq!(resolve_editor(Some(""), Some(""), Some("")), "vi");
}

#[test]
fn test_sr_8_value_with_args_kept_whole() {
    // [SR-8] 引数つきの値(`code -w`)はそのまま返す(単語分けは起動の側)。
    assert_eq!(resolve_editor(Some("code -w"), Some("hx"), None), "code -w");
    assert_eq!(resolve_editor(None, None, Some("code -w")), "code -w");
}

#[test]
fn test_sr_8_config_reads_editor_string() {
    // [SR-8] [CLI-3] 設定の editor = "nvim" を読み、知らない項目の警告は出ない。
    let (c, warnings) = parse("editor = \"nvim\"\n").expect("読める");
    assert!(warnings.is_empty(), "警告: {:?}", warnings);
    assert_eq!(c.editor.as_deref(), Some("nvim"));
    assert_eq!(
        resolve_editor(c.editor.as_deref(), Some("hx"), Some("vim")),
        "nvim"
    );
}

#[test]
fn test_sr_8_config_editor_default_is_absent() {
    // [SR-8] 既定(設定が無い)では editor を持たず、環境変数に任せる。
    assert_eq!(Config::default().editor, None);
    let (c, _) = parse("").expect("読める");
    assert_eq!(
        resolve_editor(c.editor.as_deref(), Some("hx"), Some("vim")),
        "hx"
    );
}

#[test]
fn test_sr_8_config_empty_editor_no_warning_and_absent() {
    // [SR-8] [CLI-3] 設定の editor = "" は警告なしに読め、無いのと同じ。
    let (c, warnings) = parse("editor = \"\"\n").expect("読める");
    assert!(warnings.is_empty(), "警告: {:?}", warnings);
    assert_eq!(
        resolve_editor(c.editor.as_deref(), None, Some("vim")),
        "vim"
    );
    assert_eq!(resolve_editor(c.editor.as_deref(), None, None), "vi");
}

#[test]
fn test_sr_8_config_editor_not_string_warns() {
    // [SR-8] [CLI-3] 文字列でない editor は警告して無視し、止めない。
    for text in ["editor = 3\n", "editor = true\n", "editor = [\"nvim\"]\n"] {
        let (c, warnings) = parse(text).expect("型が違っても Ok");
        assert!(
            warnings.iter().any(|w| w.contains("editor")),
            "{text:?} で editor の警告が無い: {:?}",
            warnings
        );
        assert_eq!(c.editor, None, "{text:?}");
        assert_eq!(
            resolve_editor(c.editor.as_deref(), None, Some("vim")),
            "vim"
        );
    }
}

#[test]
fn test_sr_8_config_editor_keeps_other_items() {
    // [SR-8] [CLI-3] editor を足しても、ほかの項目の読み取りは変わらない。
    let (c, warnings) = parse("editor = \"code -w\"\n\n[edit]\ncandidates = 7\n").expect("読める");
    assert!(warnings.is_empty(), "警告: {:?}", warnings);
    assert_eq!(c.editor.as_deref(), Some("code -w"));
    assert_eq!(c.resolved().candidates, 7);
}
