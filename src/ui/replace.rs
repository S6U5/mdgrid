//! 今の列の値を正規表現で探して置き換える(`impl App` の続き。CE-15)。
//! 置き換えは、変わったセルをまとめて、ためる変更の1手にする(`Changes::set_each`)。ファイルは保存の流れで書く。

use super::app::App;
use super::input::bulk_tail;
use super::native_io::Ask;
use super::startup::READONLY;
use mdgrid::i18n::Msg;
use mdgrid::source::{NewValue, RowId, Value};
use mdgrid::types::Kind;
use regex::Regex;

impl App {
    /// パレット・操作の一覧の「正規表現で置き換える」: 今の列がテキストの列なら、探す正規表現を聞く。
    pub(crate) fn start_replace(&mut self) {
        if self.readonly {
            self.message = Some(READONLY.into());
            return;
        }
        let Some(col) = self.cols.get(self.col).cloned() else {
            return;
        };
        // 型と合わない値を黙って書かないため、テキストの列(ノートのキー)でだけ始める。
        if !self.replace_column_ok(&col) {
            self.message = Some(Msg::ReplaceNotText.into());
            return;
        }
        self.open_ask(Ask::ReplacePattern(col));
    }

    /// 置き換えを始められる列か: ノートのキーの列で、列の型がテキスト。
    pub(crate) fn replace_column_ok(&self, col: &str) -> bool {
        !col.starts_with("file.") && !col.starts_with("formula.") && self.kind_of(col) == Kind::Text
    }

    /// 探す正規表現を受ける。読めなければ何も変えずに理由を出し、読めたら置き換える文字を聞く。
    pub(crate) fn replace_pattern(&mut self, col: &str, query: &str) {
        if query.is_empty() {
            self.close_palette();
            self.message = Some(Msg::AskCancelled.into());
            return;
        }
        if let Err(e) = Regex::new(query) {
            self.close_palette();
            self.message = Some(Msg::ReplaceBadPattern.fill(&[&regex_reason(&e)]));
            return;
        }
        self.open_ask(Ask::ReplaceWith {
            col: col.to_string(),
            pattern: query.to_string(),
        });
    }

    /// 置き換える文字を受けて、対象のセルに当てる。対象は、行を選んでいれば選んだ行、いなければ今のビューの全部の行
    /// (畳んだまとまりの中の行は、見えていないので入れない。CE-10 と同じ)。
    pub(crate) fn replace_with(&mut self, col: &str, pattern: &str, with: &str) {
        self.close_palette();
        let re = match Regex::new(pattern) {
            Ok(re) => re,
            Err(e) => {
                self.message = Some(Msg::ReplaceBadPattern.fill(&[&regex_reason(&e)]));
                return;
            }
        };
        // 聞いている間に列が変わっていたら(外の変化で型が変わった・列が消えた)、当てない。
        if !self.cols.iter().any(|c| c == col) || !self.replace_column_ok(col) {
            self.message = Some(Msg::ReplaceNotText.into());
            return;
        }
        let (rows, folded_out) = self.replace_rows();
        let mut reasons: Vec<String> = Vec::new();
        let mut items: Vec<(RowId, Option<NewValue>)> = Vec::new();
        for r in rows {
            // 値が文字のセルだけ(ためた値があればその値)。数・真偽・日付・リスト・空は変えない。
            let text = match self.changes.pending(&r, col) {
                Some(NewValue::Str(s)) => s.clone(),
                Some(_) => continue,
                None => {
                    let cell = self.src.get(&r, col);
                    match (cell.value, cell.lock) {
                        (Some(Value::Str(s)), _) => s,
                        // 読めないノートのセル(値が分からない)は、一致するかを確かめられないので、理由を数える。
                        (None, Some(reason)) => {
                            reasons.push(reason);
                            continue;
                        }
                        _ => continue,
                    }
                }
            };
            // 空のセル(空の文字列も)は変えない。
            if text.is_empty() || !re.is_match(&text) {
                continue;
            }
            let new = re.replace_all(&text, with).into_owned();
            if new == text {
                continue;
            }
            // 一致したが直せないセル(読むだけ・改行を含む値)は、理由を数える。
            if text.contains(['\n', '\r']) || new.contains(['\n', '\r']) {
                reasons.push(Msg::EditMultiLine.text().to_string());
                continue;
            }
            items.push((r, Some(NewValue::Str(new))));
        }
        let skips = self.changes.set_each(self.src.as_ref(), col, &items);
        reasons.extend(skips.iter().map(|s| s.reason.clone()));
        let done = items.len() - skips.len();
        self.stay.extend(items.into_iter().map(|(r, _)| r));
        self.regrid = true;
        let mut msg = if done == 0 && reasons.is_empty() {
            Msg::ReplaceNoMatch.text().to_string()
        } else {
            Msg::ReplaceDone.fill(&[&done])
        };
        bulk_tail(&mut msg, folded_out, reasons);
        self.message = Some(msg);
    }

    /// 対象の行と、畳んだまとまりの中で外した行の数。
    fn replace_rows(&self) -> (Vec<RowId>, usize) {
        let sel = self.selection();
        let all = if sel.is_empty() {
            self.rows.clone()
        } else {
            sel
        };
        let hidden: std::collections::HashSet<&RowId> = self
            .groups
            .iter()
            .filter(|(h, _)| self.folded.contains(h))
            .flat_map(|(_, r)| self.rows[r.clone()].iter())
            .collect();
        let n = all.len();
        let shown: Vec<RowId> = all.into_iter().filter(|r| !hidden.contains(r)).collect();
        let out = n - shown.len();
        (shown, out)
    }
}

/// 正規表現の読めない理由を1行にする(regex の文は何行かに分かれ、最後の行が理由)。
fn regex_reason(e: &regex::Error) -> String {
    let text = e.to_string();
    text.lines()
        .rev()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("")
        .trim_start_matches("error: ")
        .to_string()
}
