use anyhow::Context;
use panchang_core::{snapshot, SnapshotRequest};

fn main() -> anyhow::Result<()> {
    let fixture_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "crates/panchang-golden/fixtures/snapshots.json".to_string());
    let raw =
        std::fs::read_to_string(&fixture_path).with_context(|| format!("read {fixture_path}"))?;
    let fixtures: Vec<SnapshotRequest> =
        serde_json::from_str(&raw).context("parse fixture requests")?;
    for req in fixtures {
        let out = snapshot(req.clone()).with_context(|| format!("snapshot {}", req.when_local))?;
        println!(
            "{} {} lat={} lon={} => tithi={} nakshatra={} jd={:.6}",
            req.when_local,
            req.timezone,
            req.latitude,
            req.longitude,
            out.angas.tithi_name,
            out.angas.nakshatra_name,
            out.jd_ut
        );
    }
    Ok(())
}
