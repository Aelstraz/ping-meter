use std::sync::Arc;

use eframe::egui::{self, Color32, Vec2b};
use egui_plot::{Plot, PlotPoints, uniform_grid_spacer};

use crate::{PingData, Settings};

#[derive(Default)]
pub struct MainMenu {}

impl MainMenu {
    pub fn update(&mut self, ui: &mut egui::Ui, settings: &mut Settings, ping_data: &PingData) {
        let ping_label;
        let label_color;

        if ping_data.plot_points.is_empty() {
            ping_label = String::from("Waiting...");
            label_color = Color32::WHITE;
        } else {
            let plot_point = ping_data.plot_points.last().unwrap();
            if plot_point.y < 0.0 {
                label_color = Color32::WHITE;
                ping_label = String::from("Error");
            } else {
                label_color = Color32::WHITE;
                ping_label = format!("{}ms", plot_point.y);
            }
        }

        ui.group(|group| {
            group.colored_label(label_color, format!("Address: {}", &settings.ping_address));
            group.colored_label(label_color, format!("Ping: {}", ping_label));
            group.colored_label(
                label_color,
                format!("Variation: {}ms", &ping_data.ping_variation),
            );
            group.colored_label(
                label_color,
                format!("Packet Loss: {:.2}%", &ping_data.packet_loss_percentage),
            );
        });

        self.draw_graph(ui, settings, ping_data);
    }

    fn draw_graph(&mut self, ui: &mut egui::Ui, settings: &Settings, ping_data: &PingData) {
        ui.group(|group| {
            let base_spacer = uniform_grid_spacer(|input| {
                let base = if input.base_step_size < 1.0 {
                    1.0
                } else {
                    input.base_step_size
                };
                [base * 10.0, base * 5.0, base]
            });

            let plot = Plot::new("ping-plot")
                .view_aspect(settings.plot_aspect_ratio)
                .allow_zoom(false)
                .allow_drag(false)
                .allow_scroll(false)
                .allow_axis_zoom_drag(false)
                .show_grid(false)
                .show_crosshair(false)
                .show_axes(Vec2b::new(false, true))
                .y_grid_spacer(move |val| base_spacer(val))
                .y_axis_formatter(|val, _range| format!("{:.0}", val.value));

            plot.show(group, |plot| {
                let points =
                    egui_plot::Points::new("Ping", PlotPoints::Borrowed(&ping_data.plot_points))
                        .radius(4.0)
                        .color(Color32::BLUE);
                let line =
                    egui_plot::Line::new("Ping", PlotPoints::Borrowed(&ping_data.plot_points))
                        .gradient_color(
                            Arc::new(|p| {
                                if p.y < 0.0 {
                                    Color32::RED
                                } else {
                                    Color32::BLUE
                                }
                            }),
                            false,
                        );

                plot.line(line);
                plot.points(points);
            });
        });
    }
}
