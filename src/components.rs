use sysinfo::Components;

pub async fn check_components() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let components = Components::new_with_refreshed_list();
    let mut result = Vec::new();

    for component in components.list() {
        let label = component.label();
        let temperature = component.temperature(); // This returns an Option<f32>

        let info_string = match temperature {
            Some(temp) => format!("{}: {:.1}°C", label, temp),
            None => format!("{}: Temperature Unavailable", label),
        };
        result.push(info_string);
    }

    // Handle case with no components found
    if result.is_empty() {
        result.push("No system components were detected.".to_string());
    }

    Ok(result)
}
