use std::fs;
use std::io::ErrorKind;  
use std::path::Path;

struct Gpu {
    device: String,
    hwmon: Option<String>,
}

const DRM_PATH: &str = "/sys/class/drm";
const HWMON_PATH: &str = "/sys/class/hwmon";
const BATTERY_PATH: &str = "/sys/class/power_supply/BAT0";
const PROFILE_PATH: &str = "/sys/firmware/acpi/platform_profile";
const PROFILE_CHOICES_PATH: &str = "/sys/firmware/acpi/platform_profile_choices";
const ARMOURY_PATH: &str = "/sys/class/firmware-attributes/asus-armoury/attributes";
const ASUS_PLATFORM_PATH: &str = "/sys/devices/platform/asus-nb-wmi";
const KBD_LED_PATH: &str = "/sys/class/leds/asus::kbd_backlight";

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

// Write to sys file

fn write_file(path: &str, value: &str) -> Result<(), String> {
    fs::write(path, value).map_err(|e| match e.kind() {
        ErrorKind::PermissionDenied => {
            format!("{}: permission denied (run with sudo)", path)
        }
        ErrorKind::NotFound => {
            format!("{}: not supported on this machine/kernel", path)
        }
        _ => format!("{}: {}", path, e),
    })
}

fn exists(path: &str) -> bool {
    Path::new(path).exists()
}

// keyboard lightning, no RGB

fn kbd_max_brightness() -> Result<u64, String> {
    read_number(&format!("{}/max_brightness", KBD_LED_PATH))
        .map(|v| v as u64)
        .ok_or_else(|| format!("{} not found: no ASUS keyboard backlight", KBD_LED_PATH))
}

fn kbd_set_brightness(level: u64) -> Result<(), String> {
    let max = kbd_max_brightness()?;

    if level > max {
        return Err(format!("brightness must be 0-{}", max));
    }

    write_file(&format!("{}/brightness", KBD_LED_PATH), &level.to_string())
}

fn kbd_on(level: Option<u64>) -> Result<(), String> {
    let max = kbd_max_brightness()?;
    kbd_set_brightness(level.unwrap_or(max).max(1))
}


// Platform profiles (quiet, balanced, performance)

fn set_profile(name: &str) -> Result<(), String> {
    let choices = read_file(PROFILE_CHOICES_PATH).unwrap_or_default();
    if !choices.split_whitespace().any(|c| c == name) {          
        return Err(format!("invalid profile '{}'. Choices: {}", name, choices));
    }
    write_file(PROFILE_PATH, name)                                     
}

// GPU control

fn gpu_hwmon_path(device: &str) -> Option<String> {
    let mut paths: Vec<String> = fs::read_dir(format!("{}/hwmon", device))
        .ok()?
        .flatten()
        .map(|e| e.path().to_string_lossy().into_owned())
        .collect();

    paths.sort();
    paths.into_iter().next()
}


fn find_discrete_gpu() -> Option<Gpu> {
    let mut best: Option<(f64, Gpu)> = None;

    for entry in fs::read_dir(DRM_PATH).ok()?.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();

        let is_card = name
            .strip_prefix("card")
            .map(|rest| !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()))
            .unwrap_or(false);

        if !is_card {
            continue;
        }

        let device = format!("{}/{}/device", DRM_PATH, name);

        if read_file(&format!("{}/vendor", device)).as_deref() != Some("0x1002") {
            continue;
        }

        let vram = read_number(&format!("{}/mem_info_vram_total", device)).unwrap_or(0.0);
        let hwmon = gpu_hwmon_path(&device);

        if best.as_ref().map(|(v, _)| vram > *v).unwrap_or(true) {
            best = Some((vram, Gpu { device, hwmon }));
        }
    }

    best.map(|(_, gpu)| gpu)
}


fn require_gpu() -> Result<Gpu, String> {
    find_discrete_gpu().ok_or_else(|| "no AMD discrete GPU found".to_string())
}

fn asus_attr(names: &[&str]) -> Option<String> {
    for name in names {
        let armoury = format!("{}/{}/current_value", ARMOURY_PATH, name);
        if exists(&armoury) {
            return Some(armoury);
        }

        let legacy = format!("{}/{}", ASUS_PLATFORM_PATH, name);
        if exists(&legacy) {
            return Some(legacy);
        }
    }

    None
}

fn gpu_mode(mode: &str) -> Result<(), String> {
    let mux = asus_attr(&["gpu_mux_mode"]);
    let disable = asus_attr(&["dgpu_disable"]).ok_or("dgpu_disable not supported")?;

    match mode {
        "igpu" => {
            if let Some(m) = &mux {
                if read_file(m).as_deref() == Some("0") {
                    return Err("MUX is in dGPU-only mode: run `gpu_mode(hybrid)`, reboot, then retry".into());
                }
            }
            write_file(&disable, "1")
        }
        "hybrid" => {
            write_file(&disable, "0")?;                       
            if let Some(m) = &mux { write_file(m, "1")?; }    
            Ok(())
        }
        "dgpu" => {
            write_file(&disable, "0")?;                       
            let m = mux.ok_or("no MUX switch on this laptop")?;
            write_file(&m, "0")                              
        }
        _ => Err("mode must be 'igpu', 'hybrid' or 'dgpu'".into()),
    }
}



fn main() {
        print_dashboard();

        // println!("{:?}", kbd_set_brightness(2));
        // println!("{:?}", set_profile("quiet"));
        //println!("{:?}", gpu_mode("igpu")) // needs reboot, switching back to hybrid turns off monitor
        // sudo cargo run
}
