//! Per-locale number, currency and date formatting.
//!
//! Implemented in pure Rust (no heavy CLDR dependency) using the locale's
//! decimal/group separators and month/day name tables.

#![allow(dead_code)]

use crate::i18n::locales::meta_for;

const EN_MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const ES_MONTHS: [&str; 12] = [
    "enero",
    "febrero",
    "marzo",
    "abril",
    "mayo",
    "junio",
    "julio",
    "agosto",
    "septiembre",
    "octubre",
    "noviembre",
    "diciembre",
];
const PT_MONTHS: [&str; 12] = [
    "janeiro",
    "fevereiro",
    "março",
    "abril",
    "maio",
    "junho",
    "julho",
    "agosto",
    "setembro",
    "outubro",
    "novembro",
    "dezembro",
];
const AR_MONTHS: [&str; 12] = [
    "يناير",
    "فبراير",
    "مارس",
    "أبريل",
    "مايو",
    "يونيو",
    "يوليو",
    "أغسطس",
    "سبتمبر",
    "أكتوبر",
    "نوفمبر",
    "ديسمبر",
];
const FR_MONTHS: [&str; 12] = [
    "janvier",
    "février",
    "mars",
    "avril",
    "mai",
    "juin",
    "juillet",
    "août",
    "septembre",
    "octobre",
    "novembre",
    "décembre",
];

fn months_for(locale: &str) -> &'static [&'static str; 12] {
    match locale {
        "es" => &ES_MONTHS,
        "pt" => &PT_MONTHS,
        "ar" => &AR_MONTHS,
        "fr" => &FR_MONTHS,
        _ => &EN_MONTHS,
    }
}

/// Formats an integer with the locale's thousands grouping.
pub fn number(locale: &str, value: i64) -> String {
    let sep = meta_for(locale).map(|m| m.group_sep).unwrap_or(',');
    let neg = value < 0;
    let digits = value.saturating_abs().to_string();
    let mut out = String::new();
    let dv = digits.as_bytes();
    for (i, b) in dv.iter().enumerate() {
        if i > 0 && (dv.len() - i).is_multiple_of(3) {
            out.push(sep);
        }
        out.push(*b as char);
    }
    if neg {
        out.insert(0, '-');
    }
    out
}

/// Formats a decimal value (count of minor units) with the locale's separators.
pub fn decimal(locale: &str, value: f64) -> String {
    let meta = meta_for(locale)
        .map(|m| (m.decimal_sep, m.group_sep))
        .unwrap_or(('.', ','));
    let mut s = format!("{value:.2}");
    // Replace '.' with the locale decimal separator.
    let sep = meta.0;
    if sep != '.' {
        s = s.replace('.', &sep.to_string());
    }
    s
}

/// Formats an amount of "coins" (integer minor units) as a currency string.
pub fn currency(locale: &str, value: i64) -> String {
    let meta = meta_for(locale).unwrap_or_else(|| crate::i18n::locales::LocaleMeta {
        code: "en".into(),
        name: "English".into(),
        native: "English".into(),
        dir: "ltr",
        rtl: false,
        decimal_sep: '.',
        group_sep: ',',
        currency: "$",
    });
    let grouped = number(locale, value);
    let sym = meta.currency;
    // Arabic currency symbol precedes; others follow.
    if meta.rtl {
        format!("{sym} {grouped}")
    } else {
        format!("{grouped} {sym}")
    }
}

/// Formats a UNIX timestamp as a date in the locale's language.
pub fn date(locale: &str, unix_secs: i64) -> String {
    let months = months_for(locale);
    let (year, month, day) = civil_from_unix(unix_secs);
    let month = month as usize;
    match locale {
        "es" => format!("{day} de {} de {year}", months[month - 1]),
        "fr" => format!("{day} {} {year}", months[month - 1]),
        "pt" => format!("{day} de {} de {year}", months[month - 1]),
        "ar" => format!("{day} {} {year}", months[month - 1]),
        _ => format!("{} {day}, {year}", months[month - 1]),
    }
}

/// Converts a UNIX timestamp (seconds) to a civil (year, month, day) date,
/// computed in UTC using Howard Hinnant's civil date algorithm.
fn civil_from_unix(unix_secs: i64) -> (i64, u32, u32) {
    let days = unix_secs.div_euclid(86_400);
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}
