use Asus_control_center::{
    find_hwmon, find_nvme_hwmons, gpu_mode, kbd_set_brightness, read_battery, read_fan_rpm,
    read_file, read_number, read_profile, read_temperature, set_battery_charge_limit,
    set_profile, BATTERY_PATH, PROFILE_CHOICES_PATH,
};
use eframe::egui;

fn fmt_opt_f64(val: Option<f64>, unit: &str) -> String {
    match val {
        Some(v) => format!("{v:.1}{unit}"),
        None => "N/A".to_string(),
    }
}

fn fmt_opt_u64(val: Option<u64>, unit: &str) -> String {
    match val {
        Some(v) => format!("{v}{unit}"),
        None => "N/A".to_string(),
    }
}

struct LaptopInfo {
    cpu_temp: Option<f64>,
    gpu_temp: Option<f64>,
    cpu_fan: Option<u64>,
    gpu_fan: Option<u64>,
    gpu_power: Option<f64>,
    ssd1_temp: Option<f64>,
    ssd2_temp: Option<f64>,
    battery: Option<u64>,
    profile: String,
}

fn read_laptop_info() -> LaptopInfo {
    let cpu_hwmon = find_hwmon("k10temp");
    let gpu_hwmon = find_hwmon("amdgpu");
    let asus_hwmon = find_hwmon("asus");
    let nvme_devices = find_nvme_hwmons();

    LaptopInfo {
        cpu_temp: cpu_hwmon
            .as_deref()
            .and_then(|h| read_temperature(h, "temp1_input")),
        gpu_temp: gpu_hwmon
            .as_deref()
            .and_then(|h| read_temperature(h, "temp1_input")),
        cpu_fan: asus_hwmon
            .as_deref()
            .and_then(|h| read_fan_rpm(h, "fan1_input")),
        gpu_fan: asus_hwmon
            .as_deref()
            .and_then(|h| read_fan_rpm(h, "fan2_input")),
        gpu_power: gpu_hwmon
            .as_deref()
            .and_then(|h| read_number(&format!("{h}/power1_input")))
            .map(|v| v / 1_000_000.0),
        ssd1_temp: nvme_devices
            .first()
            .and_then(|h| read_temperature(h, "temp1_input")),
        ssd2_temp: nvme_devices
            .get(1)
            .and_then(|h| read_temperature(h, "temp1_input")),
        battery: read_battery(),
        profile: read_profile().unwrap_or_else(|| "unknown".to_string()),
    }
}

struct ControlGui {
    info: LaptopInfo,

    battery_limit_input: String,
    gpu_mode_input: String,
    profile_input: String,
    kbd_brightness_input: String,

    status_text: String,
    status_is_error: bool,
}

impl ControlGui {
    fn new() -> Self {
        let current_limit = read_file(&format!("{BATTERY_PATH}/charge_control_end_threshold"))
            .unwrap_or_else(|| "85".to_string());
        let current_profile = read_profile().unwrap_or_else(|| "balanced".to_string());

        Self {
            info: read_laptop_info(),
            battery_limit_input: current_limit,
            gpu_mode_input: "hybrid".to_string(),
            profile_input: current_profile,
            kbd_brightness_input: "2".to_string(),
            status_text: "Ready.".to_string(),
            status_is_error: false,
        }
    }

    fn record_result(&mut self, action: &str, res: Result<(), String>) {
        match res {
            Ok(()) => {
                self.status_is_error = false;
                self.status_text = format!("{action}: OK");
                self.info = read_laptop_info();
            }
            Err(err) => {
                self.status_is_error = true;
                self.status_text = format!("{action}: {err}");
            }
        }
    }
}

impl eframe::App for ControlGui {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("ASUS Control Center");
            ui.separator();

            ui.columns(2, |cols| {
                cols[0].group(|ui| {
                    ui.heading("Laptop Info");
                    ui.add_space(4.0);

                    egui::Grid::new("info_grid")
                        .num_columns(2)
                        .spacing([16.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("CPU Temp:");
                            ui.label(fmt_opt_f64(self.info.cpu_temp, " °C"));
                            ui.end_row();

                            ui.label("GPU Temp:");
                            ui.label(fmt_opt_f64(self.info.gpu_temp, " °C"));
                            ui.end_row();

                            ui.label("CPU Fan:");
                            ui.label(fmt_opt_u64(self.info.cpu_fan, " RPM"));
                            ui.end_row();

                            ui.label("GPU Fan:");
                            ui.label(fmt_opt_u64(self.info.gpu_fan, " RPM"));
                            ui.end_row();

                            ui.label("GPU Power:");
                            ui.label(fmt_opt_f64(self.info.gpu_power, " W"));
                            ui.end_row();

                            ui.label("SSD 1 Temp:");
                            ui.label(fmt_opt_f64(self.info.ssd1_temp, " °C"));
                            ui.end_row();

                            ui.label("SSD 2 Temp:");
                            ui.label(fmt_opt_f64(self.info.ssd2_temp, " °C"));
                            ui.end_row();

                            ui.label("Battery:");
                            ui.label(fmt_opt_u64(self.info.battery, "%"));
                            ui.end_row();

                            ui.label("Profile:");
                            ui.label(&self.info.profile);
                            ui.end_row();
                        });

                    ui.add_space(8.0);
                    if ui.button("Refresh").clicked() {
                        self.info = read_laptop_info();
                        self.status_text = "Info refreshed.".to_string();
                        self.status_is_error = false;
                    }
                });

                cols[1].group(|ui| {
                    ui.heading("Controls");
                    ui.add_space(4.0);

                    ui.label("Battery Charge Limit (0–100):");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.battery_limit_input);
                        if ui.button("Set").clicked() {
                            match self.battery_limit_input.trim().parse::<u32>() {
                                Ok(val) => {
                                    let res = set_battery_charge_limit(val);
                                    self.record_result("Battery limit", res);
                                }
                                Err(_) => {
                                    self.record_result(
                                        "Battery limit",
                                        Err("enter a valid number (0-100)".into()),
                                    );
                                }
                            }
                        }
                    });

                    ui.add_space(8.0);

                    ui.label("GPU Mode (igpu / hybrid / dgpu):");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.gpu_mode_input);
                        if ui.button("Set").clicked() {
                            let mode = self.gpu_mode_input.trim().to_string();
                            let res = gpu_mode(&mode);
                            self.record_result("GPU mode", res);
                        }
                    });

                    ui.add_space(8.0);

                    let choices = read_file(PROFILE_CHOICES_PATH).unwrap_or_default();
                    ui.label(format!("Platform Profile ({choices}):"));
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.profile_input);
                        if ui.button("Set").clicked() {
                            let prof = self.profile_input.trim().to_string();
                            let res = set_profile(&prof);
                            self.record_result("Profile", res);
                        }
                    });

                    ui.add_space(8.0);

                    ui.label("Keyboard Backlight Level:");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.kbd_brightness_input);
                        if ui.button("Set").clicked() {
                            match self.kbd_brightness_input.trim().parse::<u64>() {
                                Ok(lvl) => {
                                    let res = kbd_set_brightness(lvl);
                                    self.record_result("Backlight", res);
                                }
                                Err(_) => {
                                    self.record_result(
                                        "Backlight",
                                        Err("enter a valid integer level".into()),
                                    );
                                }
                            }
                        }
                    });
                });
            });

            ui.add_space(10.0);
            ui.separator();

            let color = if self.status_is_error {
                egui::Color32::LIGHT_RED
            } else {
                egui::Color32::LIGHT_GREEN
            };
            ui.colored_label(color, &self.status_text);
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "ASUS Control Center",
        options,
        Box::new(|_cc| Ok(Box::new(ControlGui::new()))),
    )
}