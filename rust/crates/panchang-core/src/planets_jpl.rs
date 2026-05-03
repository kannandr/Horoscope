//! JPL approximate Keplerian planetary positions (mean ecliptic of J2000),
//! valid roughly **1800 AD–2050 AD** per
//! <https://ssd.jpl.nasa.gov/planets/approx_pos.html> Table 1.
//!
//! Output longitude includes the same nutation-in-longitude correction used for
//! the Moon so positions align with [`super::sun_apparent_longitude_deg`] style
//! apparent ecliptic longitude of date.

use crate::ephemeris::{centuries_j2000, nutation_longitude_arcsec, reduce_deg};
use std::f64::consts::PI;

fn radians(x: f64) -> f64 {
    x * PI / 180.0
}

/// Planets covered by the JPL Table‑1 fit (Earth taken from the EM Bary row).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeplerPlanet {
    Mercury,
    Venus,
    Mars,
    Jupiter,
    Saturn,
}

struct Elem {
    a0: f64,
    da: f64,
    e0: f64,
    de: f64,
    i0: f64,
    di: f64,
    l0: f64,
    dl: f64,
    varpi0: f64,
    dvarpi: f64,
    node0: f64,
    dnode: f64,
}

// Table 1 — https://ssd.jpl.nasa.gov/planets/approx_pos.html
const MERCURY: Elem = Elem {
    a0: 0.38709927,
    da: 0.00000037,
    e0: 0.20563593,
    de: 0.00001906,
    i0: 7.00497902,
    di: -0.00594749,
    l0: 252.25032350,
    dl: 149472.67411175,
    varpi0: 77.45779628,
    dvarpi: 0.16047689,
    node0: 48.33076593,
    dnode: -0.12534081,
};
const VENUS: Elem = Elem {
    a0: 0.72333566,
    da: 0.00000390,
    e0: 0.00677672,
    de: -0.00004107,
    i0: 3.39467605,
    di: -0.00078890,
    l0: 181.97909950,
    dl: 58517.81538729,
    varpi0: 131.60246718,
    dvarpi: 0.00268329,
    node0: 76.67984255,
    dnode: -0.27769418,
};
/// Earth–Moon barycenter (used as Earth’s heliocentric position).
const EARTH: Elem = Elem {
    a0: 1.00000261,
    da: 0.00000562,
    e0: 0.01671123,
    de: -0.00004392,
    i0: -0.00001531,
    di: -0.01294668,
    l0: 100.46457166,
    dl: 35999.37244981,
    varpi0: 102.93768193,
    dvarpi: 0.32327364,
    node0: 0.0,
    dnode: 0.0,
};
const MARS: Elem = Elem {
    a0: 1.52371034,
    da: 0.00001847,
    e0: 0.09339410,
    de: 0.00007882,
    i0: 1.84969142,
    di: -0.00813131,
    l0: -4.55343205,
    dl: 19140.30268499,
    varpi0: -23.94362959,
    dvarpi: 0.44441088,
    node0: 49.55953891,
    dnode: -0.29257343,
};
const JUPITER: Elem = Elem {
    a0: 5.20288700,
    da: -0.00011607,
    e0: 0.04838624,
    de: -0.00013253,
    i0: 1.30439695,
    di: -0.00183714,
    l0: 34.39644051,
    dl: 3034.74612775,
    varpi0: 14.72847983,
    dvarpi: 0.21252668,
    node0: 100.47390909,
    dnode: 0.20469106,
};
const SATURN: Elem = Elem {
    a0: 9.53667594,
    da: -0.00125060,
    e0: 0.05386179,
    de: -0.00050991,
    i0: 2.48599187,
    di: 0.00193609,
    l0: 49.95424423,
    dl: 1222.49362201,
    varpi0: 92.59887831,
    dvarpi: -0.41897216,
    node0: 113.66242448,
    dnode: -0.28867794,
};

fn elem_row(p: KeplerPlanet) -> &'static Elem {
    match p {
        KeplerPlanet::Mercury => &MERCURY,
        KeplerPlanet::Venus => &VENUS,
        KeplerPlanet::Mars => &MARS,
        KeplerPlanet::Jupiter => &JUPITER,
        KeplerPlanet::Saturn => &SATURN,
    }
}

/// Solve Kepler equation `M = E - e sin E` for eccentric anomaly `E` (radians).
fn solve_kepler_e_rad(m_rad: f64, e: f64) -> f64 {
    let mut e_anom = m_rad + e * m_rad.sin();
    for _ in 0..25 {
        let dm = m_rad - (e_anom - e * e_anom.sin());
        let de = dm / (1.0 - e * e_anom.cos());
        e_anom += de;
        if de.abs() < 1e-14 {
            break;
        }
    }
    e_anom
}

fn helio_xyz_au(jd: f64, row: &Elem) -> (f64, f64, f64) {
    let t = centuries_j2000(jd);
    let a = row.a0 + row.da * t;
    let e = row.e0 + row.de * t;
    let i = radians(row.i0 + row.di * t);
    let l = reduce_deg(row.l0 + row.dl * t);
    let varpi = reduce_deg(row.varpi0 + row.dvarpi * t);
    let om = reduce_deg(row.node0 + row.dnode * t);
    let w = radians(reduce_deg(varpi - om));
    let omr = radians(om);

    let m_deg = reduce_deg(l - varpi);
    let m_rad = radians(m_deg);
    let e_anom = solve_kepler_e_rad(m_rad, e);

    let x1 = a * (e_anom.cos() - e);
    let y1 = a * (1.0 - e * e).sqrt() * e_anom.sin();

    let cw = w.cos();
    let sw = w.sin();
    let co = omr.cos();
    let so = omr.sin();
    let ci = i.cos();
    let si = i.sin();

    let px = cw * co - sw * so * ci;
    let py = cw * so + sw * co * ci;
    let pz = sw * si;
    let qx = -sw * co - cw * so * ci;
    let qy = -sw * so + cw * co * ci;
    let qz = cw * si;

    let x = px * x1 + qx * y1;
    let y = py * x1 + qy * y1;
    let z = pz * x1 + qz * y1;
    (x, y, z)
}

/// Geocentric ecliptic longitude & latitude (degrees), **mean J2000 ecliptic**, no nutation.
fn geo_ecliptic_mean_lon_lat(jd: f64, planet: KeplerPlanet) -> (f64, f64) {
    let (xp, yp, zp) = helio_xyz_au(jd, elem_row(planet));
    let (xe, ye, ze) = helio_xyz_au(jd, &EARTH);
    let x = xp - xe;
    let y = yp - ye;
    let z = zp - ze;
    let lon = degrees(y.atan2(x));
    let lat = degrees(z.atan2((x * x + y * y).sqrt()));
    (reduce_deg(lon), lat)
}

fn degrees(x: f64) -> f64 {
    x * 180.0 / PI
}

/// Nutation-corrected **apparent** geocentric ecliptic longitude, matching the
/// correction applied in [`super::moon_apparent_longitude_deg`].
pub fn planet_geocentric_apparent_longitude_deg(jd: f64, planet: KeplerPlanet) -> f64 {
    let (lon, _lat) = geo_ecliptic_mean_lon_lat(jd, planet);
    let dpsi = nutation_longitude_arcsec(jd) / 3600.0;
    reduce_deg(lon + dpsi)
}

/// True if geocentric ecliptic longitude is decreasing over a short UT span (`±step_days`).
pub fn planet_geocentric_longitude_retrograde(jd: f64, planet: KeplerPlanet, step_days: f64) -> bool {
    let l0 = planet_geocentric_apparent_longitude_deg(jd - step_days, planet);
    let l1 = planet_geocentric_apparent_longitude_deg(jd, planet);
    let l2 = planet_geocentric_apparent_longitude_deg(jd + step_days, planet);

    fn forward_delta(prev: f64, next: f64) -> f64 {
        let mut d = reduce_deg(next - prev);
        if d > 180.0 {
            d -= 360.0;
        }
        if d < -180.0 {
            d += 360.0;
        }
        d
    }

    let d1 = forward_delta(l0, l1);
    let d2 = forward_delta(l1, l2);
    (d1 + d2) < 0.0
}

/// Mean lunar **north** node (ascending) tropical longitude, degrees \[0,360).
/// Polynomial from Meeus *Astronomical Algorithms* (mean node on the ecliptic).
pub fn mean_north_node_tropical_longitude_deg(jd: f64) -> f64 {
    let t = centuries_j2000(jd);
    reduce_deg(
        125.04452 + t * (-1934.136261 + t * (0.0020708 + t / 450000.0)),
    )
}

/// Mean north node with nutation-in-longitude (for consistency with planets/Sun).
pub fn mean_north_node_apparent_longitude_deg(jd: f64) -> f64 {
    let lon = mean_north_node_tropical_longitude_deg(jd);
    let dpsi = nutation_longitude_arcsec(jd) / 3600.0;
    reduce_deg(lon + dpsi)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::julian_day_ut;
    use chrono::{TimeZone, Utc};

    #[test]
    fn ketu_opposes_rahu_mean() {
        let jd = julian_day_ut(Utc.with_ymd_and_hms(2000, 1, 1, 12, 0, 0).single().unwrap());
        let r = mean_north_node_apparent_longitude_deg(jd);
        let k = reduce_deg(r + 180.0);
        assert!((reduce_deg(k - r) - 180.0).abs() < 1e-6);
    }

    #[test]
    fn mars_moves_consistently() {
        let jd = julian_day_ut(Utc.with_ymd_and_hms(1995, 6, 15, 0, 0, 0).single().unwrap());
        let l0 = planet_geocentric_apparent_longitude_deg(jd - 1.0, KeplerPlanet::Mars);
        let l1 = planet_geocentric_apparent_longitude_deg(jd + 1.0, KeplerPlanet::Mars);
        let d = (l1 - l0).abs();
        let d = if d > 180.0 { 360.0 - d } else { d };
        assert!(d > 0.01 && d < 2.5, "mars moved {d}° over 2d");
    }
}
