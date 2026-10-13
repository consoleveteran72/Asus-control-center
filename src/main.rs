use std::fs;

const HWMON_PATH: &str = "/sys/class/hwmon";
const BATTERY_PATH: &str = "/sys/class/power_supply/BAT0";
const PROFILE_PATH: &str = "/sys/firmware/acpi/platform_profile";

fn read_file(path: &str) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
}

fn read_number(path: &str) -> Option<f64> {
    read_file(path)?.parse::<f64>().ok()
}

fn find_hwmon(name: &str) -> Option<String> {
    let entries = fs::read_dir(HWMON_PATH).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();
        let name_path = path.join("name");

        if let Ok(hwmon_name) = fs::read_to_string(name_path) {
            if hwmon_name.trim() == name {
                return Some(path.to_string_lossy().into_owned());
            }
        }
    }

    None
}

fn read_temperature(hwmon: &str, file: &str) -> Option<f64> {
    let path = format!("{}/{}", hwmon, file);

    read_number(&path).map(|value| value / 1000.0)
}

fn read_fan_rpm(hwmon: &str, file: &str) -> Option<u64> {
    let path = format!("{}/{}", hwmon, file);

    read_number(&path).map(|value| value as u64)
}

fn read_battery() -> Option<u64> {
    let path = format!("{}/capacity", BATTERY_PATH);

    read_number(&path).map(|value| value as u64)
}

fn read_profile() -> Option<String> {
    read_file(PROFILE_PATH)
}

fn find_nvme_hwmons() -> Vec<String> {
    let mut devices = Vec::new();

    let entries = match fs::read_dir(HWMON_PATH) {
        Ok(entries) => entries,
        Err(_) => return devices,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name_path = path.join("name");

        if let Ok(name) = fs::read_to_string(name_path) {
            if name.trim() == "nvme" {
                devices.push(path.to_string_lossy().into_owned());
            }
        }
    }

    devices.sort();
    devices
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

fn print_dashboard() {
    let cpu_hwmon = find_hwmon("k10temp");
    let gpu_hwmon = find_hwmon("amdgpu");
    let asus_hwmon = find_hwmon("asus");

    let cpu_temp = cpu_hwmon
        .as_deref()
        .and_then(|hwmon| read_temperature(hwmon, "temp1_input"));

    let gpu_temp = gpu_hwmon
        .as_deref()
        .and_then(|hwmon| read_temperature(hwmon, "temp1_input"));

    let gpu_power = gpu_hwmon
        .as_deref()
        .and_then(|hwmon| {
            read_number(&format!("{}/power1_input", hwmon))
        })
        .map(|value| value / 1_000_000.0);

    let cpu_fan = asus_hwmon
        .as_deref()
        .and_then(|hwmon| read_fan_rpm(hwmon, "fan1_input"));

    let gpu_fan = asus_hwmon
        .as_deref()
        .and_then(|hwmon| read_fan_rpm(hwmon, "fan2_input"));

    let battery = read_battery();

    let profile = read_profile()
        .unwrap_or_else(|| "unknown".to_string());

    let nvme_devices = find_nvme_hwmons();

    let ssd1_temp = nvme_devices
        .get(0)
        .and_then(|hwmon| read_temperature(hwmon, "temp1_input"));

    let ssd2_temp = nvme_devices
        .get(1)
        .and_then(|hwmon| read_temperature(hwmon, "temp1_input"));

    clear_screen();


    print_value("CPU", cpu_temp, "°C");
    print_value("GPU", gpu_temp, "°C");

    print_integer_value("CPU Fan", cpu_fan, " RPM");
    print_integer_value("GPU Fan", gpu_fan, " RPM");

    print_value("GPU Power", gpu_power, " W");

    print_value("SSD 1", ssd1_temp, "°C");
    print_value("SSD 2", ssd2_temp, "°C");

    match battery {
        Some(value) => {
            println!(
                " {:<18} {:>16}% ",
                "Battery",
                value
            );
        }

        None => {
            println!(
                " {:<18} {:>16}  ",
                "Battery",
                "N/A"
            );
        }
    }

    

    println!(
        " {:<18} {:>16}   ",
        "Profile",
        profile
    );

    
}

fn print_value(
    label: &str,
    value: Option<f64>,
    unit: &str,
) {
    match value {
        Some(value) => {
            println!(
                " {:<18} {:>13.1}{} ",
                label,
                value,
                unit
            );
        }

        None => {
            println!(
                " {:<18} {:>16} ",
                label,
                "N/A"
            );
        }
    }
}

fn print_integer_value(
    label: &str,
    value: Option<u64>,
    unit: &str,
) {
    match value {
        Some(value) => {
            println!(
                " {:<18} {:>13}{} ",
                label,
                value,
                unit
            );
        }

        None => {
            println!(
                " {:<18} {:>16} ",
                label,
                "N/A"
            );
        }
    }
}

fn main() {
        print_dashboard();
}
