use crate::types::AyanamshaId;

pub fn delta_deg(jd: f64, id: AyanamshaId) -> f64 {
    let t = (jd - 2451545.0) / 36525.0;
    match id {
        AyanamshaId::Lahiri | AyanamshaId::LahiriAltStub => {
            (23.85456338348 + t * (1.3965025622 + t * (7.01e-9 + t * (-4.32e-11)))) % 360.0
        }
        AyanamshaId::Raman => (22.8666667 + 1.3969714 * t) % 360.0,
    }
}
