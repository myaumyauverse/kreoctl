#[cfg(feature = "gui")]
use crate::{
    config, device,
    protocol::{PRESETS, Rgb},
};

#[cfg(feature = "gui")]
struct KreoApp {
    color: [f32; 3],
    status: String,
}

#[cfg(feature = "gui")]
impl Default for KreoApp {
    fn default() -> Self {
        let saved = config::load_last_color().unwrap_or(Rgb::new(255, 0, 0));
        Self {
            color: saved.egui(),
            status: "Choose a color, then click Apply.".into(),
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

            let rgb = Rgb::from_egui(self.color);
            let (rect, _) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), 52.0),
                egui::Sense::hover(),
            );
            ui.painter().rect_filled(
                rect,
                8.0,
                egui::Color32::from_rgb(rgb.r, rgb.g, rgb.b),
            );
            ui.label(format!("Preview: {}", rgb.hex()));
            ui.add_space(8.0);

            ui.horizontal_wrapped(|ui| {
                for (name, preset) in PRESETS {
                    if ui.button(name).clicked() {
                        self.color = preset.egui();
                    }
                }
            });
            ui.add_space(8.0);
            ui.label("Custom color");
            egui::color_picker::color_edit_button_rgb(ui, &mut self.color);
            ui.add_space(8.0);

            if ui.button("Apply to keyboard").clicked() {
                let color = Rgb::from_egui(self.color);
                match device::set_color(color).and_then(|_| config::save_last_color(color)) {
                    Ok(()) => self.status = format!("Applied {}", color.hex()),
                    Err(e) => self.status = e,
                }
            }
            if ui.button("Restore saved color").clicked() {
                match config::load_last_color() {
                    Some(color) => match device::set_color(color) {
                        Ok(()) => {
                            self.color = color.egui();
                            self.status = format!("Restored {}", color.hex());
                        }
                        Err(e) => self.status = e,
                    },
                    None => self.status = "No saved color yet.".into(),
                }
            }
            ui.separator();
            ui.label(&self.status);
            ui.small(
                "This first version supports static colors. Effects and brightness need verified protocol commands.",
            );
        });
    }
}

#[cfg(feature = "gui")]
pub fn run() -> Result<(), String> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "kreoctl",
        options,
        Box::new(|_cc| Ok(Box::new(KreoApp::default()))),
    )
    .map_err(|e| format!("Could not start GUI: {e}"))
}

#[cfg(not(feature = "gui"))]
pub fn run() -> Result<(), String> {
    Err("kreoctl was compiled without GUI support. Rebuild with `--features gui` to enable the graphical interface.".into())
}
