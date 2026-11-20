//! [base-view.expr-eval] 年の端の週でも .format() が null にならない(specs/_history/decidespec/_changes/2026-10-06-review6-fixes.md)。

use super::format_when;
use crate::types::parse_date;

// [utest -> req~base-view.expr-eval~1]
#[test]
fn test_bv_6_format_year_edges() {
    let d = parse_date("9999-12-31").unwrap();
    assert_eq!(
        format_when(d, 0, "YYYY-MM-DD").as_deref(),
        Some("9999-12-31")
    );
    let d = parse_date("0000-01-01").unwrap();
    assert_eq!(format_when(d, 0, "YYYY").as_deref(), Some("0000"));
}
