use sysinfo::Disks;

async fn check_disks() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let disks = Disks::new_with_refreshed_list();
    let mut result = Vec::new();

    for disk in disks.list() {
        let info = format!("{:?}", disk.name());
        result.push(info);
    }

    Ok(result)
}
