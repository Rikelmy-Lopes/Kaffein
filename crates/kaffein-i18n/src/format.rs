//! Culture-specific number and date formatting.
//!
//! The formats come from each catalog's `meta` block, so a new locale does not
//! need code changes to display numbers or dates in its own convention.

use crate::catalog::CatalogMeta;

/// A calendar date without a time zone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Date {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl Date {
    pub fn new(year: i32, month: u32, day: u32) -> Self {
        Self { year, month, day }
    }

    /// Converts Unix seconds to a UTC date.
    pub fn from_unix_seconds(seconds: i64) -> Self {
        let days = seconds.div_euclid(86_400);
        civil_from_days(days)
    }
}

pub(super) fn number(meta: &CatalogMeta, value: i64) -> String {
    let mut text = String::new();
    if value < 0 {
        text.push('-');
    }
    text.push_str(&group_digits(
        &value.unsigned_abs().to_string(),
        meta.group_separator(),
    ));
    text
}

pub(super) fn decimal(meta: &CatalogMeta, value: f64, fraction_digits: usize) -> String {
    if !value.is_finite() {
        return value.to_string();
    }

    let mut text = String::new();
    if value.is_sign_negative() {
        text.push('-');
    }
    let formatted = format!("{:.*}", fraction_digits, value.abs());
    let (integer, fraction) = formatted.split_once('.').unwrap_or((&formatted, ""));
    text.push_str(&group_digits(integer, meta.group_separator()));
    if fraction_digits > 0 {
        text.push(meta.decimal_separator());
        text.push_str(fraction);
    }
    text
}

pub(super) fn date(meta: &CatalogMeta, date: Date) -> String {
    meta.date_pattern()
        .replace("YYYY", &format!("{:04}", date.year))
        .replace("YY", &format!("{:02}", date.year.rem_euclid(100)))
        .replace("MM", &format!("{:02}", date.month))
        .replace("DD", &format!("{:02}", date.day))
}

fn group_digits(digits: &str, separator: char) -> String {
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            grouped.push(separator);
        }
        grouped.push(digit);
    }
    grouped
}

/// Howard Hinnant's `civil_from_days`, with day zero at 1970-01-01.
fn civil_from_days(days: i64) -> Date {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + i64::from(month <= 2);

    Date::new(year as i32, month as u32, day as u32)
}
