use std::{path::{PathBuf, Path}, env::var, fs::{OpenOptions, File}, io::Write};

use eframe::egui;
use egui::TextureHandle;
use serde::{Deserialize, Serialize};
use egui_extras::image;

use predator_ng::widgets::{toggle::*, color_box::*};

#[derive(PartialEq, Deserialize, Serialize, Default, Copy, Clone)]
enum KBLightMode {
    #[default] Static,
    Dynamic
}

#[derive(PartialEq, Deserialize, Serialize, Default, Copy, Clone)]
enum KBDynamicEffect {
    #[default] Breathing = 1,
    Neon,
    Wave,
    Shifting,
    Zoom,
    Meteor,
    Twinkling
}

#[derive(PartialEq, Deserialize, Serialize, Copy, Clone, Default)]
enum KBDynamicDirection {
    None,
    #[default] LeftToRight,
    RightToLeft
}

#[non_exhaustive]
struct PresetDynamicColor;

impl PresetDynamicColor {
    pub const RED: [u8; 3] = [255, 0, 0];
    pub const ORANGE: [u8; 3] = [255, 165, 0];
    pub const YELLOW: [u8; 3] = [255, 255, 0];
    pub const GREEN: [u8; 3] = [0, 128, 0];
    pub const BLUE: [u8; 3] = [0, 0, 255];
    pub const INDIGO: [u8; 3] = [75, 0, 130];
    pub const VIOLET: [u8; 3] = [148, 0, 211];
    pub const WHITE: [u8; 3] = [255, 255, 255];
}

#[derive(Serialize, Deserialize, Copy, Clone)]
struct Zone {
    color: [u8; 3],
    enabled: bool
}

#[derive(Serialize, Deserialize, Copy, Clone)]
struct KBLighting {
    mode: KBLightMode,
    brightness: u8,
    effect: KBDynamicEffect,
    speed: u8,
    direction: KBDynamicDirection,
    color: [u8; 3],
    zones: [Zone; 3]
}

impl Default for KBLighting {
    fn default() -> Self {
        Self {
            mode: Default::default(),
            brightness: 100,
            direction: Default::default(),
            effect: Default::default(),
            speed: 5,
            color: PresetDynamicColor::WHITE,
            zones: [
                Zone {
                    color: [255, 255, 255],
                    enabled: true
                },
                Zone {
                    color: [255, 255, 255],
                    enabled: true
                },
                Zone {
                    color: [255, 255, 255],
                    enabled: true
                }
            ]
        }
    }
}

#[derive(Default, Serialize, Deserialize, Copy, Clone)]
struct Config {
    kb: KBLighting,
}

fn update_dynamic(dynamic_dev: &mut File, cfg: Config) {
    let mut dynamic_data: [u8; 16] = [0; 16];
    dynamic_data[0] = cfg.kb.effect as u8;
    dynamic_data[1] = cfg.kb.speed;
    dynamic_data[2] = cfg.kb.brightness;
    dynamic_data[4] = cfg.kb.direction as u8;
    dynamic_data[5] = cfg.kb.color[0];
    dynamic_data[6] = cfg.kb.color[1];
    dynamic_data[7] = cfg.kb.color[2];
    dynamic_data[9] = 1;

    dynamic_dev.write(&dynamic_data).expect("Failed to write to dynamic device");
}

fn change_brightness(dynamic_dev: &mut File, cfg: Config) {
    let mut dynamic_data: [u8; 16] = [0; 16];
    dynamic_data[0] = if cfg.kb.mode == KBLightMode::Static {0} else {cfg.kb.effect as u8};
    dynamic_data[1] = if cfg.kb.mode == KBLightMode::Static {0} else {cfg.kb.speed};
    dynamic_data[2] = cfg.kb.brightness;
    dynamic_data[4] = if cfg.kb.mode == KBLightMode::Static {0} else {cfg.kb.direction as u8};
    dynamic_data[5] = if cfg.kb.mode == KBLightMode::Static {0} else {cfg.kb.color[0]};
    dynamic_data[6] = if cfg.kb.mode == KBLightMode::Static {0} else {cfg.kb.color[1]};
    dynamic_data[7] = if cfg.kb.mode == KBLightMode::Static {0} else {cfg.kb.color[2]};
    dynamic_data[9] = 1;

    dynamic_dev.write(&dynamic_data).expect("Failed to write to dynamic device");
}

fn switch_to_static(static_dev: &mut File, dynamic_dev: &mut File, cfg: Config) {
    for zone in 1..4 {
        write_to_static_dev(static_dev, cfg, zone);

        let mut dynamic_data: [u8; 16] = [0; 16];
        dynamic_data[2] = cfg.kb.brightness;
        dynamic_data[9] = 1; // This is needed for PT314-52s and possibly other models

        dynamic_dev.write(&dynamic_data).expect("Failed to write to static device");
    }
}

fn write_to_static_dev(static_dev: &mut File, cfg: Config, zone: usize) {
    let red = cfg.kb.zones[zone - 1].color[0];
    let green = cfg.kb.zones[zone - 1].color[1];
    let blue = cfg.kb.zones[zone - 1].color[2];

    let static_data: [u8; 8] = [
        0,
        1 << (zone - 1),
        red,
        green,
        blue,
        cfg.kb.zones[0].enabled as u8,
        cfg.kb.zones[1].enabled as u8,
        cfg.kb.zones[2].enabled as u8
    ];
    static_dev.write(&static_data).expect("Failed to write to static device");
}

fn toggle_zone(static_dev: &mut File, cfg: Config, zone_num: usize) {
    let zone = &cfg.kb.zones[zone_num - 1];
    let static_data = [
        1, // 0 for setting color, 1 for toggling zones
        zone_num as u8, // Zone number
        zone.color[0], // R
        zone.color[1], // G
        zone.color[2], // B
        cfg.kb.zones[0].enabled as u8, // Zone 1 enabled
        cfg.kb.zones[1].enabled as u8, // Zone 2 enabled
        cfg.kb.zones[2].enabled as u8 // Zone 3 enabled
    ];

    static_dev.write(&static_data).expect("Failed to write to static device");
}

fn show_dynamic_kb_lighting_pane(ui: &mut egui::Ui, dynamic_dev: &mut File, cfg: &mut Config, config_path: PathBuf) {
    ui.label("Light Effects");
    egui::Grid::new("Effects")
        .min_col_width(50.0)
        .min_row_height(30.0)
        .show(ui, |ui| {
            if ui.radio_value(&mut cfg.kb.effect, KBDynamicEffect::Breathing, "Breathing").changed() {
                update_dynamic(dynamic_dev, *cfg);
                let _ = confy::store_path(config_path.clone(), &cfg);
            }
            if ui.radio_value(&mut cfg.kb.effect, KBDynamicEffect::Shifting, "Shifting").changed() {
                update_dynamic(dynamic_dev, *cfg);
                let _ = confy::store_path(config_path.clone(), &cfg);
            }
            if ui.radio_value(&mut cfg.kb.effect, KBDynamicEffect::Twinkling, "Twinkling").changed() {
                update_dynamic(dynamic_dev, *cfg);
                let _ = confy::store_path(config_path.clone(), &cfg);
            }
            ui.end_row();
            if ui.radio_value(&mut cfg.kb.effect, KBDynamicEffect::Neon, "Neon").changed() {
                update_dynamic(dynamic_dev, *cfg);
                let _ = confy::store_path(config_path.clone(), &cfg);
            }
            if ui.radio_value(&mut cfg.kb.effect, KBDynamicEffect::Zoom, "Zoom").changed() {
                update_dynamic(dynamic_dev, *cfg);
                let _ = confy::store_path(config_path.clone(), &cfg);
            }
            ui.end_row();
            if ui.radio_value(&mut cfg.kb.effect, KBDynamicEffect::Wave, "Wave").changed() {
                update_dynamic(dynamic_dev, *cfg);
                let _ = confy::store_path(config_path.clone(), &cfg);
            }
            if ui.radio_value(&mut cfg.kb.effect, KBDynamicEffect::Meteor, "Meteor").changed() {
                update_dynamic(dynamic_dev, *cfg);
                let _ = confy::store_path(config_path.clone(), &cfg);
            }
            ui.end_row();
        });
    ui.label("Speed");
    if ui.add(egui::Slider::new(&mut cfg.kb.speed, 1..=9)).changed() {
        update_dynamic(dynamic_dev, *cfg);
        let _ = confy::store_path(config_path.clone(), &cfg);
    }
    ui.add_space(10.0);

    ui.label("Direction");
    if ui.radio_value(&mut cfg.kb.direction, KBDynamicDirection::LeftToRight, "Left to Right").changed() {
        update_dynamic(dynamic_dev, *cfg);
        let _ = confy::store_path(config_path.clone(), &cfg);
    }
    if ui.radio_value(&mut cfg.kb.direction, KBDynamicDirection::RightToLeft, "Right to Left").changed() {
        update_dynamic(dynamic_dev, *cfg);
        let _ = confy::store_path(config_path.clone(), &cfg);
    }
    ui.add_space(10.0);

    ui.label("Color");
    ui.horizontal(|ui| {
        if ui.add(color_box(&mut cfg.kb.color, PresetDynamicColor::RED)).changed() {
            update_dynamic(dynamic_dev, *cfg);
            let _ = confy::store_path(config_path.clone(), &cfg);
        }
        if ui.add(color_box(&mut cfg.kb.color, PresetDynamicColor::ORANGE)).changed() {
            update_dynamic(dynamic_dev, *cfg);
            let _ = confy::store_path(config_path.clone(), &cfg);
        }
        if ui.add(color_box(&mut cfg.kb.color, PresetDynamicColor::YELLOW)).changed() {
            update_dynamic(dynamic_dev, *cfg);
            let _ = confy::store_path(config_path.clone(), &cfg);
        }
        if ui.add(color_box(&mut cfg.kb.color, PresetDynamicColor::GREEN)).changed() {
            update_dynamic(dynamic_dev, *cfg);
            let _ = confy::store_path(config_path.clone(), &cfg);
        }
        if ui.add(color_box(&mut cfg.kb.color, PresetDynamicColor::BLUE)).changed() {
            update_dynamic(dynamic_dev, *cfg);
            let _ = confy::store_path(config_path.clone(), &cfg);
        }
        if ui.add(color_box(&mut cfg.kb.color, PresetDynamicColor::INDIGO)).changed() {
            update_dynamic(dynamic_dev, *cfg);
            let _ = confy::store_path(config_path.clone(), &cfg);
        }
        if ui.add(color_box(&mut cfg.kb.color, PresetDynamicColor::VIOLET)).changed() {
            update_dynamic(dynamic_dev, *cfg);
            let _ = confy::store_path(config_path.clone(), &cfg);
        }
        if ui.add(color_box(&mut cfg.kb.color, PresetDynamicColor::WHITE)).changed() {
            update_dynamic(dynamic_dev, *cfg);
            let _ = confy::store_path(config_path.clone(), &cfg);
        }
    });
    ui.vertical(|ui| {
        ui.label("Custom Color");
        if ui.color_edit_button_srgb(&mut cfg.kb.color).changed() {
            update_dynamic(dynamic_dev, *cfg);
            let _ = confy::store_path(config_path.clone(), &cfg);
        }
        ui.horizontal(|ui| {
            let r = ui.label("R");
            if ui.add(egui::DragValue::new(&mut cfg.kb.color[0])).labelled_by(r.id).changed() {
                update_dynamic(dynamic_dev, *cfg);
                let _ = confy::store_path(config_path.clone(), &cfg);
            }
            let g = ui.label("G");
            if ui.add(egui::DragValue::new(&mut cfg.kb.color[1])).labelled_by(g.id).changed() {
                update_dynamic(dynamic_dev, *cfg);
                let _ = confy::store_path(config_path.clone(), &cfg);
            }
            let b = ui.label("B");
            if ui.add(egui::DragValue::new(&mut cfg.kb.color[2])).labelled_by(b.id).changed() {
                update_dynamic(dynamic_dev, *cfg);
                let _ = confy::store_path(config_path.clone(), &cfg);
            }
        });
    });
}

fn show_static_kb_lighting_pane(ui: &mut egui::Ui, static_dev: &mut File, cfg: &mut Config, config_path: PathBuf, prohibit_tex: TextureHandle) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label("Zone 1");
            ui.horizontal_wrapped(|ui| {
                let mut picker_rect = ui.available_rect_before_wrap();

                if ui.add(toggle(&mut cfg.kb.zones[0].enabled)).changed() {
                    toggle_zone(static_dev, *cfg, 1);
                    let _ = confy::store_path(config_path.clone(), &cfg);
                }
                ui.add_enabled_ui(cfg.kb.zones[0].enabled, |ui| {
                    let picker = ui.color_edit_button_srgb(&mut cfg.kb.zones[0].color);
                    picker_rect = picker.rect;
                    if picker.changed() {
                        write_to_static_dev(static_dev, *cfg, 1);
                        let _ = confy::store_path(config_path.clone(), &cfg);
                    }
                });
                if !cfg.kb.zones[0].enabled {
                    let pos = [egui::pos2(picker_rect.max.x - 2.0, picker_rect.max.y - 7.0)];
                    ui.put(egui::Rect::from_points(&pos), egui::Image::new(&prohibit_tex).max_size(egui::Vec2::new(10.0, 10.0)));
                }
            });
            ui.add_space(10.0);
            ui.add_enabled_ui(cfg.kb.zones[0].enabled, |ui| {
                ui.horizontal(|ui| {
                    let r = ui.label("R");
                    if ui.add(egui::DragValue::new(&mut cfg.kb.zones[0].color[0])).labelled_by(r.id).changed() {
                        write_to_static_dev(static_dev, *cfg, 1);
                        let _ = confy::store_path(config_path.clone(), &cfg);
                    }
                    let g = ui.label("G");
                    if ui.add(egui::DragValue::new(&mut cfg.kb.zones[0].color[1])).labelled_by(g.id).changed() {
                        write_to_static_dev(static_dev, *cfg, 1);
                        let _ = confy::store_path(config_path.clone(), &cfg);
                    }
                    let b = ui.label("B");
                    if ui.add(egui::DragValue::new(&mut cfg.kb.zones[0].color[2])).labelled_by(b.id).changed() {
                        write_to_static_dev(static_dev, *cfg, 1);
                        let _ = confy::store_path(config_path.clone(), &cfg);
                    }
                });
            })
        });
        ui.separator();
        ui.vertical(|ui| {
            ui.label("Zone 2");
            ui.horizontal_wrapped(|ui| {
                let mut picker_rect = ui.available_rect_before_wrap();

                if ui.add(toggle(&mut cfg.kb.zones[1].enabled)).changed() {
                    toggle_zone(static_dev, *cfg, 2);
                    let _ = confy::store_path(config_path.clone(), &cfg);
                }
                ui.add_enabled_ui(cfg.kb.zones[1].enabled, |ui| {
                    let picker = ui.color_edit_button_srgb(&mut cfg.kb.zones[1].color);
                    picker_rect = picker.rect;
                    if picker.changed() {
                        write_to_static_dev(static_dev, *cfg, 2);
                        let _ = confy::store_path(config_path.clone(), &cfg);
                    }
                });
                if !cfg.kb.zones[1].enabled {
                    let pos = [egui::pos2(picker_rect.max.x - 2.0, picker_rect.max.y - 7.0)];
                    ui.put(egui::Rect::from_points(&pos), egui::Image::new(&prohibit_tex).max_size(egui::Vec2::new(10.0, 10.0)));
                }
            });
            ui.add_space(10.0);
            ui.add_enabled_ui(cfg.kb.zones[1].enabled, |ui| {
                ui.horizontal(|ui| {
                    let r = ui.label("R");
                    if ui.add(egui::DragValue::new(&mut cfg.kb.zones[1].color[0])).labelled_by(r.id).changed() {
                        write_to_static_dev(static_dev, *cfg, 2);
                        let _ = confy::store_path(config_path.clone(), &cfg);
                    }
                    let g = ui.label("G");
                    if ui.add(egui::DragValue::new(&mut cfg.kb.zones[1].color[1])).labelled_by(g.id).changed() {
                        write_to_static_dev(static_dev, *cfg, 2);
                        let _ = confy::store_path(config_path.clone(), &cfg);
                    }
                    let b = ui.label("B");
                    if ui.add(egui::DragValue::new(&mut cfg.kb.zones[1].color[2])).labelled_by(b.id).changed() {
                        write_to_static_dev(static_dev, *cfg, 2);
                        let _ = confy::store_path(config_path.clone(), &cfg);
                    }
                });
            });
        });
        ui.separator();
        ui.vertical(|ui| {
            ui.label("Zone 3");
            ui.horizontal_wrapped(|ui| {
                let mut picker_rect = ui.available_rect_before_wrap();

                if ui.add(toggle(&mut cfg.kb.zones[2].enabled)).changed() {
                    toggle_zone(static_dev, *cfg, 3);
                    let _ = confy::store_path(config_path.clone(), &cfg);
                }
                ui.add_enabled_ui(cfg.kb.zones[2].enabled, |ui| {
                    let picker = ui.color_edit_button_srgb(&mut cfg.kb.zones[2].color);
                    picker_rect = picker.rect;
                    if picker.changed() {
                        write_to_static_dev(static_dev, *cfg, 3);
                        let _ = confy::store_path(config_path.clone(), &cfg);
                    }
                });
                if !cfg.kb.zones[2].enabled {
                    let pos = [egui::pos2(picker_rect.max.x - 2.0, picker_rect.max.y - 7.0)];
                    ui.put(egui::Rect::from_points(&pos), egui::Image::new(&prohibit_tex).max_size(egui::Vec2::new(10.0, 10.0)));
                }
            });
            ui.add_space(10.0);
            ui.add_enabled_ui(cfg.kb.zones[2].enabled, |ui| {
                ui.horizontal(|ui| {
                    let r = ui.label("R");
                    if ui.add(egui::DragValue::new(&mut cfg.kb.zones[2].color[0])).labelled_by(r.id).changed() {
                        write_to_static_dev(static_dev, *cfg, 3);
                        let _ = confy::store_path(config_path.clone(), &cfg);
                    }
                    let g = ui.label("G");
                    if ui.add(egui::DragValue::new(&mut cfg.kb.zones[2].color[1])).labelled_by(g.id).changed() {
                        write_to_static_dev(static_dev, *cfg, 3);
                        let _ = confy::store_path(config_path.clone(), &cfg);
                    }
                    let b = ui.label("B");
                    if ui.add(egui::DragValue::new(&mut cfg.kb.zones[2].color[2])).labelled_by(b.id).changed() {
                        write_to_static_dev(static_dev, *cfg, 3);
                        let _ = confy::store_path(config_path.clone(), &cfg);
                    }
                });
            });
        });
    });
}

fn check_devices(options: &eframe::NativeOptions) -> Result<(File, File), String> {
    let static_dev = OpenOptions::new()
        .write(true)
        .create(false)
        .open("/dev/acer-gkbbl-static-0");

    let dynamic_dev = OpenOptions::new()
        .write(true)
        .create(false)
        .open("/dev/acer-gkbbl-0");

    if static_dev.is_err() || dynamic_dev.is_err() {
        eprintln!("[ERROR]: Could not open device files");

        let _ = eframe::run_simple_native("Predator-ng", options.clone(), move |ctx, _frame| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.colored_label(egui::Color32::RED, egui::RichText::new("Error: could not open device files").heading());
                ui.horizontal_wrapped(|ui| {
                    ui.label("Please make sure you have the kernel module loaded, and that both");
                    ui.label(egui::RichText::new("/dev/acer-gkbbl-0").code());
                    ui.label("and");
                    ui.label(egui::RichText::new("/dev/acer-gkbbl-static-0").code());
                    ui.label("exist");
                });
            });
        });

        return Err("Could not open device files".to_string());
    } else {
        return Ok((static_dev.unwrap(), dynamic_dev.unwrap()));
    }
}

fn initial_load(config_path: PathBuf, static_dev: &mut File, dynamic_dev: &mut File) -> Result<Config, confy::ConfyError> {
    let cfg: Config = confy::load_path(config_path.clone())?;

    if cfg.kb.mode == KBLightMode::Static {
        switch_to_static(static_dev, dynamic_dev, cfg);
    } else {
        update_dynamic(dynamic_dev, cfg);
    }

    Ok(cfg)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(640.0, 480.0)),
        min_window_size: Some(egui::vec2(620.0, 400.0)),
        ..Default::default()
    };

    match check_devices(&options) {
        Ok((mut static_dev, mut dynamic_dev)) => {
            let config_home = var("XDG_CONFIG_HOME")
                .or_else(|_| var("HOME").map(|home|format!("{}/.config", home))).unwrap();

            let config_path = Path::new(&config_home).join("predator-ng").to_path_buf();
            let mut cfg = initial_load(config_path.clone(), &mut static_dev, &mut dynamic_dev)?;

            let prohibit_svg_bytes = include_bytes!("../assets/prohibit.svg");

            let _ = eframe::run_simple_native("Predator-ng", options, move |ctx, _frame| {
                let prohibit_svg = image::load_svg_bytes(prohibit_svg_bytes).expect("Failed to load prohibit icon");
                let prohibit_tex = ctx.load_texture("prohibit", prohibit_svg, Default::default());

                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Keyboard Lighting Mode: ");
                        if ui.radio_value(&mut cfg.kb.mode, KBLightMode::Static, "Static").clicked() {
                            switch_to_static(&mut static_dev, &mut dynamic_dev, cfg);
                            let _ = confy::store_path(config_path.clone(), &cfg);
                        }
                        if ui.radio_value(&mut cfg.kb.mode, KBLightMode::Dynamic, "Dynamic").clicked() {
                            update_dynamic(&mut dynamic_dev, cfg);
                            let _ = confy::store_path(config_path.clone(), &cfg);
                        }
                        ui.label("Keyboard Brightness: ");
                        if ui.add(egui::Slider::new(&mut cfg.kb.brightness, 0..=100).show_value(false).step_by(25.0)).changed() {
                            change_brightness(&mut dynamic_dev, cfg);
                            let _ = confy::store_path(config_path.clone(), &cfg);
                        }
                    });
                    ui.add_space(15.0);
                    ui.group(|ui| {
                        match cfg.kb.mode {
                            KBLightMode::Static => {
                                show_static_kb_lighting_pane(ui, &mut static_dev, &mut cfg, config_path.clone(), prohibit_tex);
                            },
                            KBLightMode::Dynamic => {
                                show_dynamic_kb_lighting_pane(ui, &mut dynamic_dev, &mut cfg, config_path.clone());
                            }
                        }
                    });
                });
            });

            Ok(())
        }
        Err(_) => Ok(())
    }
}
