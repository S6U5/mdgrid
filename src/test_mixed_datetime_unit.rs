//! [CE-2] 推定で日付になった列に日時の値もあれば日時の列(specs/_decisions/2026-10-06-mixed-datetime.md)。

use super::{fits, infer, Kind};
use crate::frontmatter::Value;

fn s(x: &str) -> Value {
    Value::Str(x.into())
}

#[test]
fn test_ce_2_mixed_date_datetime_is_datetime() {
    let vals = [Value::Null, s("2026-08-01"), s("2026-08-03T08:15:00")];
    let k = infer(vals.iter());
    assert_eq!(k, Kind::DateTime);
    // 日付だけの値も日時の列に合う(`!` が付かない)。
    assert!(vals.iter().all(|v| fits(k, v)));
}

#[test]
fn test_ce_2_mixed_date_datetime_dates_only_stay_date() {
    assert_eq!(infer([s("2026-08-01"), s("2026-08-02")].iter()), Kind::Date);
}

#[test]
fn test_ce_2_mixed_date_datetime_first_value_rule_otherwise() {
    // 最初の値で日付でない型に決まった列は今までどおり(日時の値があっても変えない)。
    assert_eq!(
        infer([s("someday"), s("2026-08-03T08:15")].iter()),
        Kind::Text
    );
    assert_eq!(
        infer([Value::Int(1), s("2026-08-03T08:15")].iter()),
        Kind::Number
    );
    // 最初が日時なら日時。
    assert_eq!(
        infer([s("2026-08-03T08:15"), s("2026-08-01")].iter()),
        Kind::DateTime
    );
}
