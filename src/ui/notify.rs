//! 開いている間に、新しく当たった行を知らせる(notify.while-open。`impl App` の続き)。
//!
//! 開いたとき・保存したあと・`notify::INTERVAL` ごとに、裏の糸で範囲の表をファイルから読み(`crate::notify_hits`)、
//! 記録と比べて鳴らす(`mdgrid::notify::run`)。画面でためた変更は数えない(書いたあとに数える)。端末の合図は
//! 糸から受け取り、描いたあとに端末へ送る(描く途中の出力に混ぜない)。届かなかった理由は下の帯に、同じ理由は
//! 開いている間に1回だけ出す(notify.fail)。

use super::app::App;
use mdgrid::condition::Conditions;
use mdgrid::notify::{self, Check, Item, Report};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::time::Instant;

/// 糸の結果(確かめの結果と、端末へ送るバイト)。
type Outcome = (Report, Vec<u8>);

/// 裏の糸で確かめる1項目(項目, 条件の見せる名前, 範囲の表(名前, 開くパス))。条件の式は表ごとに、その表の印を
/// 全体の設定の条件に重ね直して引く(notify.columns-present)。
type Job = (Item, String, Vec<(String, Vec<PathBuf>)>);

/// 知らせの状態。
#[derive(Default)]
pub(crate) struct NotifyState {
    pub items: Vec<Item>,
    /// 記録の置き場(見た目の状態の置き場)。None なら確かめない(試験の既定。main が置く)。
    pub dir: Option<PathBuf>,
    /// 次の機会に確かめる(開いたとき・保存したあと)。
    pub want: bool,
    /// 最後に確かめ始めたとき。
    pub last: Option<Instant>,
    rx: Option<Receiver<Outcome>>,
    /// 下の帯に出した理由(同じ理由は2回出さない)。
    shown: HashSet<String>,
    /// 描いたあとに端末へ送るバイト。
    pub term: Vec<u8>,
    /// 試験で出す先を差し替える(届いた知らせを覚え、`fail` の出す先は失敗にする)。
    #[cfg(test)]
    pub test: Option<std::sync::Arc<std::sync::Mutex<TestOutlet>>>,
}

/// 試験の出す先。
#[cfg(test)]
#[derive(Default)]
pub(crate) struct TestOutlet {
    pub got: Vec<(notify::To, notify::Note)>,
    pub fail: Vec<notify::To>,
}

/// 項目ごとに範囲の表をファイルから読み、当たった行を選ぶ(`crate::notify_hits`)。
fn checks(jobs: Vec<Job>, global: &Arc<Conditions>, today: i64, now: i64) -> Vec<Check> {
    jobs.into_iter()
        .map(|(item, label, places)| {
            let tables = crate::notify_hits(&places, global, &item, &label, today, now);
            Check {
                item,
                label,
                tables,
            }
        })
        .collect()
}

impl App {
    /// 設定の `[[notify]]` を当てる(App::configure から)。開いたときに1回確かめる。
    pub(crate) fn configure_notify(&mut self, items: Vec<Item>) {
        self.notify.items = items;
        self.notify.want = true;
    }

    /// 確かめる時か: 項目と置き場があり、確かめている途中でなく、開いた・保存したあとか、前から間隔が過ぎた。
    pub(crate) fn notify_due(&self, at: Instant) -> bool {
        let n = &self.notify;
        !n.items.is_empty()
            && n.dir.is_some()
            && n.rx.is_none()
            && (n.want
                || n.last
                    .is_none_or(|l| at.duration_since(l) >= notify::INTERVAL))
    }

    /// 糸の結果を拾い、時が来ていれば確かめ始める(画面を回すたびに呼ぶ)。
    pub(crate) fn notify_poll(&mut self) {
        if let Some(rx) = &self.notify.rx {
            match rx.try_recv() {
                Ok(outcome) => {
                    self.notify.rx = None;
                    self.notify_done(outcome);
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => self.notify.rx = None,
            }
        }
        if self.notify_due(Instant::now()) {
            let (today, now) = mdgrid::clock::today_now(|k| std::env::var(k).ok());
            self.notify_start(today, now);
        }
    }

    /// 確かめ始める(今日 `today` と今 `now` で条件を評価する)。表を読むのも鳴らすのも裏の糸。
    pub(crate) fn notify_start(&mut self, today: i64, now: i64) {
        let Some(dir) = self.notify.dir.clone() else {
            return;
        };
        self.notify.want = false;
        self.notify.last = Some(Instant::now());
        let jobs = self.notify_jobs();
        let global = self.cond.global.clone();
        #[cfg(test)]
        let test = self.notify.test.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let checks = checks(jobs, &global, today, now);
            let mut term = Vec::new();
            #[cfg(test)]
            if let Some(t) = test {
                let report = notify::run(&dir, &checks, &mut |to, _item, note| {
                    let mut t = t.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                    t.got.push((to, note.clone()));
                    if t.fail.contains(&to) {
                        Err("test".into())
                    } else {
                        Ok(())
                    }
                });
                let _ = tx.send((report, term));
                return;
            }
            let out = notify::Outlets::current();
            let report = notify::run(&dir, &checks, &mut |to, item, note| {
                out.send(to, item, note, &mut |b| {
                    term.extend(b);
                    Ok(())
                })
            });
            let _ = tx.send((report, term));
        });
        self.notify.rx = Some(rx);
    }

    /// notify.cli: その場で1回確かめて鳴らす(`--notify`)。端末の合図は `terminal` に渡す。項目が無ければ
    /// 何もせず、記録の置き場が無ければ理由を返す。
    pub(crate) fn notify_once(
        &self,
        today: i64,
        now: i64,
        terminal: &mut dyn FnMut(Vec<u8>) -> Result<(), String>,
    ) -> Report {
        if self.notify.items.is_empty() {
            return Report::default();
        }
        let Some(dir) = self.notify.dir.clone() else {
            return Report {
                failures: vec![mdgrid::i18n::Msg::NotifyNoStateDir.text().to_string()],
                ..Report::default()
            };
        };
        let checks = checks(self.notify_jobs(), &self.cond.global, today, now);
        let out = notify::Outlets::current();
        notify::run(&dir, &checks, &mut |to, item, note| {
            out.send(to, item, note, terminal)
        })
    }

    /// 確かめる項目(見せる名前・範囲の表)。条件の式といつも隠す条件は、表ごとに、その表の印を全体の設定の条件に
    /// 重ねて引く(`crate::notify_hits`)。いつも隠す条件は、数字のキーで見せている間も当てる(画面の見せ方で知らせが
    /// 変わらないように。`--notify` と同じ行を見る)。
    fn notify_jobs(&self) -> Vec<Job> {
        self.notify
            .items
            .iter()
            .filter_map(|item| {
                let c = self.cond.all.get(&item.cond)?;
                let label = c.shown().to_string();
                Some((item.clone(), label, self.notify_places(&item.scope)))
            })
            .collect()
    }

    /// 確かめている途中か。
    #[cfg(test)]
    pub(crate) fn notify_busy(&self) -> bool {
        self.notify.rx.is_some()
    }

    /// 確かめの結果: 端末の合図をためて、届かなかった理由と記録を作り直した旨を下の帯に出す(同じ理由は1回)。
    fn notify_done(&mut self, (report, term): Outcome) {
        self.notify.term.extend(term);
        let fresh: Vec<String> = report
            .failures
            .into_iter()
            .chain(report.warnings)
            .filter(|f| self.notify.shown.insert(f.clone()))
            .collect();
        if !fresh.is_empty() {
            self.message = Some(fresh.join(mdgrid::i18n::Msg::SentenceSep.text()));
        }
    }

    /// 描いたあとに端末へ送るバイトを取り出す。
    pub(crate) fn notify_take_term(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.notify.term)
    }

    /// 項目の範囲の表(名前, 開くパス)。範囲の表が無ければ今の表。
    fn notify_places(&self, scope: &mdgrid::notice::Scope) -> Vec<(String, Vec<PathBuf>)> {
        use mdgrid::notice::Scope;
        let here = || {
            let paths = super::stack::target_paths(self);
            let name = super::stack::table_label(self);
            if paths.is_empty() {
                Vec::new()
            } else {
                vec![(name, paths)]
            }
        };
        let pick = |names: Option<&[String]>| -> Vec<(String, Vec<PathBuf>)> {
            self.scope_places()
                .into_iter()
                .filter(|p| names.is_none_or(|ns| ns.contains(&p.name)))
                .map(|p| (p.name, vec![p.path]))
                .collect()
        };
        match scope {
            Scope::Table => here(),
            Scope::Workspace => {
                let all = pick(None);
                if all.is_empty() {
                    here()
                } else {
                    all
                }
            }
            Scope::Tables(names) => pick(Some(names)),
        }
    }
}
