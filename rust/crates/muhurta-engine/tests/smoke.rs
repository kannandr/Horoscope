use muhurta_engine::{search_muhurta, InProcessPanchangClient, MuhurtaSearchRequest};

#[tokio::test]
async fn muhurta_runs_for_one_day_in_bangalore() {
    let client = InProcessPanchangClient;
    let out = search_muhurta(
        &client,
        MuhurtaSearchRequest {
            date_start: "2026-04-30".to_string(),
            date_end: "2026-04-30".to_string(),
            timezone: "Asia/Kolkata".to_string(),
            latitude: 12.97,
            longitude: 77.59,
            purpose_preset: None,
            min_duration_minutes: Some(45),
            ayanamsha: None,
            engine: None,
        },
    )
    .await
    .expect("muhurta");
    assert_eq!(out.preset, "south_indian_tamil_general");
    assert!(out.windows.iter().all(|w| w.duration_minutes >= 15));
    assert!(out.windows.iter().all(|w| w.score >= 55));
}
