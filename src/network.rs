// Author: Ben Rowan
// The purpose of this file is to provide network related functions for gathering and organizing information related to the
// in-flow and out-flow of network traffic to a system.

use std::time::{Duration, Instant};
use sysinfo::Networks;

async fn network_info() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Implementation of network_info function
    let networks = Networks::new_with_refreshed_list();

    let output = networks
        .iter()
        .map(|(interface_name, data)| {
            format!(
                "{interface_name}: {} MB (down) / {} MB (Up)",
                data.total_received() / 1024 / 1024,
                data.total_transmitted() / 1024 / 1024,
            )
        })
        .collect();

    Ok(output)
}

async fn network_traffic() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut networks = Networks::new();
    let mut results = Vec::new();

    // First measurement
    networks.refresh(true);
    let previous_data: Vec<(String, u64, u64)> = networks
        .iter()
        .map(|(name, data)| {
            (
                name.to_string(),
                data.total_received(),
                data.total_transmitted(),
            )
        })
        .collect();

    // Wait for a short interval to measure traffic rate
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Second measurement
    networks.refresh(true);

    for (interface_name, data) in networks.iter() {
        if let Some((_, prev_received, prev_transmitted)) = previous_data
            .iter()
            .find(|(name, _, _)| name == interface_name)
        {
            let received_diff = data.total_received().saturating_sub(*prev_received);
            let transmitted_diff = data.total_transmitted().saturating_sub(*prev_transmitted);

            results.push(format!(
                "{interface_name}: {:.1} kB/s ↓ / {:.1} kB/s ↑",
                received_diff as f64 / 1024.0,
                transmitted_diff as f64 / 1024.0
            ));
        }
    }

    Ok(results)
}
