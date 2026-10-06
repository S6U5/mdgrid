//! 保存の確認の画面(`impl App` の続き。WB-9・WB-11 の「保存する」・WB-14・WB-16)。

use super::app::{App, ColorMode};
use super::diff::{diff, DiffLine};
use super::keymap::{Action, Mode};
use super::view::{footer, message};
use super::width::{fit, sanitize, Align};
use mdgrid::changes::Outcome;
use mdgrid::i18n::Msg;
use mdgrid::source::{content_hash, EditError, RowId, SaveError};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

/// 保存の確認の画面の1ファイル(WB-9・WB-16)。
pub(crate) struct ReviewItem {
    pub row: RowId,
    pub label: String,
    /// 外で変わった(WB-16)。
    pub external: bool,
    /// 差分、または書けない理由。
    pub diff: Result<Vec<DiffLine>, String>,
    /// 見せた差分の元の内容(今のファイル)のハッシュ。WB-16 の `o` で、見せた内容の上にだけ書く。
    pub seen: Option<[u8; 32]>,
}

/// 保存の確認の画面の状態。
pub(crate) struct Review {
    pub items: Vec<ReviewItem>,
    /// 選んだファイル(`o`・`d` の対象)。
    pub sel: usize,
}

/// 書けない理由の短い日本語(表示用)。
pub(crate) fn edit_error_text(e: &EditError) -> String {
    match e {
        EditError::ReadOnly(r) => Msg::ReviewReadOnly.fill(&[&format!("{r:?}")]),
        EditError::NotEditable(s) => s.clone(),
        EditError::Newline => Msg::ReviewNewline.into(),
        EditError::Verify(s) => Msg::ReviewVerify.fill(&[&s]),
    }
}

pub(crate) fn save_error_text(e: &SaveError) -> String {
    match e {
        SaveError::Changed => Msg::ReviewChanged.into(),
        SaveError::Edit(e) => edit_error_text(e),
        SaveError::Io(e) => Msg::ReviewIo.fill(&[&e]),
        SaveError::NoPermission => Msg::ReviewReadOnly.fill(&[&Msg::NoPermission.text()]),
    }
}

impl App {
    /// 保存の確認と終了の確認の動作。
    pub(crate) fn review_action(&mut self, action: Action) {
        match action {
            Action::SaveAll => self.save(),
            Action::Back => {
                self.review = None;
                self.quit_after_save = false;
                self.set_mode(Mode::Table);
            }
            Action::NextFile => {
                if let Some(r) = &mut self.review {
                    r.sel = (r.sel + 1).min(r.items.len().saturating_sub(1));
                }
            }
            Action::PrevFile => {
                if let Some(r) = &mut self.review {
                    r.sel = r.sel.saturating_sub(1);
                }
            }
            Action::Overwrite => self.overwrite(),
            Action::DiscardRow => {
                if let Some(row) = self.review_row() {
                    let label = self.src.label(&row);
                    self.changes.discard(&row);
                    self.after_review_change(Msg::ReviewDiscarded.fill(&[&label]));
                }
            }
            Action::QuitSave => {
                // WB-17: 読んだ値と同じセルだけなら書くものは無い。保存して終わる操作なので、そのまま終わる。
                if !self.drop_same_pending() {
                    self.quit = true;
                    return;
                }
                self.quit_after_save = true;
                self.open_review();
            }
            Action::QuitDiscard => self.quit = true,
            Action::Help => self.open_help(),
            _ => {}
        }
    }

    /// 保存の確認の差分を作り直す。ヘルプ・パレットを重ねている間はモードを変えない
    /// (確認の画面が無くなったら、ヘルプを閉じたときに表へ戻る)。
    pub(crate) fn reopen_review_keeping_overlay(&mut self) {
        let mode = self.mode;
        self.open_review();
        if matches!(mode, Mode::Help | Mode::Palette) {
            self.mode = mode;
            if self.review.is_none() {
                if let Some(h) = &mut self.help {
                    if h.from == Mode::Confirm {
                        h.from = Mode::Table;
                    }
                }
            }
        }
    }

    /// 保存の確認の画面を(作り直して)開く(WB-9)。ためた変更が無ければ表に戻る。
    pub(crate) fn open_review(&mut self) {
        let sel = self.review.as_ref().map(|r| r.sel).unwrap_or(0);
        let items: Vec<ReviewItem> = self
            .changes
            .previews(self.src.as_ref())
            .into_iter()
            .map(|p| match p {
                Ok(p) => ReviewItem {
                    label: self.src.label(&p.row),
                    external: p.external,
                    diff: Ok(diff(&p.before, &p.after)),
                    seen: Some(content_hash(&p.before)),
                    row: p.row,
                },
                Err((row, e)) => ReviewItem {
                    label: self.src.label(&row),
                    external: self.changes.external(&row),
                    diff: Err(edit_error_text(&e)),
                    seen: None,
                    row,
                },
            })
            .collect();
        if items.is_empty() {
            self.review = None;
            self.set_mode(Mode::Table);
            return;
        }
        let sel = sel.min(items.len() - 1);
        self.review = Some(Review { items, sel });
        self.set_mode(Mode::Confirm);
    }

    fn review_row(&self) -> Option<RowId> {
        let r = self.review.as_ref()?;
        Some(r.items.get(r.sel)?.row.clone())
    }

    /// 保存の確認を開く前に、ためた行の読んだ値と同じセルを外す(WB-17。undo で戻ったセルなど)。
    /// 書くものが残れば true。
    pub(crate) fn drop_same_pending(&mut self) -> bool {
        let rows = self.changes.rows();
        self.changes.drop_same(self.src.as_ref(), &rows);
        self.changes.count() > 0
    }

    /// 保存の確認の画面で変更を減らしたあと:残りがあれば差分を作り直し、無ければ表へ。
    fn after_review_change(&mut self, msg: String) {
        if self.changes.count() == 0 && self.quit_after_save {
            self.quit = true;
        }
        self.open_review();
        if self.mode != Mode::Confirm {
            self.quit_after_save = false;
        }
        self.refresh();
        self.message = Some(msg);
    }

    /// 全部のファイルを書く(WB-9)。行ごとの結果をメッセージ行に出し、書けなかった変更は残す(WB-14)。
    /// 外で変わって止まった行があれば、保存の確認の画面に残って選ばせる(WB-16)。
    fn save(&mut self) {
        let results = self.changes.save(self.src.as_mut());
        self.report(results, false);
    }

    /// 保存の結果を出す。止まった行があるか、1ファイルだけ書いて残りがあれば、保存の確認の画面に残る。
    fn report(&mut self, results: Vec<(RowId, Outcome)>, one: bool) {
        // NV-12: 保存したら、留めていた行を本来の位置へ。
        self.stay.clear();
        let mut saved = Vec::new();
        let mut stopped = Vec::new();
        let mut failed = Vec::new();
        for (row, out) in &results {
            let label = self.src.label(row);
            match out {
                Outcome::Saved => saved.push(label),
                Outcome::Changed => stopped.push(label),
                Outcome::Failed(e) => failed.push(format!("{label}: {}", save_error_text(e))),
            }
        }
        let mut parts = vec![Msg::ReviewSaved.fill(&[&saved.len()])];
        if !saved.is_empty() {
            parts[0].push_str(&Msg::ReviewSavedList.fill(&[&saved.join(", ")]));
        }
        if !stopped.is_empty() {
            parts.push(Msg::ReviewStopped.fill(&[&stopped.len(), &stopped.join(", ")]));
        }
        if !failed.is_empty() {
            parts.push(Msg::ReviewFailed.fill(&[&failed.len(), &failed.join(", ")]));
        }
        let msg = parts.join(Msg::SentenceSep.text());
        if stopped.is_empty() && !(one && self.changes.count() > 0) {
            // 止まった行が無ければ表に戻る(書けなかった変更は残る)。
            if self.changes.count() == 0 && self.quit_after_save {
                self.quit = true;
            }
            self.review = None;
            self.quit_after_save = false;
            self.set_mode(Mode::Table);
            self.refresh();
            self.message = Some(msg);
        } else {
            self.open_review();
            self.refresh();
            self.message = Some(msg);
        }
    }

    /// WB-16 の「外の変更の上に書く」: 選んだファイルだけを、見せた差分の元の内容を基準にして書く。
    /// 見せたあとにまた外で変わっていたら書かず、差分を作り直して見直してもらう(WB-9)。
    fn overwrite(&mut self) {
        let Some(r) = &self.review else {
            return;
        };
        let Some(item) = r.items.get(r.sel) else {
            return;
        };
        let (row, seen) = (item.row.clone(), item.seen);
        let label = self.src.label(&row);
        if !self.changes.external(&row) {
            self.message = Some(Msg::ReviewNotExternal.fill(&[&label]));
            return;
        }
        let Some(seen) = seen else {
            self.message = Some(Msg::ReviewNoDiff.fill(&[&label]));
            return;
        };
        match self.changes.overwrite_seen(self.src.as_mut(), &row, seen) {
            Err(e) => self.message = Some(Msg::ReviewCannotReload.fill(&[&label, &e])),
            Ok(false) => {
                self.open_review();
                self.refresh();
                self.message = Some(Msg::ReviewChangedAgain.fill(&[&label]));
            }
            Ok(true) => {
                let results = self.changes.save_rows(self.src.as_mut(), &[row]);
                self.report(results, true);
            }
        }
    }
}

// ---- 描画(純関数) ----

fn diff_color(mode: ColorMode, add: bool) -> Option<Color> {
    match (mode, add) {
        (ColorMode::None, _) => None,
        (ColorMode::Indexed, true) => Some(Color::Indexed(2)),
        (ColorMode::Indexed, false) => Some(Color::Indexed(1)),
        (ColorMode::Rgb, true) => Some(Color::Rgb(80, 200, 120)),
        (ColorMode::Rgb, false) => Some(Color::Rgb(230, 90, 90)),
    }
}

/// 差分の画面の本文(WB-9・WB-16)と、各ファイルの見出しの行の位置。
fn review_body(app: &App, r: &Review, w: usize) -> (Vec<Line<'static>>, Vec<usize>) {
    let mut out = Vec::new();
    let mut heads = Vec::new();
    let bold = Style::default().add_modifier(Modifier::BOLD);
    for (k, item) in r.items.iter().enumerate() {
        if k > 0 {
            out.push(Line::from(" ".repeat(w)));
        }
        heads.push(out.len());
        let mut head = format!(
            "{}{}{}",
            if k == r.sel { ">" } else { " " },
            if item.external { "!" } else { "" },
            sanitize(&item.label)
        );
        if item.external {
            head.push_str(Msg::ReviewExternalHead.text());
        }
        out.push(Line::from(Span::styled(fit(&head, w, Align::Left), bold)));
        match &item.diff {
            Err(e) => out.push(Line::from(fit(
                &Msg::ReviewDiffError.fill(&[&sanitize(e)]),
                w,
                Align::Left,
            ))),
            Ok(lines) => {
                for d in lines {
                    let (t, color) = match d {
                        DiffLine::Same(t) => (format!("   {}", sanitize(t)), None),
                        DiffLine::Del(t) => (
                            format!("  -{}", sanitize(t)),
                            super::theme::meaning(app, |p| p.del).or(diff_color(app.color, false)),
                        ),
                        DiffLine::Add(t) => (
                            format!("  +{}", sanitize(t)),
                            super::theme::meaning(app, |p| p.add).or(diff_color(app.color, true)),
                        ),
                        DiffLine::Gap => ("   …".to_string(), None),
                    };
                    let mut st = Style::default();
                    if let Some(c) = color {
                        st = st.fg(c);
                    }
                    out.push(Line::from(Span::styled(fit(&t, w, Align::Left), st)));
                }
            }
        }
    }
    (out, heads)
}

/// 保存の確認の画面(WB-9): ヘッダー・ファイルごとの差分・下の帯・メッセージ行。
pub(crate) fn render_review(app: &App, r: &Review, w: usize, h: usize) -> Vec<Line<'static>> {
    let head = Msg::ReviewHead.fill(&[&r.items.len()]);
    let mut lines = vec![Line::from(Span::styled(
        fit(&head, w, Align::Left),
        Style::default().add_modifier(Modifier::BOLD),
    ))];
    let body_h = h.saturating_sub(3);
    let (body, heads) = review_body(app, r, w);
    // 選んだファイルの見出しが見えるように流す。
    let sel_head = heads.get(r.sel).copied().unwrap_or(0);
    let scroll = if body.len() <= body_h {
        0
    } else {
        sel_head.min(body.len() - body_h)
    };
    lines.extend(body.into_iter().skip(scroll).take(body_h));
    while lines.len() < h.saturating_sub(2) {
        lines.push(Line::from(" ".repeat(w)));
    }
    lines.truncate(h.saturating_sub(2));
    lines.push(footer(app, w));
    lines.push(message(app, w));
    lines.truncate(h);
    lines
}
