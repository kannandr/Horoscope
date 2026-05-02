use crate::angas;
use crate::ayanamsha;
use crate::ephemeris;
use crate::types::{AyanamshaId, EngineId};

fn sidereal(jd: f64, ay: AyanamshaId, engine: EngineId) -> (f64, f64) {
    let trop = ephemeris::apparent_tropical_longitudes(jd, engine);
    let d = ayanamsha::delta_deg(jd, ay);
    (ephemeris::reduce_deg(trop.sun_deg - d), ephemeris::reduce_deg(trop.moon_lon_deg - d))
}

fn elongation(jd: f64, ay: AyanamshaId, engine: EngineId) -> f64 {
    let (s, m) = sidereal(jd, ay, engine);
    angas::elongation_deg(s, m)
}

fn moon_lon(jd: f64, ay: AyanamshaId, engine: EngineId) -> f64 {
    sidereal(jd, ay, engine).1
}

fn scalar_forward<F>(jd0: f64, mut target: f64, max_days: f64, step_days: f64, f: F) -> Option<f64>
where
    F: Fn(f64) -> f64,
{
    let mut prev = f(jd0);
    let mut prev_jd = jd0;
    let mut jd = jd0;
    let tgt = if target >= 360.0 - 1e-9 {
        360.0
    } else {
        target %= 360.0;
        target
    };
    while jd < jd0 + max_days {
        jd += step_days;
        let cur = f(jd);
        if tgt >= 360.0 - 1e-9 {
            if prev > cur + 100.0 {
                return Some(bisect_wrap(prev_jd, jd, &f));
            }
        } else if prev < cur && prev < tgt && tgt <= cur {
            return Some(bisect_scalar(prev_jd, jd, tgt, &f));
        } else if prev >= cur && prev < tgt && tgt <= cur + 360.0 {
            return Some(bisect_scalar(prev_jd, jd, tgt, &f));
        }
        prev = cur;
        prev_jd = jd;
    }
    None
}

fn scalar_backward<F>(jd0: f64, mut target: f64, max_days: f64, step_days: f64, f: F) -> Option<f64>
where
    F: Fn(f64) -> f64,
{
    if target < 1e-9 {
        target = 0.0;
    }
    let mut prev = f(jd0);
    let mut prev_jd = jd0;
    let mut jd = jd0;
    while jd > jd0 - max_days {
        jd -= step_days;
        let cur = f(jd);
        if target == 0.0 {
            if cur > 300.0 && prev < 60.0 {
                return Some(bisect_wrap(jd, prev_jd, &f));
            }
        } else if cur < target && target <= prev {
            return Some(bisect_scalar(jd, prev_jd, target, &f));
        }
        prev = cur;
        prev_jd = jd;
    }
    None
}

fn bisect_scalar<F>(mut lo: f64, mut hi: f64, target: f64, f: &F) -> f64
where
    F: Fn(f64) -> f64,
{
    for _ in 0..48 {
        let mid = 0.5 * (lo + hi);
        if f(mid) < target { lo = mid; } else { hi = mid; }
    }
    0.5 * (lo + hi)
}

fn bisect_wrap<F>(mut lo: f64, mut hi: f64, f: &F) -> f64
where
    F: Fn(f64) -> f64,
{
    for _ in 0..48 {
        let mid = 0.5 * (lo + hi);
        let v = f(mid);
        if v < 3.0 || v > f(lo) { hi = mid; } else { lo = mid; }
    }
    0.5 * (lo + hi)
}

pub fn next_tithi_end_jd(jd0: f64, ay: AyanamshaId, engine: EngineId) -> Option<f64> {
    let e0 = elongation(jd0, ay, engine);
    let target = (((e0 / 12.0).floor() + 1.0) * 12.0) % 360.0;
    scalar_forward(jd0, target, 2.0, 1.0 / 1440.0, |jd| elongation(jd, ay, engine))
}

pub fn prev_tithi_start_jd(jd0: f64, ay: AyanamshaId, engine: EngineId) -> Option<f64> {
    let e0 = elongation(jd0, ay, engine);
    let target = ((e0 / 12.0).floor() * 12.0) % 360.0;
    scalar_backward(jd0, target, 3.0, 1.0 / 1440.0, |jd| elongation(jd, ay, engine))
}

pub fn next_karana_end_jd(jd0: f64, ay: AyanamshaId, engine: EngineId) -> Option<f64> {
    let e0 = elongation(jd0, ay, engine);
    let target = (((e0 / 6.0).floor() + 1.0) * 6.0) % 360.0;
    scalar_forward(jd0, target, 1.2, 1.0 / 1440.0, |jd| elongation(jd, ay, engine))
}

pub fn prev_karana_start_jd(jd0: f64, ay: AyanamshaId, engine: EngineId) -> Option<f64> {
    let e0 = elongation(jd0, ay, engine);
    let target = ((e0 / 6.0).floor() * 6.0) % 360.0;
    scalar_backward(jd0, target, 2.0, 1.0 / 1440.0, |jd| elongation(jd, ay, engine))
}

pub fn next_nakshatra_end_jd(jd0: f64, ay: AyanamshaId, engine: EngineId) -> Option<f64> {
    let span = 360.0 / 27.0;
    let m = moon_lon(jd0, ay, engine);
    let target = ((m / span).floor() + 1.0) * span;
    scalar_forward(jd0, target, 2.0, 1.0 / 1440.0, |jd| moon_lon(jd, ay, engine))
}

pub fn prev_nakshatra_start_jd(jd0: f64, ay: AyanamshaId, engine: EngineId) -> Option<f64> {
    let span = 360.0 / 27.0;
    let m = moon_lon(jd0, ay, engine);
    let target = ((m / span).floor() * span) % 360.0;
    scalar_backward(jd0, target, 35.0, 1.0 / 1440.0, |jd| moon_lon(jd, ay, engine))
}

pub fn next_yoga_end_jd(jd0: f64, ay: AyanamshaId, engine: EngineId) -> Option<f64> {
    let span = 360.0 / 27.0;
    let (s, m) = sidereal(jd0, ay, engine);
    let y = (s + m) % 360.0;
    let target = ((y / span).floor() + 1.0) * span;
    scalar_forward(jd0, target, 2.0, 1.0 / 1440.0, |jd| {
        let (ss, mm) = sidereal(jd, ay, engine);
        (ss + mm) % 360.0
    })
}
