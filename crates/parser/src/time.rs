//! Lecture des horodatages, sans dépendance de date.
//!
//! Formats reconnus :
//! - ISO 8601 / RFC 3339 : `2026-11-16T00:00:21Z`, `…+02:00`, `…+0200`,
//!   séparateur `T` ou espace, fraction de seconde de 1 à 9 chiffres ;
//! - date seule : `2026-11-16` (précision : jour) ;
//! - époque Unix en secondes, millisecondes, microsecondes ou nanosecondes
//!   (nombre ou chaîne de chiffres), selon l'ordre de grandeur.
//!
//! La base de l'heure reflète la source : fuseau ou `Z` présent → `Explicit` ;
//! absent → `AssumedUtc` ; époque Unix → `Explicit` (UTC par définition).

use evidentyx_core::{EventTime, TimeBasis, TimePrecision};

/// Bornes de plausibilité : 1970-01-01 à 2100-01-01 (UTC).
const MIN_MICROS: i64 = 0;
const MAX_MICROS: i64 = 4_102_444_800_000_000;

/// Nombre de jours depuis 1970-01-01 pour une date du calendrier grégorien
/// (algorithme de Howard Hinnant, `days_from_civil`).
pub fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let m = i64::from(m);
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn days_in_month(y: i64, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap(y) => 29,
        2 => 28,
        _ => 0,
    }
}

fn within_bounds(t: EventTime) -> Option<EventTime> {
    (MIN_MICROS..MAX_MICROS)
        .contains(&t.utc_micros)
        .then_some(t)
}

fn digits(b: &[u8], from: usize, len: usize) -> Option<u32> {
    let slice = b.get(from..from + len)?;
    slice.iter().try_fold(0u32, |acc, c| {
        c.is_ascii_digit().then(|| acc * 10 + u32::from(c - b'0'))
    })
}

/// Lit un horodatage textuel ISO 8601 / RFC 3339 ou une date seule.
pub fn parse_iso(text: &str) -> Option<EventTime> {
    let b = text.trim().as_bytes();
    let year = i64::from(digits(b, 0, 4)?);
    if b.get(4) != Some(&b'-') || b.get(7) != Some(&b'-') {
        return None;
    }
    let month = digits(b, 5, 2)?;
    let day = digits(b, 8, 2)?;
    if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
        return None;
    }
    let days = days_from_civil(year, month, day);
    if b.len() == 10 {
        return within_bounds(EventTime {
            utc_micros: days * 86_400_000_000,
            precision: TimePrecision::Day,
            basis: TimeBasis::AssumedUtc,
        });
    }
    if !matches!(b.get(10), Some(b'T' | b't' | b' '))
        || b.get(13) != Some(&b':')
        || b.get(16) != Some(&b':')
    {
        return None;
    }
    let (h, mi, s) = (digits(b, 11, 2)?, digits(b, 14, 2)?, digits(b, 17, 2)?);
    if h > 23 || mi > 59 || s > 60 {
        return None;
    }
    let mut i = 19;
    let mut micros: i64 = 0;
    let mut precision = TimePrecision::Second;
    if matches!(b.get(i), Some(b'.' | b',')) {
        i += 1;
        let start = i;
        while b.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        let n = i - start;
        if n == 0 {
            return None;
        }
        let frac = &b[start..start + n.min(6)];
        let mut value: i64 = frac.iter().fold(0, |acc, c| acc * 10 + i64::from(c - b'0'));
        for _ in n.min(6)..6 {
            value *= 10;
        }
        micros = value;
        precision = if n <= 3 {
            TimePrecision::Millisecond
        } else {
            TimePrecision::Microsecond
        };
    }
    let (offset_secs, basis) = match b.get(i) {
        None => (0, TimeBasis::AssumedUtc),
        Some(b'Z' | b'z') if i + 1 == b.len() => (0, TimeBasis::Explicit),
        Some(sign @ (b'+' | b'-')) => {
            let oh = i64::from(digits(b, i + 1, 2)?);
            let rest = &b[i + 3..];
            let om = match rest {
                [] => 0,
                [b':', m1, m2] | [m1, m2] if m1.is_ascii_digit() && m2.is_ascii_digit() => {
                    i64::from((m1 - b'0') * 10 + (m2 - b'0'))
                }
                _ => return None,
            };
            if oh > 14 || om > 59 {
                return None;
            }
            let off = (oh * 3600 + om * 60) * if *sign == b'+' { 1 } else { -1 };
            (off, TimeBasis::Explicit)
        }
        _ => return None,
    };
    let secs = days * 86_400 + i64::from(h) * 3600 + i64::from(mi) * 60 + i64::from(s.min(59))
        - offset_secs;
    within_bounds(EventTime {
        utc_micros: secs * 1_000_000 + micros,
        precision,
        basis,
    })
}

/// Interprète un nombre comme une époque Unix, d'après son ordre de grandeur
/// (secondes, millisecondes, microsecondes, nanosecondes).
pub fn parse_epoch(value: f64) -> Option<EventTime> {
    if !value.is_finite() || value <= 0.0 {
        return None;
    }
    let (micros, precision) = if value < 1e11 {
        let fractional = value.fract() != 0.0;
        (
            value * 1e6,
            if fractional {
                TimePrecision::Millisecond
            } else {
                TimePrecision::Second
            },
        )
    } else if value < 1e14 {
        (value * 1e3, TimePrecision::Millisecond)
    } else if value < 1e17 {
        (value, TimePrecision::Microsecond)
    } else {
        (value / 1e3, TimePrecision::Microsecond)
    };
    within_bounds(EventTime {
        utc_micros: micros.round() as i64,
        precision,
        basis: TimeBasis::Explicit,
    })
}

/// Lit une valeur textuelle : ISO 8601 d'abord, sinon une époque en chiffres.
pub fn parse_text(text: &str) -> Option<EventTime> {
    let t = text.trim();
    parse_iso(t).or_else(|| {
        (!t.is_empty() && t.bytes().all(|c| c.is_ascii_digit() || c == b'.'))
            .then(|| t.parse::<f64>().ok())
            .flatten()
            .and_then(parse_epoch)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const T0: i64 = 1_763_251_221_000_000; // 2025-11-16T00:00:21Z

    #[test]
    fn jours_depuis_l_epoque() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(2000, 3, 1), 11_017);
        assert_eq!(days_from_civil(2024, 2, 29), 19_782);
    }

    #[test]
    fn iso_avec_z_et_decalages() {
        let t = parse_iso("2025-11-16T00:00:21Z").unwrap();
        assert_eq!(
            (t.utc_micros, t.precision, t.basis),
            (T0, TimePrecision::Second, TimeBasis::Explicit)
        );
        assert_eq!(
            parse_iso("2025-11-16T02:00:21+02:00").unwrap().utc_micros,
            T0
        );
        assert_eq!(
            parse_iso("2025-11-15T19:00:21-0500").unwrap().utc_micros,
            T0
        );
        assert_eq!(
            parse_iso("2025-11-16 00:00:21").unwrap().basis,
            TimeBasis::AssumedUtc
        );
    }

    #[test]
    fn fractions_et_precision() {
        let ms = parse_iso("2025-11-16T00:00:21.123Z").unwrap();
        assert_eq!(
            (ms.utc_micros, ms.precision),
            (T0 + 123_000, TimePrecision::Millisecond)
        );
        let us = parse_iso("2025-11-16T00:00:21.123456789Z").unwrap();
        assert_eq!(
            (us.utc_micros, us.precision),
            (T0 + 123_456, TimePrecision::Microsecond)
        );
    }

    #[test]
    fn date_seule() {
        let d = parse_iso("2024-02-29").unwrap();
        assert_eq!(d.precision, TimePrecision::Day);
        assert!(parse_iso("2023-02-29").is_none(), "pas d'année bissextile");
    }

    #[test]
    fn rejets() {
        for bad in [
            "",
            "hier",
            "2025-13-01",
            "2025-11-16T25:00:00Z",
            "2025-11-16T00:00:21+99:00",
            "16/11/2025",
            "1960-01-01",
            "2025-11-16T00:00:21Zx",
        ] {
            assert!(parse_iso(bad).is_none(), "{bad}");
        }
    }

    #[test]
    fn epoques() {
        assert_eq!(parse_epoch(1_763_251_221.0).unwrap().utc_micros, T0);
        assert_eq!(
            parse_epoch(1_763_251_221_000.0).unwrap().precision,
            TimePrecision::Millisecond
        );
        assert_eq!(parse_epoch(1_763_251_221_000_000.0).unwrap().utc_micros, T0);
        assert_eq!(
            parse_epoch(1_763_251_221_000_000_000.0).unwrap().utc_micros,
            T0
        );
        assert!(parse_epoch(42.0).is_some()); // 1970, plausible au sens strict
        assert!(parse_epoch(-1.0).is_none());
        assert_eq!(parse_text("1763251221").unwrap().utc_micros, T0);
    }
}
