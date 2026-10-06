//! [WB-5] 読めない YAML の理由に、ファイルの行と誤りの文を添える(specs/_changes/2026-10-06-yaml-error-detail.md)。

use super::yaml_error;

#[test]
fn test_wb_5_yaml_error_unclosed_flow_list() {
    let (line, msg) = yaml_error(b"---\ntitle: x\ntags: [a, b\n---\nbody\n").expect("誤り");
    assert!(
        (3..=4).contains(&line),
        "閉じていないリストの行の近く: {line}"
    );
    assert!(!msg.is_empty());
}

#[test]
fn test_wb_5_yaml_error_none_for_valid_yaml() {
    assert_eq!(yaml_error(b"---\ntitle: x\ntags: [a, b]\n---\n"), None);
    // フロントマターが無いものは YAML の誤りとしては出さない(別の理由がある)。
    assert_eq!(yaml_error(b"no frontmatter\n"), None);
}
