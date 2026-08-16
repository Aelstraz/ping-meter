use eframe::egui::{self, Color32};

use crate::{PingData, Settings};

#[derive(Default)]
pub struct SettingsMenu {
    ping_interval: String,
    ping_timeout: String,
    ping_address: String,
    max_plot_points: String,
    plot_aspect_ratio: String,
    error: String,
}

impl SettingsMenu {
    pub fn on_enabled(&mut self, settings: &Settings) {
        self.ping_interval = settings.ping_interval.to_string();
        self.ping_timeout = settings.ping_timeout.to_string();
        self.ping_address = settings.ping_address.to_string();
        self.max_plot_points = settings.max_plot_points.to_string();
        self.plot_aspect_ratio = settings.plot_aspect_ratio.to_string();
        self.error = String::default();
    }

    pub fn update(&mut self, ui: &mut egui::Ui, settings: &mut Settings, ping_data: &mut PingData) {
        ui.group(|ui| {
            ui.set_max_width(400.0);
            ui.heading("Settings");

            ui.horizontal(|ui| {
                ui.label("Ping Address:");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    ui.text_edit_singleline(&mut self.ping_address);
                });
            });

            ui.horizontal(|ui| {
                ui.label("Ping Interval(sec):");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    ui.text_edit_singleline(&mut self.ping_interval);
                });
            });

            ui.horizontal(|ui| {
                ui.label("Ping Timeout(sec):");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    ui.text_edit_singleline(&mut self.ping_timeout);
                });
            });

            ui.horizontal(|ui| {
                ui.label("Max Plot Points:");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    ui.text_edit_singleline(&mut self.max_plot_points);
                });
            });

            ui.horizontal(|ui| {
                ui.label("Plot Aspect Ratio:");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    ui.text_edit_singleline(&mut self.plot_aspect_ratio);
                });
            });

            if !self.error.is_empty() {
                ui.colored_label(Color32::RED, format!("Error: {}", self.error));
            }

            if ui.button("Apply").clicked() {
                if self.parse_inputs(settings) {
                    self.error = String::default();
                    settings.save();
                    *ping_data = PingData::default();
                }
            }
        });

        ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));
        ui.label(format!("Created by {}", env!("CARGO_PKG_AUTHORS")));
    }

    fn parse_inputs(&mut self, settings: &mut Settings) -> bool {
        if !self.ping_address.is_empty() {
            settings.ping_address = self.ping_address.clone();
        } else {
            self.error = String::from("Invalid ping address!");
            return false;
        }

        match self.ping_interval.parse() {
            Ok(val) => {
                if val >= 0.1 {
                    settings.ping_interval = val;
                } else {
                    self.error =
                        format!("Invalid ping interval! Must be greater than or equal to 0.1");
                    return false;
                }
            }
            Err(err) => {
                self.error = format!("Invalid ping interval! {}", err);
                return false;
            }
        };

        match self.ping_timeout.parse() {
            Ok(val) => {
                if val > 0.0 {
                    settings.ping_timeout = val;
                } else {
                    self.error = format!("Invalid ping timeout! Must be greater than 0");
                    return false;
                }
            }
            Err(err) => {
                self.error = format!("Invalid ping timeout! {}", err);
                return false;
            }
        };

        match self.max_plot_points.parse() {
            Ok(val) => {
                if val >= 1 && val <= 100 {
                    settings.max_plot_points = val;
                } else {
                    self.error = format!("Invalid max plot points! Must be between 1 and 100");
                    return false;
                }
            }
            Err(err) => {
                self.error = format!("Invalid max plot points! {}", err);
                return false;
            }
        };

        match self.plot_aspect_ratio.parse() {
            Ok(val) => {
                if val > 0.0 {
                    settings.plot_aspect_ratio = val;
                } else {
                    self.error = format!("Invalid plot aspect ratio! Must be greater than 0");
                    return false;
                }
            }
            Err(err) => {
                self.error = format!("Invalid plot aspect ratio! {}", err);
                return false;
            }
        };

        return true;
    }
}
