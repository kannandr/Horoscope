//! Vimshottari mahadasha / antardasha (bhukti) using a **365.25-day Vimshottari year**
//! for JD spans (matches §4.1 horizon wording). Antara fractions use the 120-year cycle.

use chrono_tz::Tz;

use panchang_core::{local_iso_from_jd, names};

use crate::output::{
    AntardashaOut, DashaAliasesOut, DashaBhuktiOut, DashaWindowOut, MahadashaSegmentOut,
    MahadashaYearsOut, MoonAtBirthDashaOut,
};

/// One tropical mean year per Vimshottari year when converting mahadasha lengths to days.
pub(crate) const DAYS_PER_VIMSHOTTARI_YEAR: f64 = 365.25;

pub(crate) const LORDS_ORDER: [&str; 9] = [
    "ketu", "venus", "sun", "moon", "mars", "rahu", "jupiter", "saturn", "mercury",
];

const LORD_DISPLAY_EN: [&str; 9] = [
    "Ketu", "Venus", "Sun", "Moon", "Mars", "Rahu", "Jupiter", "Saturn", "Mercury",
];

const LORD_DISPLAY_TA: [&str; 9] = [
    "Ketu", "Velli", "Nyayiru", "Thingal", "Chevvai", "Rahu", "Viyazhan", "Sani", "Budhan",
];

const MAH_YEARS: [f64; 9] = [7.0, 20.0, 6.0, 10.0, 7.0, 18.0, 16.0, 19.0, 17.0];

#[inline]
fn lord_index_from_nakshatra(nakshatra_index: u8) -> usize {
    (nakshatra_index.saturating_sub(1) as usize) % 9
}

#[inline]
fn mahadasha_days_for_lord(lord_idx: usize) -> f64 {
    MAH_YEARS[lord_idx % 9] * DAYS_PER_VIMSHOTTARI_YEAR
}

/// Fraction \[0,1) of progress through the Moon's birth nakshatra (by sidereal longitude).
fn moon_fraction_in_nakshatra(moon_sidereal_deg: f64, nakshatra_index: u8) -> f64 {
    let lon = panchang_core::reduce_deg(moon_sidereal_deg);
    let span = 360.0 / 27.0;
    let idx = nakshatra_index as f64;
    let pos = lon - (idx - 1.0) * span;
    (pos / span).clamp(0.0, 1.0)
}

fn clip_interval(
    start: f64,
    end: f64,
    win_lo: f64,
    win_hi: f64,
) -> Option<(f64, f64)> {
    let s = start.max(win_lo);
    let e = end.min(win_hi);
    if e < win_lo || s > win_hi || e <= s + 1e-9 {
        None
    } else {
        Some((s, e))
    }
}

pub(crate) fn build_dasha_bhukti(
    birth_jd_ut: f64,
    moon_sidereal_deg: f64,
    nakshatra_index: u8,
    as_of_jd_ut: f64,
    horizon_end_jd_ut: f64,
    horizon_years_after_as_of: i32,
    timezone_label: &str,
    tz: Tz,
    as_of_local: String,
    horizon_end_local: String,
) -> DashaBhuktiOut {
    let lord_start_idx = lord_index_from_nakshatra(nakshatra_index);
    let frac = moon_fraction_in_nakshatra(moon_sidereal_deg, nakshatra_index);

    let md0_days = mahadasha_days_for_lord(lord_start_idx);
    let elapsed_at_birth = frac * md0_days;
    let balance_at_birth = md0_days - elapsed_at_birth;

    let md0_start = birth_jd_ut - elapsed_at_birth;

    let nak_name = names::NAKSHATRA_NAMES[(nakshatra_index.saturating_sub(1).min(26)) as usize]
        .to_string();

    let mut mahadashas: Vec<MahadashaSegmentOut> = Vec::new();

    let mut lord_idx = lord_start_idx;
    let mut seg_start = md0_start;

    while seg_start <= horizon_end_jd_ut + 1e-6 {
        let md_days = mahadasha_days_for_lord(lord_idx);
        let seg_end = seg_start + md_days;

        if clip_interval(seg_start, seg_end, birth_jd_ut, horizon_end_jd_ut).is_some() {
            let mut antardashas: Vec<AntardashaOut> = Vec::new();

            let mut antara_years_acc = 0.0_f64;
            for k in 0..9 {
                let antara_lord_idx = (lord_idx + k) % 9;
                let y_ant = MAH_YEARS[antara_lord_idx];
                let frac_lo = antara_years_acc / 120.0;
                antara_years_acc += y_ant;
                let frac_hi = antara_years_acc / 120.0;
                let ant_start = seg_start + md_days * frac_lo;
                let ant_end = seg_start + md_days * frac_hi;

                if let Some((cs, ce)) =
                    clip_interval(ant_start, ant_end, birth_jd_ut, horizon_end_jd_ut)
                {
                    antardashas.push(AntardashaOut {
                        lord: LORDS_ORDER[antara_lord_idx].to_string(),
                        lord_display_en: LORD_DISPLAY_EN[antara_lord_idx].to_string(),
                        lord_display_ta: LORD_DISPLAY_TA[antara_lord_idx],
                        start_jd_ut: cs,
                        end_jd_ut: ce,
                        start_local: local_iso_from_jd(cs, tz),
                        end_local: local_iso_from_jd(ce, tz),
                    });
                }
            }

            let (clip_ms, clip_me) =
                clip_interval(seg_start, seg_end, birth_jd_ut, horizon_end_jd_ut).unwrap();

            mahadashas.push(MahadashaSegmentOut {
                lord: LORDS_ORDER[lord_idx].to_string(),
                lord_display_en: LORD_DISPLAY_EN[lord_idx].to_string(),
                lord_display_ta: LORD_DISPLAY_TA[lord_idx],
                start_jd_ut: clip_ms,
                end_jd_ut: clip_me,
                start_local: local_iso_from_jd(clip_ms, tz),
                end_local: local_iso_from_jd(clip_me, tz),
                antardashas,
            });
        }

        seg_start = seg_end;
        lord_idx = (lord_idx + 1) % 9;

        if seg_start > horizon_end_jd_ut + 1e-6 {
            break;
        }
    }

    DashaBhuktiOut {
        system: "vimshottari",
        aliases: DashaAliasesOut {
            bhukti_en: "Antardasha",
            bhukti_ta_hint: "Bukti / anthira thasa",
        },
        lords_order: LORDS_ORDER,
        mahadasha_years: MahadashaYearsOut {
            ketu: 7,
            venus: 20,
            sun: 6,
            moon: 10,
            mars: 7,
            rahu: 18,
            jupiter: 16,
            saturn: 19,
            mercury: 17,
        },
        moon_at_birth: MoonAtBirthDashaOut {
            nakshatra_index,
            nakshatra_name: nak_name,
            starting_mahadasha_lord: LORDS_ORDER[lord_start_idx].to_string(),
            balance_of_starting_mahadasha_at_birth_days: balance_at_birth,
        },
        window: DashaWindowOut {
            birth_jd_ut,
            as_of_local,
            as_of_jd_ut,
            horizon_end_local,
            horizon_end_jd_ut,
            horizon_years_after_as_of,
            timezone: timezone_label.to_string(),
        },
        mahadashas,
        notes: Some(
            "Intervals are clipped to [birth_jd_ut, horizon_end_jd_ut]. Vimshottari year = 365.25 mean solar days.",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nakshatra_index_maps_to_starting_lord() {
        // Purva Ashadha is index 20 → Venus (plan doc).
        assert_eq!(lord_index_from_nakshatra(20), 1);
        assert_eq!(LORDS_ORDER[1], "venus");
        // Ashwini → Ketu
        assert_eq!(lord_index_from_nakshatra(1), 0);
    }

    #[test]
    fn moon_at_nakshatra_start_has_full_balance() {
        let span = 360.0 / 27.0;
        let lon = span * 19.0 + 0.01; // early Purva Ashadha
        let nak = 20;
        let frac = moon_fraction_in_nakshatra(lon, nak);
        assert!(frac < 0.01, "frac={frac}");
        let bal = mahadasha_days_for_lord(lord_index_from_nakshatra(nak)) * (1.0 - frac);
        assert!(
            (bal - 20.0 * DAYS_PER_VIMSHOTTARI_YEAR).abs() < 25.0,
            "bal={bal}"
        );
    }
}
