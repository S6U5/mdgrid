//! ボードの画面の核(specs/board/spec.md の BD-2): 行をまとめる列の値ごとのレーンに分ける。画面に頼らない。

use crate::source::RowId;

/// 1つのレーン。`value` が None は、値が空の行のレーン「(なし)」。`rows` は渡した並びのまま。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lane {
    pub value: Option<String>,
    pub rows: Vec<RowId>,
}

/// 行(と、まとめる列の値の文字。空は None)をレーンに分ける(BD-2)。
/// レーンの並び: `order`(列の決まりの values)の値を先に、その順で(行が無くても出す。重なりは最初の1つ)。
/// 残りの値は文字の順。値が空の行のレーンは、空の行があるときだけ最後。レーンの中は渡した行の並び。
pub fn lanes(rows: &[(RowId, Option<String>)], order: &[String]) -> Vec<Lane> {
    let mut out: Vec<Lane> = Vec::new();
    for v in order {
        if !out.iter().any(|l| l.value.as_deref() == Some(v.as_str())) {
            out.push(Lane {
                value: Some(v.clone()),
                rows: Vec::new(),
            });
        }
    }
    let fixed = out.len();
    let mut none: Vec<RowId> = Vec::new();
    for (row, value) in rows {
        match value.as_deref().filter(|s| !s.is_empty()) {
            None => none.push(row.clone()),
            Some(v) => match out.iter_mut().find(|l| l.value.as_deref() == Some(v)) {
                Some(l) => l.rows.push(row.clone()),
                None => out.push(Lane {
                    value: Some(v.to_string()),
                    rows: vec![row.clone()],
                }),
            },
        }
    }
    out[fixed..].sort_by(|a, b| a.value.cmp(&b.value));
    if !none.is_empty() {
        out.push(Lane {
            value: None,
            rows: none,
        });
    }
    out
}
