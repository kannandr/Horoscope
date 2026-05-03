use std::f64::consts::PI;

use crate::meeus_tables::{
    NUTATION_ARG_TABLE, NUTATION_COSINE_COEF_TABLE, NUTATION_SINE_COEF_TABLE,
    PERIODIC_TERMS_B_TABLE, PERIODIC_TERMS_LR_TABLE,
};
use crate::types::EngineId;

#[derive(Debug, Clone, Copy)]
pub struct TropicalLongitudes {
    pub sun_deg: f64,
    pub moon_lon_deg: f64,
    pub moon_lat_deg: f64,
}

pub fn reduce_deg(x: f64) -> f64 {
    let y = x % 360.0;
    if y < 0.0 {
        y + 360.0
    } else {
        y
    }
}

fn radians(x: f64) -> f64 {
    x * PI / 180.0
}
fn degrees(x: f64) -> f64 {
    x * 180.0 / PI
}
pub(crate) fn centuries_j2000(jd: f64) -> f64 {
    (jd - 2451545.0) / 36525.0
}

pub fn sun_apparent_longitude_deg(jd: f64) -> f64 {
    let t = centuries_j2000(jd);
    let l0 = reduce_deg(280.46646 + t * (36000.76983 + t * 0.0003032));
    let m = reduce_deg(357.52911 + t * (35999.05029 - t * 0.0001537));
    let mr = radians(m);
    let c = (1.914602 - t * (0.004817 + t * 0.000014)) * mr.sin()
        + (0.019993 - t * 0.000101) * (2.0 * mr).sin()
        + 0.000289 * (3.0 * mr).sin();
    let true_lon = reduce_deg(l0 + c);
    let om = reduce_deg(125.04 - 1934.136 * t);
    reduce_deg(true_lon - 0.00569 - 0.00478 * radians(om).sin())
}

pub fn mean_sun_geometric_longitude_deg(jd: f64) -> f64 {
    let t = centuries_j2000(jd);
    reduce_deg(280.46646 + t * (36000.76983 + t * 0.0003032))
}

pub fn mean_moon_longitude_deg(jd: f64) -> f64 {
    let t = centuries_j2000(jd);
    reduce_deg(
        218.3164477
            + (481267.88123421 + (-0.0015786 + (1.0 / 538841.0 - t / 65194000.0) * t) * t) * t,
    )
}

fn moon_mean_elements(jd: f64) -> (f64, f64, f64, f64, f64, f64) {
    let t = centuries_j2000(jd);
    let lp = mean_moon_longitude_deg(jd);
    let d = reduce_deg(
        297.8501921
            + (445267.1114034 + (-0.0018819 + (1.0 / 545868.0 - t / 113065000.0) * t) * t) * t,
    );
    let m_sun = reduce_deg(357.5291092 + (35999.0502909 + (-0.0001536 + t / 24490000.0) * t) * t);
    let mp = reduce_deg(
        134.9633964 + (477198.8675055 + (0.0087414 + (1.0 / 69699.9 + t / 14712000.0) * t) * t) * t,
    );
    let f = reduce_deg(
        93.2720950
            + (483202.0175233 + (-0.0036539 + (-1.0 / 3526000.0 + t / 863310000.0) * t) * t) * t,
    );
    let e = 1.0 + (-0.002516 - 0.0000074 * t) * t;
    (lp, d, m_sun, mp, f, e)
}

pub fn moon_geocentric_ecliptic_deg(jd: f64) -> (f64, f64) {
    let (lp, d, m_sun, mp, f, e) = moon_mean_elements(jd);
    let dr = radians(d);
    let mr = radians(m_sun);
    let mpr = radians(mp);
    let fr = radians(f);
    let lpr = radians(lp);
    let t = centuries_j2000(jd);
    let a1 = radians(reduce_deg(119.75 + 131.849 * t));
    let a2 = radians(reduce_deg(53.09 + 479264.290 * t));
    let a3 = radians(reduce_deg(313.45 + 481266.484 * t));
    let e2 = e * e;

    let mut sigmal = 0.0;
    for row in PERIODIC_TERMS_LR_TABLE {
        let d_d = row[0];
        let d_m = row[1];
        let d_mp = row[2];
        let d_f = row[3];
        let mut coeff_l = row[4];
        let arg = d_d * dr + d_m * mr + d_mp * mpr + d_f * fr;
        if d_m.abs() == 1.0 {
            coeff_l *= e;
        } else if d_m.abs() == 2.0 {
            coeff_l *= e2;
        }
        sigmal += coeff_l * arg.sin();
    }
    sigmal += 3958.0 * a1.sin() + 1962.0 * (lpr - fr).sin() + 318.0 * a2.sin();

    let mut sigmab = 0.0;
    for row in PERIODIC_TERMS_B_TABLE {
        let d_d = row[0];
        let d_m = row[1];
        let d_mp = row[2];
        let d_f = row[3];
        let mut coeff_b = row[4];
        let arg = d_d * dr + d_m * mr + d_mp * mpr + d_f * fr;
        if d_m.abs() == 1.0 {
            coeff_b *= e;
        } else if d_m.abs() == 2.0 {
            coeff_b *= e2;
        }
        sigmab += coeff_b * arg.sin();
    }
    sigmab += -2235.0 * lpr.sin()
        + 382.0 * a3.sin()
        + 175.0 * (a1 - fr).sin()
        + 175.0 * (a1 + fr).sin()
        + 127.0 * (lpr - mpr).sin()
        - 115.0 * (lpr + mpr).sin();

    (reduce_deg(lp + sigmal / 1_000_000.0), sigmab / 1_000_000.0)
}

fn nutation_arguments(jd: f64) -> [f64; 5] {
    let t = centuries_j2000(jd);
    [
        radians(reduce_deg(
            297.85036 + t * (445267.111480 + t * (-0.0019142 + t / 189474.0)),
        )),
        radians(reduce_deg(
            357.52772 + t * (35999.050340 + t * (-0.0001603 - t / 300000.0)),
        )),
        radians(reduce_deg(
            134.96298 + t * (477198.867398 + t * (0.0086972 + t / 56250.0)),
        )),
        radians(reduce_deg(
            93.27191 + t * (483202.017538 + t * (-0.0036825 + t / 327270.0)),
        )),
        radians(reduce_deg(
            125.04452 + t * (-1934.136261 + t * (0.0020708 + t / 450000.0)),
        )),
    ]
}

pub fn nutation_longitude_arcsec(jd: f64) -> f64 {
    let t = centuries_j2000(jd);
    let args = nutation_arguments(jd);
    let mut dpsi = 0.0;
    for (i, row) in NUTATION_ARG_TABLE.iter().enumerate() {
        let mut arg = 0.0;
        for j in 0..5 {
            arg += row[j] * args[j];
        }
        let coeff = NUTATION_SINE_COEF_TABLE[i][0] + NUTATION_SINE_COEF_TABLE[i][1] * t;
        dpsi += (coeff * arg.sin()) / 10000.0;
    }
    dpsi
}

pub fn nutation_obliquity_arcsec(jd: f64) -> f64 {
    let t = centuries_j2000(jd);
    let args = nutation_arguments(jd);
    let mut deps = 0.0;
    for (i, row) in NUTATION_ARG_TABLE.iter().enumerate() {
        let mut arg = 0.0;
        for j in 0..5 {
            arg += row[j] * args[j];
        }
        let coeff_row = NUTATION_COSINE_COEF_TABLE
            .get(i)
            .copied()
            .unwrap_or([0.0, 0.0]);
        let coeff = coeff_row[0] + coeff_row[1] * t;
        deps += (coeff * arg.cos()) / 10000.0;
    }
    deps
}

pub fn moon_apparent_longitude_deg(jd: f64) -> (f64, f64) {
    let (lon, lat) = moon_geocentric_ecliptic_deg(jd);
    let dpsi_deg = nutation_longitude_arcsec(jd) / 3600.0;
    (reduce_deg(lon + dpsi_deg), lat)
}

pub fn apparent_tropical_longitudes(jd: f64, engine: EngineId) -> TropicalLongitudes {
    match engine {
        EngineId::Meeus => {
            let (moon_lon_deg, moon_lat_deg) = moon_apparent_longitude_deg(jd);
            TropicalLongitudes {
                sun_deg: sun_apparent_longitude_deg(jd),
                moon_lon_deg,
                moon_lat_deg,
            }
        }
        EngineId::SuryaMean => TropicalLongitudes {
            sun_deg: mean_sun_geometric_longitude_deg(jd),
            moon_lon_deg: mean_moon_longitude_deg(jd),
            moon_lat_deg: 0.0,
        },
    }
}

fn mean_obliquity_deg(jd: f64) -> f64 {
    let u = (jd - 2451545.0) / 3652500.0;
    let delta_arcsec = u
        * (-4680.93
            + u * (-1.55
                + u * (1999.25
                    + u * (-51.38
                        + u * (-249.67
                            + u * (-39.05 + u * (7.12 + u * (27.87 + u * (5.79 + u * 2.45)))))))));
    23.0 + 26.0 / 60.0 + 21.448 / 3600.0 + delta_arcsec / 3600.0
}

fn true_obliquity_deg(jd: f64) -> f64 {
    mean_obliquity_deg(jd) + nutation_obliquity_arcsec(jd) / 3600.0
}

fn gmst_deg(jd: f64) -> f64 {
    let t = centuries_j2000(jd);
    reduce_deg(
        280.46061837 + 360.98564736629 * (jd - 2451545.0) + 0.000387933 * t * t
            - (t * t * t) / 38710000.0,
    )
}

/// Tropical ecliptic longitude of the ascendant (intersection of the eastern
/// horizon with the ecliptic), degrees \[0, 360).
///
/// Uses GMST and true obliquity consistent with [`sun_altitude_deg`]. Formula:
/// `atan2(sin(RAMC), cos(RAMC)*cos(ε) + tan(φ)*sin(ε))` with RAMC = GMST + λ_east.
pub fn ascendant_tropical_deg(jd: f64, lat_deg: f64, lon_deg: f64) -> f64 {
    let eps = radians(true_obliquity_deg(jd));
    let phi = radians(lat_deg);
    let theta = radians(reduce_deg(gmst_deg(jd) + lon_deg));
    let y = theta.sin();
    let x = theta.cos() * eps.cos() + phi.tan() * eps.sin();
    reduce_deg(degrees(y.atan2(x)))
}

pub fn sun_altitude_deg(jd: f64, lat_deg: f64, lon_deg: f64) -> f64 {
    let lam = radians(sun_apparent_longitude_deg(jd));
    let eps = radians(true_obliquity_deg(jd));
    let sin_dec = eps.sin() * lam.sin();
    let dec = sin_dec.clamp(-1.0, 1.0).asin();
    let y = lam.sin() * eps.cos();
    let x = lam.cos();
    let ra = reduce_deg(degrees(y.atan2(x)));
    let lst = reduce_deg(gmst_deg(jd) + lon_deg);
    let h = radians(reduce_deg(lst - ra));
    let latr = radians(lat_deg);
    degrees(
        (latr.sin() * dec.sin() + latr.cos() * dec.cos() * h.cos())
            .clamp(-1.0, 1.0)
            .asin(),
    )
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::{ascendant_tropical_deg, moon_apparent_longitude_deg};
    use crate::time::julian_day_ut;

    #[test]
    fn ascendant_tropical_advances_with_ut() {
        let jd0 = julian_day_ut(
            Utc.with_ymd_and_hms(1990, 8, 15, 9, 2, 0)
                .single()
                .unwrap(),
        );
        let jd1 = jd0 + 1.0 / 24.0;
        let a0 = ascendant_tropical_deg(jd0, 13.0827, 80.2707);
        let a1 = ascendant_tropical_deg(jd1, 13.0827, 80.2707);
        assert!(a0.is_finite() && a1.is_finite());
        let mut d = (a1 - a0).abs();
        if d > 180.0 {
            d = 360.0 - d;
        }
        assert!(
            d > 0.25 && d < 45.0,
            "expected ascendant to shift over 1h UT, got Δ={d}° ({a0}° → {a1}°)"
        );
    }

    #[test]
    fn meeus_moon_snapshot_1992_matches_reference_range() {
        let jd = julian_day_ut(Utc.with_ymd_and_hms(1992, 4, 12, 0, 0, 0).single().unwrap());
        let (lon, _lat) = moon_apparent_longitude_deg(jd);
        assert!((133.0..133.4).contains(&lon), "moon longitude was {lon}");
    }
}
