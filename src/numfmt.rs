//! 数の見せ方(`[format.<列>]`。cell-view.number-format)。桁の区切り・小数の桁・頭と末尾の字を付けた文字を作る
//! 純関数。見せ方だけで、ノートの値・並べ替え・式・書き出しには使わない(cell-view.number-format-display-only)。
//! 桁の区切りと小数点は画面の言語の習慣(文言の台帳の `NumSample`。cell-view.number-format-locale)。

use crate::i18n::Msg;

/// 小数の桁の上限(cell-view.number-format-invalid)。
pub const MAX_DECIMALS: u8 = 10;

/// 頭と末尾の字に使えない字か: 制御の文字と、文字の向きを変える書式の文字・行と段落の区切り・BOM
/// (端末の画面を乱せる。cell-view.number-format-invalid)。
pub fn unsafe_char(c: char) -> bool {
    c.is_control()
        || matches!(
            c,
            '\u{061C}'
                | '\u{200E}'
                | '\u{200F}'
                | '\u{202A}'..='\u{202E}'
                | '\u{2066}'..='\u{2069}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{FEFF}'
        )
}

/// 1つの範囲に書いた `[format.<列>]` の項目(書いた項目だけ)。狭い範囲が項目ごとに勝つ。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FormatLayer {
    pub thousands: Option<bool>,
    pub decimals: Option<u8>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
}

impl FormatLayer {
    /// 書いた項目を `to` に重ねる。
    pub fn apply(&self, to: &mut NumFormat) {
        if let Some(b) = self.thousands {
            to.thousands = b;
        }
        if let Some(d) = self.decimals {
            to.decimals = Some(d);
        }
        if let Some(p) = &self.prefix {
            to.prefix.clone_from(p);
        }
        if let Some(s) = &self.suffix {
            to.suffix.clone_from(s);
        }
    }

    pub fn is_empty(&self) -> bool {
        *self == FormatLayer::default()
    }
}

/// 範囲を重ねた列の数の見せ方。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NumFormat {
    /// 整数の部分を3桁ごとに区切る。
    pub thousands: bool,
    /// 小数の桁(足りなければ 0 で埋め、超える桁は四捨五入)。None は値の桁のまま。
    pub decimals: Option<u8>,
    /// 数の前に付ける字(負の数は `-` をこの前に置く)。
    pub prefix: String,
    /// 数の後ろに付ける字。
    pub suffix: String,
}

impl NumFormat {
    /// 全部の項目を書いた層(`--print-config --resolved` の出力)。
    pub fn to_layer(&self) -> FormatLayer {
        FormatLayer {
            thousands: Some(self.thousands),
            decimals: self.decimals,
            prefix: Some(self.prefix.clone()),
            suffix: Some(self.suffix.clone()),
        }
    }

    /// 整数を見せる文字。
    pub fn int(&self, i: i64) -> String {
        let digits = i.unsigned_abs().to_string();
        let frac = match self.decimals {
            Some(d) if d > 0 => "0".repeat(usize::from(d)),
            _ => String::new(),
        };
        self.join(i < 0, &digits, &frac)
    }

    /// 数を見せる文字。有限でない数と、指数でしか書けない大きな数は、値の字のまま(頭と末尾だけ付ける)。
    pub fn float(&self, f: f64) -> String {
        if !f.is_finite() {
            return format!("{}{f}{}", self.prefix, self.suffix);
        }
        let text = match self.decimals {
            Some(d) => {
                let d = usize::from(d);
                // 四捨五入(0.5 は 0 から遠い方へ)。`{:.d}` は2進の値で偶数へ丸めるので、先に桁をずらして丸める。
                let scale = 10f64.powi(i32::try_from(d).unwrap_or(0));
                let r = (f * scale).round() / scale;
                let r = if r.is_finite() { r } else { f };
                format!("{:.d$}", r.abs())
            }
            None if f.fract() == 0.0 && f.abs() < 1e15 => format!("{}", f.abs() as i64),
            None => format!("{}", f.abs()),
        };
        if text.contains(['e', 'E']) {
            return format!("{}{f}{}", self.prefix, self.suffix);
        }
        let (int, frac) = text.split_once('.').unwrap_or((&text, ""));
        // 丸めて 0 になった負の数(-0.04 を小数1桁)は `-` を付けない。
        let negative = f < 0.0 && (int.chars().chain(frac.chars()).any(|c| c != '0'));
        self.join(negative, int, frac)
    }

    fn join(&self, negative: bool, int: &str, frac: &str) -> String {
        let (group, point) = separators();
        let int = if self.thousands {
            group_digits(int, &group)
        } else {
            int.to_string()
        };
        let sign = if negative { "-" } else { "" };
        let frac = if frac.is_empty() {
            String::new()
        } else {
            format!("{point}{frac}")
        };
        format!("{sign}{}{int}{frac}{}", self.prefix, self.suffix)
    }
}

/// 整数の字の並びを3桁ごとに `sep` で区切る。
fn group_digits(digits: &str, sep: &str) -> String {
    let n = digits.chars().count();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (n - i).is_multiple_of(3) {
            out.push_str(sep);
        }
        out.push(c);
    }
    out
}

/// 今の言語の桁の区切りと小数点。台帳の見本 `NumSample`(`1,234.5` の形)の 1 と 2 の間が区切り、4 と 5 の間が
/// 小数点。読めなければ `,` と `.`。
pub fn separators() -> (String, String) {
    parse_sample(Msg::NumSample.text()).unwrap_or_else(|| (",".to_string(), ".".to_string()))
}

fn parse_sample(s: &str) -> Option<(String, String)> {
    let one = s.find('1')?;
    let two = s[one..].find('2')? + one;
    let four = s[two..].find('4')? + two;
    let five = s[four..].find('5')? + four;
    let group = s[one + 1..two].to_string();
    let point = s[four + 1..five].to_string();
    (!point.is_empty()).then_some((group, point))
}

#[cfg(test)]
#[path = "test_numfmt_unit.rs"]
mod tests;
