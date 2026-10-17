#[cfg(feature = "gui")]
use crate::{
    animation,
    color::{Rgb, parse_color},
    config,
    error::KreoError,
    keyboard::{KeyboardModel, KeyboardVariant},
    profile::ProfileStore,
    protocol::{BRIGHTNESS_MAX, DirectionAxis, LightingConfig, LightingMode, SPEED_MAX, SPEED_MIN},
    sync::{Command, DeviceWorker, Event},
};
#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, CornerRadius, Margin, Pos2, Rect, Sense, Stroke, Vec2};

// Official Kreo Dark & Purple Palette
#[cfg(feature = "gui")]
const KREO_BG: Color32 = Color32::from_rgb(11, 12, 17);
#[cfg(feature = "gui")]
const KREO_SIDEBAR_BG: Color32 = Color32::from_rgb(18, 19, 26);
#[cfg(feature = "gui")]
const KREO_PANEL_BG: Color32 = Color32::from_rgb(22, 23, 32);
#[cfg(feature = "gui")]
const KREO_BORDER: Color32 = Color32::from_rgb(38, 40, 56);
#[cfg(feature = "gui")]
const KREO_PURPLE_ACTIVE: Color32 = Color32::from_rgb(108, 71, 255); // #6C47FF
#[cfg(feature = "gui")]
const KREO_TEXT_WHITE: Color32 = Color32::from_rgb(245, 245, 250);
#[cfg(feature = "gui")]
const KREO_TEXT_MUTED: Color32 = Color32::from_rgb(140, 144, 165);
#[cfg(feature = "gui")]
const KREO_PURPLE_SOFT: Color32 = Color32::from_rgb(170, 150, 255);

// The exact 14 preset colors from the official Windows software
#[cfg(feature = "gui")]
const OFFICIAL_PRESETS: &[(&str, Rgb)] = &[
    // Row 1
    ("Red", Rgb { r: 255, g: 0, b: 0 }),
    (
        "Orange",
        Rgb {
            r: 255,
            g: 128,
            b: 0,
        },
    ),
    (
        "Yellow",
        Rgb {
            r: 255,
            g: 240,
            b: 0,
        },
    ),
    ("Green", Rgb { r: 0, g: 255, b: 0 }),
    (
        "Cyan",
        Rgb {
            r: 0,
            g: 255,
            b: 255,
        },
    ),
    (
        "Blue",
        Rgb {
            r: 0,
            g: 102,
            b: 255,
        },
    ),
    (
        "Purple",
        Rgb {
            r: 138,
            g: 43,
            b: 226,
        },
    ),
    // Row 2
    (
        "Magenta",
        Rgb {
            r: 255,
            g: 0,
            b: 255,
        },
    ),
    (
        "Pink",
        Rgb {
            r: 255,
            g: 20,
            b: 147,
        },
    ),
    (
        "Mint",
        Rgb {
            r: 0,
            g: 255,
            b: 136,
        },
    ),
    (
        "Ice Blue",
        Rgb {
            r: 0,
            g: 191,
            b: 255,
        },
    ),
    (
        "Royal Blue",
        Rgb {
            r: 65,
            g: 105,
            b: 225,
        },
    ),
    (
        "Deep Violet",
        Rgb {
            r: 148,
            g: 0,
            b: 211,
        },
    ),
    (
        "White",
        Rgb {
            r: 255,
            g: 255,
            b: 255,
        },
    ),
];

#[cfg(feature = "gui")]
#[derive(Debug, PartialEq, Eq)]
enum SidebarTab {
    LightSetting,
    OtherSettings,
}

#[cfg(feature = "gui")]
struct KreoApp {
    profile_store: ProfileStore,
    keyboard: KeyboardModel,
    active_tab: SidebarTab,
    // Color picker values; also the paint color for selected keys.
    r_val: u8,
    g_val: u8,
    b_val: u8,
    editing_profile_name: bool,
    temp_profile_name: String,
    device_connected: bool,
    status_msg: String,
    frame_count: u64,

    // Device synchronisation
    worker: DeviceWorker,
    sent_seq: u64,
    sidebar_open: bool,
}

#[cfg(feature = "gui")]
fn parse_key_colors(
    colors: &std::collections::HashMap<String, String>,
) -> impl Iterator<Item = (String, Rgb)> + '_ {
    colors
        .iter()
        .filter_map(|(code, hex)| parse_color(hex).ok().map(|c| (code.clone(), c)))
}

#[cfg(feature = "gui")]
impl KreoApp {
    fn new(ctx: &egui::Context) -> Self {
        let mut store = config::load_profile_store();
        for p in &mut store.profiles {
            p.brightness = p.brightness.min(BRIGHTNESS_MAX);
            p.speed = p.speed.clamp(SPEED_MIN, SPEED_MAX);
        }
        let active_prof = store.active_profile().clone();

        let mut keyboard = KeyboardModel::with_variant(KeyboardVariant::Hive75);
        keyboard.load_key_colors(parse_key_colors(&active_prof.per_key_colors));

        let wake_ctx = ctx.clone();
        let worker = DeviceWorker::spawn(move || wake_ctx.request_repaint());

        Self {
            profile_store: store,
            keyboard,
            active_tab: SidebarTab::LightSetting,
            r_val: active_prof.color.r,
            g_val: active_prof.color.g,
            b_val: active_prof.color.b,
            editing_profile_name: false,
            temp_profile_name: active_prof.name.clone(),
            device_connected: false,
            status_msg: "Looking for keyboard…".into(),
            frame_count: 0,
            worker,
            sent_seq: 0,
            sidebar_open: false,
        }
    }

    fn set_keyboard_variant(&mut self, variant: KeyboardVariant) {
        if self.keyboard.variant != variant {
            self.keyboard.set_variant(variant);
            trim_memory();
        }
    }

    /// The picker color (used to paint selected keys).
    fn current_color(&self) -> Rgb {
        Rgb::new(self.r_val, self.g_val, self.b_val)
    }

    /// The effect color stored in the profile.
    fn base_color(&self) -> Rgb {
        self.profile_store.active_profile().color
    }

    fn current_config(&self) -> LightingConfig {
        let p = self.profile_store.active_profile();
        LightingConfig {
            mode: p.mode,
            color: p.color,
            brightness: p.brightness,
            speed: p.speed,
            direction: p.direction,
            multicolor: p.multicolor,
        }
    }

    fn persist(&self) {
        let _ = config::save_profile_store(&self.profile_store);
        let _ = config::save_last_lighting(&self.current_config());
    }

    fn sync_key_colors_to_profile(&mut self) {
        let map = self
            .keyboard
            .per_key_colors
            .iter()
            .map(|(k, c)| (k.clone(), c.hex()))
            .collect();
        self.profile_store.active_profile_mut().per_key_colors = map;
    }

    /// Sends the active profile's lighting to the keyboard.
    fn apply_to_hardware(&mut self) {
        if self.profile_store.active_profile().mode == LightingMode::Custom {
            self.apply_per_key_to_hardware();
            return;
        }
        self.sent_seq += 1;
        self.worker.send(Command::Apply {
            seq: self.sent_seq,
            config: self.current_config(),
        });
        self.persist();
    }

    /// Uploads per-key colors and switches the keyboard to Custom mode.
    fn apply_per_key_to_hardware(&mut self) {
        let prof = self.profile_store.active_profile_mut();
        prof.mode = LightingMode::Custom;
        prof.multicolor = false;
        if prof.brightness == 0 {
            prof.brightness = BRIGHTNESS_MAX;
        }
        let brightness = prof.brightness;
        self.sync_key_colors_to_profile();
        self.sent_seq += 1;
        self.worker.send(Command::ApplyPerKey {
            seq: self.sent_seq,
            colors: self.keyboard.build_led_buffer(self.base_color()).to_vec(),
            brightness,
        });
        self.persist();
    }

    fn set_mode(&mut self, mode: LightingMode) {
        let prof = self.profile_store.active_profile_mut();
        prof.mode = mode;
        prof.multicolor = mode.is_inherently_multicolor();
        if mode != LightingMode::Off && prof.brightness == 0 {
            prof.brightness = BRIGHTNESS_MAX;
        }
        if mode != LightingMode::Custom {
            self.keyboard.clear_selection();
        }
        self.apply_to_hardware();
    }

    /// Handles a color pick: paints selected keys, or sets the effect color.
    fn set_color(&mut self, color: Rgb) {
        self.r_val = color.r;
        self.g_val = color.g;
        self.b_val = color.b;

        if !self.keyboard.selected_keys.is_empty() {
            self.keyboard.apply_color_to_selection(color);
            self.apply_per_key_to_hardware();
            return;
        }

        let prof = self.profile_store.active_profile_mut();
        prof.color = color;
        if prof.mode == LightingMode::Custom {
            // Unpainted keys follow the base color.
            self.apply_per_key_to_hardware();
            return;
        }
        if !prof.mode.has_color() {
            prof.mode = LightingMode::Normal;
        }
        prof.multicolor = false;
        self.apply_to_hardware();
    }

    fn handle_device_events(&mut self) {
        while let Some(event) = self.worker.try_recv() {
            match event {
                Event::Connected(info) => {
                    self.device_connected = true;
                    self.set_keyboard_variant(info.variant);
                    self.status_msg = format!("{} connected", info.variant.name());
                    if !info.variant.is_verified() {
                        self.status_msg.push_str(" (experimental support)");
                    }
                }
                Event::Disconnected(reason) => {
                    self.device_connected = false;
                    self.status_msg = format!("Keyboard not available: {reason}");
                }
                Event::Applied { seq, config } => {
                    if seq == self.sent_seq {
                        self.status_msg = format!("Applied {}", describe(&config));
                    }
                }
                Event::HardwareState { .. } => {}
                Event::Error(e) => self.status_msg = e,
            }
        }
    }

    fn switch_profile(&mut self, index: usize) {
        self.profile_store.active_index = index;
        let prof = self.profile_store.active_profile().clone();
        self.r_val = prof.color.r;
        self.g_val = prof.color.g;
        self.b_val = prof.color.b;
        self.temp_profile_name = prof.name.clone();
        self.keyboard.clear_selection();
        self.keyboard
            .load_key_colors(parse_key_colors(&prof.per_key_colors));
        self.apply_to_hardware();
        trim_memory();
    }
}

#[cfg(feature = "gui")]
fn describe(cfg: &LightingConfig) -> String {
    let mut parts = vec![cfg.mode.name().to_string()];
    if cfg.mode == LightingMode::Custom {
        parts.push("per-key colors".into());
    } else if cfg.mode.has_color() {
        parts.push(if cfg.multicolor {
            "rainbow".into()
        } else {
            cfg.color.hex()
        });
    }
    if cfg.mode != LightingMode::Off {
        parts.push(format!("brightness {}/{BRIGHTNESS_MAX}", cfg.brightness));
    }
    if cfg.mode.has_speed() {
        parts.push(format!("speed {}/{SPEED_MAX}", cfg.speed));
    }
    parts.join(" • ")
}

/// On Linux with glibc, forces unused per-thread heap arena memory back to the kernel via madvise.
#[inline]
fn trim_memory() {
    #[cfg(all(feature = "gui", target_os = "linux"))]
    unsafe {
        unsafe extern "C" {
            fn malloc_trim(pad: usize) -> std::os::raw::c_int;
        }
        malloc_trim(0);
    }
}

#[cfg(feature = "gui")]
impl eframe::App for KreoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame_count = self.frame_count.saturating_add(1);
        self.handle_device_events();

        // Release free glibc heap arenas once initial GPU upload and glyph tessellation settle
        if self.frame_count == 3 || self.frame_count == 10 {
            trim_memory();
        }

        // ==================== LEFT SIDEBAR ====================
        egui::SidePanel::left("kreo_sidebar")
            .resizable(false)
            .exact_width(260.0)
            .frame(
                egui::Frame::new()
                    .fill(KREO_SIDEBAR_BG)
                    .inner_margin(Margin::symmetric(14, 16)),
            )
            .show_animated(ctx, self.sidebar_open, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(0.0, 10.0);

                        // Brand Header Card
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("KREO")
                                    .color(Color32::WHITE)
                                    .size(16.0)
                                    .strong(),
                            );
                            ui.label(
                                egui::RichText::new(self.keyboard.variant.short_name())
                                    .color(KREO_PURPLE_ACTIVE)
                                    .size(16.0)
                                    .strong(),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(self.keyboard.variant.key_count_label())
                                    .color(KREO_TEXT_MUTED)
                                    .size(11.0),
                            );

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    for &var in &[KeyboardVariant::Hive65, KeyboardVariant::Hive75]
                                    {
                                        let is_active = self.keyboard.variant == var;
                                        let label = if var == KeyboardVariant::Hive75 {
                                            "75%"
                                        } else {
                                            "65%"
                                        };
                                        let btn = egui::Button::new(
                                            egui::RichText::new(label)
                                                .color(if is_active {
                                                    Color32::WHITE
                                                } else {
                                                    KREO_TEXT_MUTED
                                                })
                                                .size(10.0)
                                                .strong(),
                                        )
                                        .fill(if is_active {
                                            KREO_PURPLE_ACTIVE
                                        } else {
                                            Color32::from_rgb(26, 28, 38)
                                        })
                                        .stroke(Stroke::new(
                                            1.0_f32,
                                            if is_active {
                                                KREO_PURPLE_ACTIVE
                                            } else {
                                                Color32::from_rgb(46, 48, 64)
                                            },
                                        ))
                                        .corner_radius(CornerRadius::same(4))
                                        .min_size(Vec2::new(34.0, 18.0));

                                        let mut resp = ui.add(btn);
                                        if !var.is_verified() {
                                            resp = resp.on_hover_text(
                                                "Experimental: Hive 65 per-key LED map and layout are unverified",
                                            );
                                        }
                                        if resp.clicked() {
                                            self.set_keyboard_variant(var);
                                        }
                                    }
                                },
                            );
                        });

                        ui.add_space(6.0);

                        // Unified Profile Card
                        egui::Frame::new()
                            .fill(Color32::from_rgb(18, 19, 26))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 38, 50)))
                            .corner_radius(CornerRadius::same(8))
                            .inner_margin(Margin::symmetric(12, 10))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new("ACTIVE PROFILE")
                                            .color(KREO_TEXT_MUTED)
                                            .size(10.0)
                                            .strong(),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui
                                                .small_button("+ New")
                                                .on_hover_text("Add new profile")
                                                .clicked()
                                            {
                                                let new_count =
                                                    self.profile_store.profiles.len() + 1;
                                                self.profile_store
                                                    .add_profile(&format!("Profile {new_count}"));
                                                self.switch_profile(
                                                    self.profile_store.profiles.len() - 1,
                                                );
                                            }
                                        },
                                    );
                                });
                                ui.add_space(4.0);

                                let active_idx = self.profile_store.active_index;
                                let curr_name = &self.profile_store.active_profile().name;
                                let mut switched_idx = None;
                                egui::ComboBox::from_id_salt("sidebar_profile_combo")
                                    .selected_text(
                                        egui::RichText::new(curr_name)
                                            .color(Color32::WHITE)
                                            .size(12.5)
                                            .strong(),
                                    )
                                    .width(ui.available_width() - 8.0)
                                    .show_ui(ui, |ui| {
                                        for (i, p) in self.profile_store.profiles.iter().enumerate()
                                        {
                                            let is_sel = i == active_idx;
                                            if ui.selectable_label(is_sel, &p.name).clicked() {
                                                switched_idx = Some(i);
                                            }
                                        }
                                    });

                                if let Some(idx) = switched_idx {
                                    self.switch_profile(idx);
                                }

                                ui.add_space(6.0);

                                let curr_idx = self.profile_store.active_index;
                                if self.editing_profile_name {
                                    ui.horizontal(|ui| {
                                        let edit =
                                            ui.text_edit_singleline(&mut self.temp_profile_name);
                                        if ui.small_button("Done").clicked()
                                            || edit.lost_focus()
                                            || ui.input(|i| i.key_pressed(egui::Key::Enter))
                                        {
                                            self.profile_store
                                                .rename_profile(curr_idx, &self.temp_profile_name);
                                            self.editing_profile_name = false;
                                        }
                                    });
                                } else {
                                    ui.horizontal(|ui| {
                                        if ui
                                            .small_button("Rename")
                                            .on_hover_text("Rename active profile")
                                            .clicked()
                                        {
                                            self.editing_profile_name = true;
                                            self.temp_profile_name =
                                                self.profile_store.active_profile().name.clone();
                                        }
                                        if ui
                                            .small_button("Save")
                                            .on_hover_text("Save profile to disk")
                                            .clicked()
                                        {
                                            let _ = config::save_profile_store(&self.profile_store);
                                        }
                                        if self.profile_store.profiles.len() > 1 {
                                            let del_btn = egui::Button::new(
                                                egui::RichText::new("Delete")
                                                    .color(Color32::from_rgb(240, 120, 120)),
                                            );
                                            if ui
                                                .add(del_btn)
                                                .on_hover_text("Delete active profile")
                                                .clicked()
                                            {
                                                self.profile_store.delete_profile(curr_idx);
                                                self.switch_profile(
                                                    self.profile_store.active_index,
                                                );
                                            }
                                        }
                                    });
                                }
                            });

                        ui.add_space(8.0);

                        // Navigation Tabs
                        ui.label(
                            egui::RichText::new("NAVIGATION")
                                .color(KREO_TEXT_MUTED)
                                .size(10.0)
                                .strong(),
                        );
                        ui.add_space(2.0);

                        let tabs = [
                            (
                                SidebarTab::LightSetting,
                                "Light Setting",
                                "Effects & Per-Key RGB",
                            ),
                            (
                                SidebarTab::OtherSettings,
                                "Other Settings",
                                "Firmware & Diagnostics",
                            ),
                        ];

                        for (tab_val, title, subtitle) in tabs {
                            let is_active = self.active_tab == tab_val;
                            let (card_rect, resp) = ui.allocate_exact_size(
                                Vec2::new(ui.available_width(), 46.0),
                                Sense::click(),
                            );
                            let is_hovered = resp.hovered();

                            let card_bg = if is_active {
                                KREO_PURPLE_ACTIVE
                            } else if is_hovered {
                                Color32::from_rgb(26, 28, 38)
                            } else {
                                Color32::from_rgb(18, 19, 26)
                            };

                            let border_stroke = if is_active {
                                Stroke::new(1.0_f32, Color32::from_rgb(170, 135, 255))
                            } else {
                                Stroke::new(1.0_f32, Color32::from_rgb(32, 34, 46))
                            };

                            ui.painter()
                                .rect_filled(card_rect, CornerRadius::same(6), card_bg);
                            ui.painter().rect_stroke(
                                card_rect,
                                CornerRadius::same(6),
                                border_stroke,
                                egui::StrokeKind::Inside,
                            );

                            if is_active {
                                let ind_rect = Rect::from_min_size(
                                    card_rect.left_top(),
                                    Vec2::new(3.5, card_rect.height()),
                                );
                                ui.painter().rect_filled(
                                    ind_rect,
                                    CornerRadius::same(2),
                                    Color32::WHITE,
                                );
                            }

                            let title_color = if is_active {
                                Color32::WHITE
                            } else {
                                Color32::from_rgb(220, 222, 235)
                            };
                            let sub_color = if is_active {
                                Color32::from_rgb(230, 220, 255)
                            } else {
                                KREO_TEXT_MUTED
                            };

                            ui.painter().text(
                                Pos2::new(card_rect.left() + 14.0, card_rect.center().y - 8.0),
                                egui::Align2::LEFT_CENTER,
                                title,
                                egui::FontId::proportional(13.0),
                                title_color,
                            );
                            ui.painter().text(
                                Pos2::new(card_rect.left() + 14.0, card_rect.center().y + 8.5),
                                egui::Align2::LEFT_CENTER,
                                subtitle,
                                egui::FontId::proportional(10.0),
                                sub_color,
                            );

                            if resp.clicked() && self.active_tab != tab_val {
                                self.active_tab = tab_val;
                                trim_memory();
                            }

                            ui.add_space(4.0);
                        }

                        ui.add_space(14.0);

                        // Sync to Hardware Button
                        let sync_btn = egui::Button::new(
                            egui::RichText::new("Sync to Hardware")
                                .color(Color32::WHITE)
                                .size(12.5)
                                .strong(),
                        )
                        .fill(KREO_PURPLE_ACTIVE)
                        .corner_radius(CornerRadius::same(6))
                        .min_size(Vec2::new(ui.available_width(), 34.0));
                        if ui.add(sync_btn).clicked() {
                            self.apply_to_hardware();
                        }

                        ui.add_space(10.0);

                        // Driver Quick Info Card
                        egui::Frame::new()
                            .fill(Color32::from_rgb(15, 16, 22))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(30, 32, 42)))
                            .corner_radius(CornerRadius::same(6))
                            .inner_margin(Margin::symmetric(10, 8))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new("Model:")
                                                .color(KREO_TEXT_MUTED)
                                                .size(10.5),
                                        );
                                        ui.label(
                                            egui::RichText::new(self.keyboard.variant.name())
                                                .color(Color32::WHITE)
                                                .size(10.5)
                                                .strong(),
                                        );
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new("Driver:")
                                                .color(KREO_TEXT_MUTED)
                                                .size(10.5),
                                        );
                                        ui.label(
                                            egui::RichText::new("Linux hidraw (v1.0)")
                                                .color(Color32::WHITE)
                                                .size(10.5),
                                        );
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new("Polling:")
                                                .color(KREO_TEXT_MUTED)
                                                .size(10.5),
                                        );
                                        ui.label(
                                            egui::RichText::new("1000 Hz Wired")
                                                .color(KREO_TEXT_WHITE)
                                                .size(10.5)
                                                .strong(),
                                        );
                                    });
                                });
                            });
                    });
            });

        // ==================== MAIN CONTENT AREA ====================
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(KREO_BG)
                    .inner_margin(Margin::symmetric(24, 16)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if menu_button(ui, self.sidebar_open)
                        .on_hover_text(if self.sidebar_open {
                            "Hide sidebar"
                        } else {
                            "Show profiles & navigation"
                        })
                        .clicked()
                    {
                        self.sidebar_open = !self.sidebar_open;
                    }
                });
                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| match self.active_tab {
                        SidebarTab::LightSetting => {
                            let avail_w = ui.available_width();
                            let container_w = (avail_w - 16.0).clamp(760.0, 1260.0);
                            let avail_h = ui.available_height();
                            if avail_h > 800.0 {
                                let top_pad = ((avail_h - 760.0) * 0.15).clamp(4.0, 24.0);
                                ui.add_space(top_pad);
                            }

                            ui.vertical_centered(|ui| {
                                ui.set_max_width(container_w);
                                self.render_light_setting_view(ui, container_w);
                            });
                        }
                        SidebarTab::OtherSettings => {
                            let avail_w = ui.available_width();
                            let container_w = (avail_w - 16.0).clamp(760.0, 1260.0);
                            ui.vertical_centered(|ui| {
                                ui.set_max_width(container_w);
                                self.render_settings_view(ui);
                            });
                        }
                    });
            });
    }
}

#[cfg(feature = "gui")]
impl KreoApp {
    fn render_light_setting_view(&mut self, ui: &mut egui::Ui, container_w: f32) {
        let cfg = self.current_config();

        // 1. Header
        ui.horizontal(|ui| {
            render_kreo_logo(ui, self.keyboard.variant);
        });

        ui.add_space(12.0);

        // 2. KEYBOARD VISUALIZER (75% or 65%)
        render_keyboard(ui, &mut self.keyboard, &cfg, container_w);

        ui.add_space(14.0);

        // 3. Per-key selection toolbar
        self.render_selection_toolbar(ui);

        ui.add_space(14.0);

        // 4. Control panel
        egui::Frame::new()
            .fill(KREO_PANEL_BG)
            .stroke(Stroke::new(1.0_f32, KREO_BORDER))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::symmetric(20, 16))
            .show(ui, |ui| {
                // Inner width + 2 * 20px margin = keyboard width, so the panel lines up with it
                ui.set_width(container_w - 40.0);
                ui.columns(3, |cols| {
                    cols[0].vertical(|ui| self.render_effect_column(ui));
                    cols[1].vertical(|ui| self.render_color_column(ui));
                    cols[2].vertical(|ui| self.render_picker_column(ui));
                });
            });

        ui.add_space(8.0);

        // Status line
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("STATUS")
                    .color(KREO_PURPLE_SOFT)
                    .size(10.5)
                    .strong(),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(&self.status_msg)
                    .color(KREO_TEXT_MUTED)
                    .size(11.5),
            );
        });
    }

    fn render_selection_toolbar(&mut self, ui: &mut egui::Ui) {
        if !self.keyboard.variant.is_verified() {
            ui.label(
                egui::RichText::new(
                    "Hive 65 support is experimental: effects should work, but per-key colours may land on the wrong keys.",
                )
                .color(Color32::from_rgb(251, 191, 36))
                .size(11.5),
            );
            ui.add_space(4.0);
        }
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 6.0);
            ui.label(
                egui::RichText::new("Per-key:")
                    .color(KREO_TEXT_MUTED)
                    .size(12.0)
                    .strong(),
            );
            let tools: &[(&str, usize)] = match self.keyboard.variant {
                KeyboardVariant::Hive75 => &[
                    ("All Keys", 0),
                    ("WASD", 1),
                    ("Arrows", 2),
                    ("F-Row", 3),
                    ("Letters", 4),
                ],
                KeyboardVariant::Hive65 => &[
                    ("All Keys", 0),
                    ("WASD", 1),
                    ("Arrows", 2),
                    ("Numbers", 6),
                    ("Letters", 4),
                ],
            };
            for &(name, id) in tools {
                if ui.add(chip_button(name, false)).clicked() {
                    match id {
                        0 => self.keyboard.select_all(),
                        1 => self.keyboard.select_wasd(),
                        2 => self.keyboard.select_arrows(),
                        3 => self.keyboard.select_f_keys(),
                        4 => self.keyboard.select_letters(),
                        6 => self.keyboard.select_numbers(),
                        _ => {}
                    }
                }
            }

            let count = self.keyboard.selected_keys.len();
            if count > 0 {
                if ui.add(chip_button("Clear Selection", false)).clicked() {
                    self.keyboard.clear_selection();
                }
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(format!("{count} selected - pick a color to paint"))
                        .color(Color32::from_rgb(170, 150, 255))
                        .size(12.0)
                        .strong(),
                );
                let swatch = self.current_color();
                let apply_btn = egui::Button::new(
                    egui::RichText::new(format!("Paint {}", swatch.hex()))
                        .color(Color32::WHITE)
                        .size(11.5)
                        .strong(),
                )
                .fill(KREO_PURPLE_ACTIVE)
                .corner_radius(CornerRadius::same(5))
                .min_size(Vec2::new(0.0, 24.0));
                if ui.add(apply_btn).clicked() {
                    self.set_color(swatch);
                }
            } else {
                ui.label(
                    egui::RichText::new("Click keys or use a quick select, then pick a color")
                        .color(KREO_TEXT_MUTED)
                        .size(11.0),
                );
            }

            if !self.keyboard.per_key_colors.is_empty()
                && ui
                    .add(chip_button("Reset Key Colors", false))
                    .on_hover_text("Remove all per-key colors (keys follow the effect color)")
                    .clicked()
            {
                self.keyboard.clear_key_colors();
                self.sync_key_colors_to_profile();
                if self.profile_store.active_profile().mode == LightingMode::Custom {
                    self.apply_per_key_to_hardware();
                } else {
                    self.persist();
                }
            }
        });
    }

    fn render_effect_column(&mut self, ui: &mut egui::Ui) {
        let cfg = self.current_config();
        ui.spacing_mut().slider_width = (ui.available_width() - 8.0).max(120.0);
        section_label(ui, "LIGHTING EFFECT");

        // Mode grid
        let mut picked = None;
        let cols_n = 4;
        let gap = 6.0;
        let chip_w =
            ((ui.available_width() - gap * (cols_n as f32 - 1.0)) / cols_n as f32).max(60.0);
        egui::Grid::new("mode_grid")
            .spacing(Vec2::splat(gap))
            .show(ui, |ui| {
                // Surmount only supports four fixed colours; like Kreo's software, keep it CLI-only.
                let modes = LightingMode::ALL
                    .iter()
                    .filter(|m| **m != LightingMode::Surmount);
                for (i, mode) in modes.enumerate() {
                    let label = if *mode == LightingMode::Off {
                        "Off"
                    } else {
                        mode.name()
                    };
                    let btn =
                        chip_button(label, cfg.mode == *mode).min_size(Vec2::new(chip_w, 26.0));
                    if ui.add(btn).clicked() && cfg.mode != *mode {
                        picked = Some(*mode);
                    }
                    if (i + 1) % cols_n == 0 {
                        ui.end_row();
                    }
                }
            });
        if let Some(mode) = picked {
            self.set_mode(mode);
        }

        ui.add_space(12.0);

        // Brightness
        let lit = cfg.mode != LightingMode::Off;
        let mut b = cfg.brightness;
        slider_header(ui, "Brightness", &format!("{b}/{BRIGHTNESS_MAX}"), lit);
        let resp = ui.add_enabled(
            lit,
            egui::Slider::new(&mut b, 0..=BRIGHTNESS_MAX).show_value(false),
        );
        if resp.changed() {
            self.profile_store.active_profile_mut().brightness = b;
            self.apply_to_hardware();
        }

        ui.add_space(8.0);

        // Speed
        let has_speed = cfg.mode.has_speed();
        let mut sp = cfg.speed;
        slider_header(
            ui,
            "Animation Speed",
            &if has_speed {
                format!("{sp}/{SPEED_MAX}")
            } else {
                "n/a for this effect".into()
            },
            has_speed,
        );
        let resp = ui.add_enabled(
            has_speed,
            egui::Slider::new(&mut sp, SPEED_MIN..=SPEED_MAX).show_value(false),
        );
        if resp.changed() {
            self.profile_store.active_profile_mut().speed = sp;
            self.apply_to_hardware();
        }

        ui.add_space(8.0);

        // Direction
        let axis = cfg.mode.direction_axis();
        let (first, second) = match axis {
            Some(DirectionAxis::Vertical) => ("Bottom to top", "Top to bottom"),
            _ => ("Right to left", "Left to right"),
        };
        slider_header(
            ui,
            "Direction",
            if axis.is_some() {
                ""
            } else {
                "n/a for this effect"
            },
            axis.is_some(),
        );
        let mut dir = cfg.direction;
        ui.add_enabled_ui(axis.is_some(), |ui| {
            ui.horizontal(|ui| {
                if ui.add(chip_button(first, dir == 0)).clicked() {
                    dir = 0;
                }
                if ui.add(chip_button(second, dir == 1)).clicked() {
                    dir = 1;
                }
            });
        });
        if dir != cfg.direction {
            self.profile_store.active_profile_mut().direction = dir;
            self.apply_to_hardware();
        }
    }

    fn render_color_column(&mut self, ui: &mut egui::Ui) {
        let cfg = self.current_config();
        let painting = !self.keyboard.selected_keys.is_empty();
        section_label(
            ui,
            if painting {
                "PAINT SELECTED KEYS"
            } else {
                "EFFECT COLOR"
            },
        );

        // Solid vs rainbow
        let can_random = cfg.mode.has_random() && !painting;
        ui.add_enabled_ui(can_random, |ui| {
            ui.horizontal(|ui| {
                let solid = !cfg.multicolor || !can_random;
                if ui.add(chip_button("Solid Color", solid)).clicked() && !solid {
                    self.profile_store.active_profile_mut().multicolor = false;
                    self.apply_to_hardware();
                }
                if ui.add(chip_button("Rainbow", !solid)).clicked() && solid {
                    self.profile_store.active_profile_mut().multicolor = true;
                    self.apply_to_hardware();
                }
            });
        });

        let hint = if painting {
            None
        } else if cfg.mode == LightingMode::Custom {
            Some("Keys without their own color use this color.")
        } else if cfg.mode == LightingMode::Surmount {
            Some("Surmount uses the closest of red, yellow, green or cyan.")
        } else if !cfg.mode.has_color() && cfg.mode != LightingMode::Off {
            Some("This effect uses built-in colors; picking one switches to Normal.")
        } else {
            None
        };
        if let Some(hint) = hint {
            ui.add_space(4.0);
            ui.label(egui::RichText::new(hint).color(KREO_TEXT_MUTED).size(10.5));
        }

        ui.add_space(10.0);

        // 14 Preset Color Boxes in 2 Rows of 7
        let active = if painting {
            self.current_color()
        } else {
            self.base_color()
        };
        let mut picked = None;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
            for (i, (name, color)) in OFFICIAL_PRESETS.iter().enumerate() {
                let (rect, response) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::click());
                ui.painter().rect_filled(
                    rect,
                    CornerRadius::same(6),
                    Color32::from_rgb(color.r, color.g, color.b),
                );
                let stroke = if active == *color {
                    Stroke::new(2.0_f32, Color32::WHITE)
                } else if response.hovered() {
                    Stroke::new(1.5_f32, Color32::from_white_alpha(200))
                } else {
                    Stroke::new(1.0_f32, Color32::from_black_alpha(140))
                };
                ui.painter().rect_stroke(
                    rect,
                    CornerRadius::same(6),
                    stroke,
                    egui::StrokeKind::Inside,
                );
                if response.on_hover_text(*name).clicked() {
                    picked = Some(*color);
                }
                if i == 6 {
                    ui.end_row();
                }
            }
        });
        if let Some(color) = picked {
            self.set_color(color);
        }

        ui.add_space(10.0);

        // RGB inputs
        egui::Frame::new()
            .fill(Color32::from_rgb(20, 21, 28))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(42, 44, 58)))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::symmetric(10, 6))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(5.0, 0.0);
                    let mut changed = false;
                    for (label, color, val) in [
                        ("R", Color32::from_rgb(240, 110, 110), &mut self.r_val),
                        ("G", Color32::from_rgb(110, 230, 140), &mut self.g_val),
                        ("B", Color32::from_rgb(120, 160, 255), &mut self.b_val),
                    ] {
                        ui.label(egui::RichText::new(label).color(color).size(11.5).strong());
                        changed |= ui
                            .add_sized(
                                [38.0, 20.0],
                                egui::DragValue::new(val).range(0..=255).speed(1.0),
                            )
                            .changed();
                        ui.add_space(4.0);
                    }
                    ui.label(
                        egui::RichText::new(self.current_color().hex())
                            .color(Color32::from_rgb(180, 185, 200))
                            .monospace()
                            .size(11.5),
                    );
                    if changed {
                        self.set_color(self.current_color());
                    }
                });
            });
    }

    fn render_picker_column(&mut self, ui: &mut egui::Ui) {
        section_label(ui, "SPECTRUM & PREVIEW");

        let spec_w = ui.available_width().min(300.0);
        if let Some(color) = spectrum_picker(ui, spec_w) {
            self.set_color(color);
        }

        ui.add_space(8.0);

        // Preview of the color that the next pick applies
        let col = self.current_color();
        let (p_rect, _) = ui.allocate_exact_size(Vec2::new(spec_w, 26.0), Sense::hover());
        ui.painter().rect_filled(
            p_rect,
            CornerRadius::same(5),
            Color32::from_rgb(col.r, col.g, col.b),
        );
        ui.painter().rect_stroke(
            p_rect,
            CornerRadius::same(5),
            Stroke::new(1.0_f32, Color32::from_rgb(44, 46, 60)),
            egui::StrokeKind::Inside,
        );
        let lum = 0.299 * col.r as f32 + 0.587 * col.g as f32 + 0.114 * col.b as f32;
        ui.painter().text(
            p_rect.center(),
            egui::Align2::CENTER_CENTER,
            col.hex(),
            egui::FontId::monospace(12.0),
            if lum > 140.0 {
                Color32::BLACK
            } else {
                Color32::WHITE
            },
        );

        ui.add_space(10.0);
        ui.label(
            egui::RichText::new(describe(&self.current_config()))
                .color(Color32::from_rgb(200, 202, 215))
                .size(11.0),
        );
    }

    fn render_settings_view(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Settings & Device Status").color(KREO_TEXT_WHITE));
        ui.add_space(12.0);

        egui::Frame::new()
            .fill(KREO_PANEL_BG)
            .stroke(Stroke::new(1.0_f32, KREO_BORDER))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::same(16))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new(format!(
                        "Hardware: {} (Status: {})",
                        self.keyboard.variant.name(),
                        if self.device_connected {
                            "Online"
                        } else {
                            "Offline / Virtual"
                        }
                    ))
                    .color(KREO_TEXT_WHITE),
                );
                ui.label(
                    egui::RichText::new(
                        "Interface: Linux hidraw userspace driver (USB interface 1)",
                    )
                    .color(KREO_TEXT_MUTED),
                );
                ui.add_space(8.0);

                if ui.button("Restore Default Lighting").clicked() {
                    self.set_color(Rgb::new(0, 255, 136));
                }
            });
    }
}

/// A compact selectable pill button used for modes, toggles and quick selects.
#[cfg(feature = "gui")]
fn chip_button(label: &str, active: bool) -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new(label.to_string())
            .color(if active {
                Color32::WHITE
            } else {
                Color32::from_rgb(215, 218, 230)
            })
            .size(11.5),
    )
    .fill(if active {
        KREO_PURPLE_ACTIVE
    } else {
        Color32::from_rgb(26, 28, 38)
    })
    .stroke(Stroke::new(
        1.0_f32,
        if active {
            Color32::from_rgb(150, 120, 255)
        } else {
            Color32::from_rgb(46, 48, 64)
        },
    ))
    .corner_radius(CornerRadius::same(5))
    .min_size(Vec2::new(0.0, 24.0))
}

/// Three-line menu button drawn with the painter (the default font has no ☰ glyph).
#[cfg(feature = "gui")]
fn menu_button(ui: &mut egui::Ui, open: bool) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(30.0, 26.0), Sense::click());
    let bg = if open || resp.hovered() {
        Color32::from_rgb(34, 36, 48)
    } else {
        Color32::from_rgb(26, 28, 38)
    };
    let border = if open {
        KREO_PURPLE_ACTIVE
    } else {
        KREO_BORDER
    };
    ui.painter().rect_filled(rect, CornerRadius::same(5), bg);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(5),
        Stroke::new(1.0_f32, border),
        egui::StrokeKind::Inside,
    );
    let line = Stroke::new(1.6_f32, KREO_TEXT_WHITE);
    for dy in [-5.0, 0.0, 5.0] {
        let y = rect.center().y + dy;
        ui.painter().line_segment(
            [
                Pos2::new(rect.left() + 9.0, y),
                Pos2::new(rect.right() - 9.0, y),
            ],
            line,
        );
    }
    resp
}

#[cfg(feature = "gui")]
fn section_label(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .color(KREO_TEXT_MUTED)
            .size(11.0)
            .strong(),
    );
    ui.add_space(6.0);
}

#[cfg(feature = "gui")]
fn slider_header(ui: &mut egui::Ui, title: &str, value: &str, enabled: bool) {
    ui.horizontal(|ui| {
        let title_color = if enabled {
            Color32::from_rgb(210, 212, 225)
        } else {
            Color32::from_rgb(90, 94, 110)
        };
        ui.label(egui::RichText::new(title).color(title_color).size(12.0));
        ui.label(egui::RichText::new(value).color(KREO_TEXT_MUTED).size(11.0));
    });
    ui.add_space(2.0);
}

/// Draws a white → hue → black spectrum and returns the color under the pointer when picked.
#[cfg(feature = "gui")]
fn spectrum_picker(ui: &mut egui::Ui, width: f32) -> Option<Rgb> {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 90.0), Sense::click_and_drag());

    let cols = 24_u32;
    let mut mesh = egui::Mesh::default();
    let rows = [rect.top(), rect.center().y, rect.bottom()];
    for (row, &y) in rows.iter().enumerate() {
        for ix in 0..=cols {
            let frac = ix as f32 / cols as f32;
            let color = match row {
                0 => Color32::WHITE,
                1 => {
                    let rgb = animation::rainbow_hue(frac);
                    Color32::from_rgb(rgb.r, rgb.g, rgb.b)
                }
                _ => Color32::BLACK,
            };
            mesh.vertices.push(egui::epaint::Vertex {
                pos: Pos2::new(rect.left() + rect.width() * frac, y),
                uv: egui::epaint::WHITE_UV,
                color,
            });
        }
    }
    let stride = cols + 1;
    for band in 0..2 {
        for ix in 0..cols {
            let tl = band * stride + ix;
            let bl = tl + stride;
            mesh.add_triangle(tl, tl + 1, bl);
            mesh.add_triangle(tl + 1, bl + 1, bl);
        }
    }
    ui.painter().add(egui::Shape::mesh(mesh));
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(5),
        Stroke::new(1.0_f32, Color32::from_rgb(44, 46, 60)),
        egui::StrokeKind::Inside,
    );

    // Only report on release or drag so a drag does not flood the keyboard.
    if !(response.clicked() || response.dragged()) {
        return None;
    }
    let pos = response.interact_pointer_pos()?;
    let nx = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
    let ny = ((pos.y - rect.top()) / rect.height()).clamp(0.0, 1.0);
    let hue = animation::rainbow_hue(nx);
    let mix = |c: u8| -> u8 {
        let c = c as f32;
        let v = if ny < 0.5 {
            let f = ny * 2.0;
            255.0 * (1.0 - f) + c * f
        } else {
            c * (1.0 - (ny - 0.5) * 2.0)
        };
        v.round() as u8
    };
    Some(Rgb::new(mix(hue.r), mix(hue.g), mix(hue.b)))
}

/// Renders the white stylized Kreo logo: "K" with a dot underneath.
#[cfg(feature = "gui")]
fn render_kreo_logo(ui: &mut egui::Ui, variant: KeyboardVariant) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(26.0, 26.0), Sense::hover());
        let center = rect.center();

        let p = ui.painter();
        let white = Stroke::new(2.6_f32, Color32::WHITE);
        // Vertical stem with curved top
        p.line_segment(
            [
                Pos2::new(center.x - 5.0, center.y - 9.0),
                Pos2::new(center.x - 5.0, center.y + 5.0),
            ],
            white,
        );
        // Upper diagonal arm
        p.line_segment(
            [
                Pos2::new(center.x - 5.0, center.y - 2.0),
                Pos2::new(center.x + 6.0, center.y - 8.0),
            ],
            white,
        );
        // Lower diagonal leg
        p.line_segment(
            [
                Pos2::new(center.x - 2.0, center.y - 2.0),
                Pos2::new(center.x + 6.0, center.y + 5.0),
            ],
            white,
        );
        // Dot below
        p.circle_filled(Pos2::new(center.x, center.y + 9.0), 2.0, Color32::WHITE);

        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(format!("KREO {}", variant.short_name()))
                .color(Color32::WHITE)
                .size(13.5)
                .strong(),
        );
        ui.label(
            egui::RichText::new("• MECHANICAL RGB")
                .color(KREO_TEXT_MUTED)
                .size(10.5),
        );
    });
}

/// Renders the interactive mechanical keyboard with per-key RGB and rotary knob (75% or 65%).
#[cfg(feature = "gui")]
fn render_keyboard(
    ui: &mut egui::Ui,
    keyboard: &mut KeyboardModel,
    cfg: &LightingConfig,
    board_w: f32,
) {
    let pad = 18.0_f32;
    let inner_w = board_w - pad * 2.0;
    let key_gap = (inner_w * 0.0055).clamp(3.5, 6.0);
    let u_size = (inner_w - 15.0 * key_gap) / 16.0;
    let total_rows = keyboard.variant.total_rows();
    let board_h = total_rows as f32 * u_size + (total_rows - 1) as f32 * key_gap + pad * 2.0;

    let (board_rect, _) = ui.allocate_exact_size(Vec2::new(board_w, board_h), Sense::hover());

    // Keyboard Outer Aluminum/Plastic Case Bezel
    ui.painter().rect_filled(
        board_rect,
        CornerRadius::same(12),
        Color32::from_rgb(22, 23, 30),
    );
    ui.painter().rect_stroke(
        board_rect,
        CornerRadius::same(12),
        Stroke::new(1.5_f32, Color32::from_rgb(44, 46, 60)),
        egui::StrokeKind::Inside,
    );

    // Pass 1: lay out every key
    let mut row_offsets = [board_rect.left() + pad; 6];
    let row_y_base = board_rect.top() + pad;
    let knob_w = u_size * 1.25;
    let rects: Vec<Rect> = keyboard
        .keys
        .iter()
        .map(|key_def| {
            let row = key_def.row;
            let w = if key_def.code == "Knob" {
                knob_w
            } else {
                key_def.width * u_size + (key_def.width - 1.0).max(0.0) * key_gap
            };
            let y = row_y_base + row as f32 * (u_size + key_gap);
            let x = match (keyboard.variant, key_def.code) {
                (_, "Knob") => board_rect.right() - pad - knob_w,
                (KeyboardVariant::Hive75, "Del") => {
                    board_rect.right() - pad - knob_w - key_gap - u_size
                }
                (KeyboardVariant::Hive65, "Del" | "PgUp" | "PgDn")
                | (KeyboardVariant::Hive75, "Ins" | "End" | "PgUp" | "PgDn")
                | (_, "Right") => board_rect.right() - pad - u_size,
                (_, "Up" | "Down") => board_rect.right() - pad - u_size * 2.0 - key_gap,
                (_, "Left") => board_rect.right() - pad - u_size * 3.0 - key_gap * 2.0,
                _ => {
                    let curr_x = row_offsets[row];
                    row_offsets[row] += w + key_gap;
                    curr_x
                }
            };
            Rect::from_min_size(Pos2::new(x, y), Vec2::new(w, u_size))
        })
        .collect();

    // Key centres in key units, as the effect preview expects
    let unit = u_size + key_gap;
    let to_units = |p: Pos2| {
        (
            (p.x - board_rect.left() - pad + key_gap / 2.0) / unit,
            (p.y - board_rect.top() - pad + key_gap / 2.0) / unit,
        )
    };
    // Scale by brightness (0 = off .. 4 = max), like the firmware
    let b_factor = cfg.brightness.min(BRIGHTNESS_MAX) as f32 / BRIGHTNESS_MAX as f32;

    // Pass 2: paint
    let mut clicked_code: Option<&'static str> = None;
    for (key_def, &key_rect) in keyboard.keys.iter().zip(&rects) {
        if key_def.code == "Knob" {
            // Draw CNC Rotary Knob
            let knob_center = key_rect.center();
            let radius_outer = (u_size * 0.44).clamp(14.0, 28.0);
            let radius_inner = radius_outer * 0.8;

            ui.painter()
                .circle_filled(knob_center, radius_outer, Color32::from_rgb(52, 54, 68));
            ui.painter().circle_stroke(
                knob_center,
                radius_outer,
                Stroke::new(1.0_f32, Color32::from_rgb(70, 74, 94)),
            );
            ui.painter()
                .circle_filled(knob_center, radius_inner, Color32::from_rgb(30, 32, 42));

            let notch_pos = Pos2::new(
                knob_center.x + radius_inner * 0.6,
                knob_center.y - radius_inner * 0.4,
            );
            ui.painter().circle_filled(notch_pos, 2.2, Color32::WHITE);
            continue;
        }

        let per_key = keyboard.get_key_color(key_def.code, cfg.color);
        let key_rgb = match cfg.mode {
            LightingMode::Custom => per_key,
            LightingMode::Off => Rgb::new(0, 0, 0),
            _ if cfg.multicolor => animation::rainbow_hue(to_units(key_rect.center()).0 / 16.0),
            _ => cfg.color,
        };
        let final_r = (key_rgb.r as f32 * b_factor).round() as u8;
        let final_g = (key_rgb.g as f32 * b_factor).round() as u8;
        let final_b = (key_rgb.b as f32 * b_factor).round() as u8;

        let is_selected = keyboard.is_selected(key_def.code);

        // Underglow light
        let glow_alpha = (b_factor * 110.0).round() as u8;
        ui.painter().rect_filled(
            key_rect,
            CornerRadius::same(5),
            Color32::from_rgba_premultiplied(final_r / 2, final_g / 2, final_b / 2, glow_alpha),
        );

        // Keycap Body
        let shrink_amt = (u_size * 0.04).clamp(1.5, 3.0);
        let keycap_rect = key_rect.shrink(shrink_amt);
        let keycap_bg = if is_selected {
            Color32::from_rgb(50, 40, 80)
        } else {
            Color32::from_rgb(26, 28, 36)
        };
        ui.painter()
            .rect_filled(keycap_rect, CornerRadius::same(4), keycap_bg);

        // Selection Stroke or Keycap Edge
        let stroke = if is_selected {
            Stroke::new(1.8_f32, KREO_PURPLE_ACTIVE)
        } else {
            Stroke::new(1.0_f32, Color32::from_rgb(42, 44, 56))
        };
        ui.painter().rect_stroke(
            keycap_rect,
            CornerRadius::same(4),
            stroke,
            egui::StrokeKind::Inside,
        );

        // Legend Text with RGB tint
        let font_sz = (u_size * 0.28).clamp(9.5, 14.0);
        let text_color = Color32::from_rgb(final_r.max(170), final_g.max(170), final_b.max(170));
        ui.painter().text(
            keycap_rect.center(),
            egui::Align2::CENTER_CENTER,
            key_def.label,
            egui::FontId::proportional(font_sz),
            text_color,
        );

        // Interactivity: Click to toggle selection
        let resp = ui.interact(key_rect, ui.id().with(key_def.id), Sense::click());
        if resp.clicked() {
            clicked_code = Some(key_def.code);
        }
    }

    if let Some(code) = clicked_code {
        keyboard.toggle_selection(code);
    }
}

#[cfg(feature = "gui")]
pub fn run() -> Result<(), KreoError> {
    let options = eframe::NativeOptions {
        depth_buffer: 0,
        multisampling: 0,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 840.0])
            .with_min_inner_size([960.0, 640.0])
            .with_title("Kreo Hive RGB Controller"),
        ..Default::default()
    };
    eframe::run_native(
        "kreoctl",
        options,
        Box::new(|cc| {
            let mut style = (*cc.egui_ctx.style()).clone();
            style.visuals.dark_mode = true;
            style.visuals.panel_fill = KREO_BG;
            style.visuals.window_fill = KREO_BG;
            style.visuals.faint_bg_color = KREO_PANEL_BG;
            style.visuals.selection.bg_fill = KREO_PURPLE_ACTIVE;
            cc.egui_ctx.set_style(style);
            Ok(Box::new(KreoApp::new(&cc.egui_ctx)))
        }),
    )
    .map_err(|e| KreoError::Gui(e.to_string()))
}

#[cfg(not(feature = "gui"))]
pub fn run() -> Result<(), crate::error::KreoError> {
    Err(crate::error::KreoError::GuiNotCompiled)
}
