#[cfg(feature = "gui")]
use crate::{
    color::{PRESETS, Rgb},
    config, device,
    error::KreoError,
    protocol::{LightingConfig, LightingMode},
};

#[cfg(feature = "gui")]
struct KreoApp {
    color: [f32; 3],
    mode: LightingMode,
    brightness: u8,
    speed: u8,
    status: String,
}

#[cfg(feature = "gui")]
impl Default for KreoApp {
    fn default() -> Self {
        let saved = config::load_last_lighting().unwrap_or_default();
        Self {
            color: saved.color.egui(),
            mode: saved.mode,
            brightness: saved.brightness,
            speed: saved.speed,
            status: "Configure lighting, then click Apply.".into(),
        }
    }
}

#[cfg(feature = "gui")]
impl eframe::App for KreoApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        use eframe::egui;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("kreoctl");
            ui.label("Kreo Hive RGB controller for Linux");
            ui.add_space(8.0);

            let base_rgb = Rgb::from_egui(self.color);
            // Modulate preview brightness according to slider
            let preview_factor = (self.brightness as f32) / 5.0;
            let preview_r = (base_rgb.r as f32 * preview_factor).round() as u8;
            let preview_g = (base_rgb.g as f32 * preview_factor).round() as u8;
            let preview_b = (base_rgb.b as f32 * preview_factor).round() as u8;

            let (rect, _) = ui
                .allocate_exact_size(egui::vec2(ui.available_width(), 48.0), egui::Sense::hover());
            ui.painter().rect_filled(
                rect,
                8.0,
                egui::Color32::from_rgb(preview_r, preview_g, preview_b),
            );
            ui.label(format!(
                "Preview: {} (Mode: {}, Brightness: {}/5, Speed: {}/5)",
                base_rgb.hex(),
                self.mode.name(),
                self.brightness,
                self.speed
            ));
            ui.add_space(8.0);

            ui.label("Lighting Mode:");
            ui.horizontal_wrapped(|ui| {
                ui.selectable_value(&mut self.mode, LightingMode::Static, "Static");
                ui.selectable_value(&mut self.mode, LightingMode::Breathing, "Breathing");
                ui.selectable_value(&mut self.mode, LightingMode::Cycle, "Spectrum Cycle");
                ui.selectable_value(&mut self.mode, LightingMode::Wave, "Rainbow Wave");
                ui.selectable_value(&mut self.mode, LightingMode::Reactive, "Reactive");
                ui.selectable_value(&mut self.mode, LightingMode::Off, "Off");
            });
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Brightness:");
                let mut b = self.brightness as i32;
                if ui
                    .add(egui::Slider::new(&mut b, 1..=5).text("level"))
                    .changed()
                {
                    self.brightness = b as u8;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Speed:");
                let mut s = self.speed as i32;
                if ui
                    .add(egui::Slider::new(&mut s, 1..=5).text("level"))
                    .changed()
                {
                    self.speed = s as u8;
                }
            });
            ui.add_space(8.0);

            ui.label("Color Presets:");
            ui.horizontal_wrapped(|ui| {
                for (name, preset) in PRESETS {
                    if ui.button(name).clicked() {
                        self.color = preset.egui();
                    }
                }
            });
            ui.add_space(6.0);

            ui.label("Custom Color:");
            egui::color_picker::color_edit_button_rgb(ui, &mut self.color);
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("Apply to keyboard").clicked() {
                    let cfg = LightingConfig {
                        mode: self.mode,
                        color: Rgb::from_egui(self.color),
                        brightness: self.brightness,
                        speed: self.speed,
                        direction: 0,
                        multicolor: self.mode.is_inherently_multicolor(),
                    };
                    match device::set_lighting(&cfg).and_then(|_| config::save_last_lighting(&cfg))
                    {
                        Ok(()) => {
                            self.status = format!(
                                "Applied {} (Mode: {}, Brightness: {}, Speed: {})",
                                cfg.color.hex(),
                                cfg.mode.name(),
                                cfg.brightness,
                                cfg.speed
                            );
                        }
                        Err(e) => self.status = e.to_string(),
                    }
                }

                if ui.button("Restore saved").clicked() {
                    match config::load_last_lighting() {
                        Ok(cfg) => match device::set_lighting(&cfg) {
                            Ok(()) => {
                                self.color = cfg.color.egui();
                                self.mode = cfg.mode;
                                self.brightness = cfg.brightness;
                                self.speed = cfg.speed;
                                self.status =
                                    format!("Restored saved configuration ({})", cfg.color.hex());
                            }
                            Err(e) => self.status = e.to_string(),
                        },
                        Err(e) => self.status = e.to_string(),
                    }
                }
            });

            ui.separator();
            ui.label(&self.status);
        });
    }
}

#[cfg(feature = "gui")]
pub fn run() -> Result<(), KreoError> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "kreoctl",
        options,
        Box::new(|_cc| Ok(Box::new(KreoApp::default()))),
    )
    .map_err(|e| KreoError::Gui(e.to_string()))
}

#[cfg(not(feature = "gui"))]
pub fn run() -> Result<(), crate::error::KreoError> {
    Err(crate::error::KreoError::GuiNotCompiled)
}
