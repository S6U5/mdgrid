//! カタログの色の欄(SR-38・SR-40): 役割と色の名前が本体(src/colors.rs)と同じか。

use serde_json::Value;

fn names() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/catalog/index.html");
    let html = std::fs::read_to_string(p).unwrap();
    let head = r#"<script type="application/json" id="names">"#;
    let from = html.find(head).expect("名前の表が無い") + head.len();
    let to = from + html[from..].find("</script>").unwrap();
    serde_json::from_str(&html[from..to]).unwrap()
}

fn strs(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn test_sr_38_catalog_colors() {
    // [SR-38][SR-40] カタログの色の役割と色の名前が本体と同じ(増減・改名したら落ちる)。
    let n = names();
    let roles: Vec<String> = mdgrid::colors::ROLES
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(strs(&n["colorRoles"]), roles);
    let named: Vec<String> = mdgrid::colors::NAMED
        .iter()
        .map(|(s, _)| s.to_string())
        .collect();
    assert_eq!(strs(&n["colorNames"]), named);
}
