//! カタログの nerd_font の既定(SR-38・SR-36): 設定に書く既定は "auto"(本体の項目の表と同じ)。

#[test]
fn test_sr_38_catalog_nerd_default() {
    // [SR-38][SR-36] カタログの nerd_font の既定の書き方が、本体の項目の表の既定と同じ。
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/catalog/index.html");
    let html = std::fs::read_to_string(p).unwrap();
    let head = r#"<script type="application/json" id="names">"#;
    let from = html.find(head).unwrap() + head.len();
    let to = from + html[from..].find("</script>").unwrap();
    let n: serde_json::Value = serde_json::from_str(&html[from..to]).unwrap();
    let item = mdgrid::config::ITEMS
        .iter()
        .find(|i| i.name == "nerd_font")
        .unwrap();
    let want = item.default.unwrap().trim_matches('"');
    assert_eq!(n["defaults"]["nerd_font_setting"].as_str(), Some(want));
    let (c, _) = mdgrid::config::parse("").unwrap();
    assert!(c.nerd_font_auto);
}
