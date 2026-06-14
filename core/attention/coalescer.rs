//! Packet coalescing boundary.

#[derive(Clone, Debug, PartialEq)]
pub struct CoalescedTrend {
    pub trend: String,
    pub confidence: f32,
    pub source_count: usize,
    pub coalesced_from: String,
}

pub fn bridge_risk_trend(source_count: usize) -> Option<CoalescedTrend> {
    if source_count < 3 {
        return None;
    }

    let confidence = (0.5 + (source_count as f32 / 1000.0 * 0.33)).min(0.99);
    Some(CoalescedTrend {
        trend: "Bridge A risk increasing".to_string(),
        confidence,
        source_count,
        coalesced_from: "low_level_anomaly_packets".to_string(),
    })
}
