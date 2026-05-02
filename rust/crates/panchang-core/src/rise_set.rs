use crate::ephemeris;

fn alt_rel_horizon(jd: f64, lat: f64, lon: f64) -> f64 {
    ephemeris::sun_altitude_deg(jd, lat, lon) - (-0.833)
}

fn bracket_next(jd0: f64, lat: f64, lon: f64, rising: bool, max_days: f64) -> Option<(f64, f64)> {
    let step = 5.0 / 1440.0;
    let mut prev_jd = jd0;
    let mut prev = alt_rel_horizon(prev_jd, lat, lon);
    let mut jd = jd0;
    while jd < jd0 + max_days {
        jd += step;
        let cur = alt_rel_horizon(jd, lat, lon);
        if rising {
            if prev <= 0.0 && cur > 0.0 {
                return Some((prev_jd, jd));
            }
        } else if prev >= 0.0 && cur < 0.0 {
            return Some((prev_jd, jd));
        }
        prev = cur;
        prev_jd = jd;
    }
    None
}

fn bisect_alt(mut lo: f64, mut hi: f64, lat: f64, lon: f64, rising: bool) -> f64 {
    for _ in 0..40 {
        let mid = 0.5 * (lo + hi);
        let v = alt_rel_horizon(mid, lat, lon);
        if rising {
            if v > 0.0 {
                hi = mid;
            } else {
                lo = mid;
            }
        } else if v < 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    0.5 * (lo + hi)
}

pub fn find_next_sunrise_jd(jd0: f64, lat: f64, lon: f64) -> Option<f64> {
    let step = 5.0 / 1440.0;
    bracket_next(jd0 + step, lat, lon, true, 2.0).map(|(lo, hi)| bisect_alt(lo, hi, lat, lon, true))
}

pub fn find_next_sunset_jd(jd0: f64, lat: f64, lon: f64) -> Option<f64> {
    let step = 5.0 / 1440.0;
    bracket_next(jd0 + step, lat, lon, false, 2.0)
        .map(|(lo, hi)| bisect_alt(lo, hi, lat, lon, false))
}

pub fn find_prev_sunrise_jd(jd0: f64, lat: f64, lon: f64) -> Option<f64> {
    let step = 5.0 / 1440.0;
    let mut jd = jd0;
    let mut prev = alt_rel_horizon(jd, lat, lon);
    let mut prev_jd = jd;
    while jd > jd0 - 2.0 {
        jd -= step;
        let cur = alt_rel_horizon(jd, lat, lon);
        if cur < 0.0 && prev >= 0.0 {
            return Some(bisect_alt(jd, prev_jd, lat, lon, true));
        }
        prev = cur;
        prev_jd = jd;
    }
    None
}
