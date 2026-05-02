use chrono_tz::Tz;

use crate::names;
use crate::time;
use crate::types::HoraInterval;

fn weekday_at_jd(jd: f64, tz: Tz) -> usize {
    use chrono::Datelike;
    time::datetime_utc_from_jd(jd)
        .with_timezone(&tz)
        .weekday()
        .num_days_from_monday() as usize
}

fn first_ruler_for_weekday(weekday: usize) -> &'static str {
    match weekday {
        0 => "Moon",
        1 => "Mars",
        2 => "Mercury",
        3 => "Jupiter",
        4 => "Venus",
        5 => "Saturn",
        _ => "Sun",
    }
}

fn sequence_start(first: &str) -> usize {
    names::PLANET_HORA_SEQUENCE
        .iter()
        .position(|x| *x == first)
        .unwrap_or(0)
}

pub fn build_hora_table(sr: f64, ss: f64, next_sr: f64, tz: Tz) -> Vec<HoraInterval> {
    let first = first_ruler_for_weekday(weekday_at_jd(sr, tz));
    let mut seq = sequence_start(first);
    let day_len = (ss - sr) / 12.0;
    let night_len = (next_sr - ss) / 12.0;
    let mut out = Vec::with_capacity(24);
    for i in 0..12 {
        let start = sr + day_len * i as f64;
        let end = start + day_len;
        out.push(HoraInterval {
            index: i,
            ruler: names::PLANET_HORA_SEQUENCE[seq % 7].to_string(),
            is_daytime: true,
            jd_start: start,
            jd_end: end,
            start_local: time::local_iso_from_jd(start, tz),
            end_local: time::local_iso_from_jd(end, tz),
        });
        seq += 1;
    }
    for i in 0..12 {
        let start = ss + night_len * i as f64;
        let end = start + night_len;
        out.push(HoraInterval {
            index: i + 12,
            ruler: names::PLANET_HORA_SEQUENCE[seq % 7].to_string(),
            is_daytime: false,
            jd_start: start,
            jd_end: end,
            start_local: time::local_iso_from_jd(start, tz),
            end_local: time::local_iso_from_jd(end, tz),
        });
        seq += 1;
    }
    out
}
