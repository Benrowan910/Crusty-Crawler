use hardware_query::HardwareInfo;

pub struct HardwareMonitorState {
    pub last_update: Instant,
    pub power_info: Option<String>,
    pub thermal_info: Option<String>,
    pub optimization_suggestions: Vec<String>,
}

impl Default for HardwareMonitorState {
    fn default() -> Self {
        Self {
            last_update: Instant::now() - Duration::from_secs(61), // Force immediate update
            power_info: None,
            thermal_info: None,
            optimization_suggestions: Vec::new(),
        }
    }
}

pub fn update_hardware_info(hardware_state: &mut HardwareMonitorState) {
    match HardwareInfo::query() {
        Ok(hw_info) => {
            let mut power_output = String::new();
            let mut thermal_output = String::new();
            let mut suggestions = Vec::new();

            // Power management information
            if let Some(power) = hw_info.power_profile() {
                power_output.push_str(&format!("Power State: {}\n", power.power_state));
                if let Some(power_draw) = power.total_power_draw {
                    power_output.push_str(&format!("Current Power Draw: {:.1}W\n", power_draw));
                }

                // Get optimization recommendations
                let optimizations = power.suggest_power_optimizations();
                for opt in optimizations {
                    suggestions.push(format!("💡 {}", opt.recommendation));
                }
            } else {
                power_output.push_str("Power information not available\n");
            }

            // Thermal analysis
            let thermal = hw_info.thermal();
            if let Some(max_temp) = thermal.max_temperature() {
                thermal_output.push_str(&format!("Max Temperature: {:.1}°C\n", max_temp));
                thermal_output.push_str(&format!("Thermal Status: {}\n", thermal.thermal_status()));

                // Predict thermal throttling
                let prediction = thermal.predict_thermal_throttling(1.0);
                if prediction.will_throttle {
                    thermal_output.push_str(&format!(
                        "⚠️ Thermal throttling predicted: {}\n",
                        prediction.severity
                    ));
                    suggestions.push(format!("🚨 Thermal alert: {}", prediction.severity));
                }

                // Get cooling recommendations
                let cooling_recs = thermal.suggest_cooling_optimizations();
                for rec in cooling_recs.iter().take(2) {
                    suggestions.push(format!("🌡️ {}", rec.description));
                }
            } else {
                thermal_output.push_str("Thermal information not available\n");
            }

            hardware_state.power_info = Some(power_output);
            hardware_state.thermal_info = Some(thermal_output);
            hardware_state.optimization_suggestions = suggestions;
            hardware_state.last_update = Instant::now();
        }
        Err(e) => {
            let error_msg = format!("Error querying hardware: {}", e);
            hardware_state.power_info = Some(error_msg.clone());
            hardware_state.thermal_info = Some(error_msg);
            hardware_state.last_update = Instant::now();
        }
    }
}

#[warn(private_interfaces)]
pub fn get_hardware_status(server_state: &std::sync::Mutex<crate::ServerState>) -> String {
    let mut output = String::new();

    // Update hardware info if needed
    {
        let state = server_state.lock().unwrap();
        if state.hardware_state.lock().unwrap().last_update.elapsed() > Duration::from_secs(60) {
            update_hardware_info(&mut state.hardware_state.lock().unwrap());
        }
    }

    // Add hardware information
    {
        let state = server_state.lock().unwrap();
        let hardware_state = state.hardware_state.lock().unwrap();

        output.push_str("\n=== Power Information ===\n");
        if let Some(power_info) = &hardware_state.power_info {
            output.push_str(power_info);
        } else {
            output.push_str("Power info not available\n");
        }

        output.push_str("\n=== Thermal Information ===\n");
        if let Some(thermal_info) = &hardware_state.thermal_info {
            output.push_str(thermal_info);
        } else {
            output.push_str("Thermal info not available\n");
        }

        if !hardware_state.optimization_suggestions.is_empty() {
            output.push_str("\n=== Optimization Suggestions ===\n");
            for suggestion in &hardware_state.optimization_suggestions {
                output.push_str(&format!("{}\n", suggestion));
            }
        }
    }

    output
}
