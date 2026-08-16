mod main_menu;
mod pinger;
mod settings;
mod settings_menu;

use std::{
    sync::mpsc::{self, RecvError, RecvTimeoutError, SendError, TryRecvError},
    thread,
    time::Duration,
};

use eframe::egui::{self};
use egui_plot::PlotPoint;

use crate::{main_menu::MainMenu, pinger::Pinger, settings::Settings, settings_menu::SettingsMenu};

fn main() {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([430.0, 535.0]),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Ping Meter",
        native_options,
        Box::new(|cc| {
            return Ok(Box::new(App::new(cc)));
        }),
    );
}

struct App {
    main_menu: MainMenu,
    settings_menu: SettingsMenu,
    current_menu: Menu,
    ping_channel: DualChannel<ChannelMessage>,
    ping_data: PingData,
    settings: Settings,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let settings = Settings::load();
        let settings_clone = settings.clone();

        let (ui_channel, ping_channel) = DualChannel::new();
        let ctx_clone = cc.egui_ctx.clone();

        thread::spawn(move || {
            let mut ping = Pinger::new(settings_clone);
            ping.start_loop(&ui_channel, &ctx_clone);
        });

        Self {
            main_menu: MainMenu::default(),
            settings_menu: SettingsMenu::default(),
            current_menu: Menu::Main,
            ping_channel,
            ping_data: PingData::default(),
            settings,
        }
    }

    fn set_menu(&mut self, menu: Menu) {
        if menu != self.current_menu {
            self.current_menu = menu;

            if menu == Menu::Settings {
                self.settings_menu.on_enabled(&self.settings)
            }
        }
    }

    fn calculate_ping_stats(&mut self) {
        let mut lost_packet_count = 0.0;
        let mut variation = 0.0;

        if self.ping_data.plot_points.len() > 0 {
            self.ping_data.plot_points[0].x = 0.0;
        }

        for i in 1..self.ping_data.plot_points.len() {
            self.ping_data.plot_points[i].x = i as f64;

            if self.ping_data.plot_points[i].y < 0.0 {
                lost_packet_count += 1.0;
                variation += self.settings.ping_timeout as f64 * 1000.0;
            } else {
                variation +=
                    (self.ping_data.plot_points[i].y - self.ping_data.plot_points[i - 1].y).abs();
            }
        }

        if self.ping_data.plot_points.len() > 0 {
            variation /= self.ping_data.plot_points.len() as f64;
            variation = variation.ceil();
            self.ping_data.ping_variation = variation;
            self.ping_data.packet_loss_percentage =
                (lost_packet_count / self.ping_data.plot_points.len() as f32) * 100.0;
        }
    }

    fn on_receive_channel_data(&mut self, data: &ChannelMessage) {
        match data {
            ChannelMessage::Update {
                ping_address: address,
                ping_result,
            } => {
                if address == &self.settings.ping_address {
                    let val = match ping_result {
                        Ok(duration) => *duration,
                        Err(_) => -1.0,
                    };

                    if self.ping_data.plot_points.len() > self.settings.max_plot_points {
                        self.ping_data.plot_points.remove(0);
                    }

                    self.ping_data.plot_points.push(PlotPoint::new(0.0, val));
                    self.calculate_ping_stats();
                }
            }
            _ => {}
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        match self.ping_channel.try_receive() {
            Ok(val) => {
                self.on_receive_channel_data(&val);
            }
            Err(_) => {}
        }

        egui::CentralPanel::default().show(ui, |ui| {
            let menu;
            let button_label;

            if self.current_menu == Menu::Main {
                menu = Menu::Settings;
                button_label = "Settings";
            } else {
                menu = Menu::Main;
                button_label = "Back";
            }

            if ui.button(button_label).clicked() {
                self.set_menu(menu);
            }

            match self.current_menu {
                Menu::Main => self
                    .main_menu
                    .update(ui, &mut self.settings, &self.ping_data),
                Menu::Settings => {
                    self.settings_menu
                        .update(ui, &mut self.settings, &mut self.ping_data)
                }
                _ => {}
            };
        });

        if self.settings.update {
            let _ = self.ping_channel.send(ChannelMessage::Settings {
                settings: self.settings.clone(),
            });
            self.settings.update = false;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Menu {
    Main,
    Settings,
}

#[derive(Default)]
pub struct PingData {
    ping_variation: f64,
    packet_loss_percentage: f32,
    plot_points: Vec<PlotPoint>,
}

pub enum ChannelMessage {
    Settings {
        settings: Settings,
    },
    Update {
        ping_address: String,
        ping_result: Result<f64, ()>,
    },
}

struct DualChannel<T> {
    sender: mpsc::SyncSender<T>,
    receiver: mpsc::Receiver<T>,
}

impl<T> DualChannel<T> {
    pub fn new() -> (Self, Self) {
        let (sender_a, receiver_a) = mpsc::sync_channel(3);
        let (sender_b, receiver_b) = mpsc::sync_channel(3);

        (
            Self {
                sender: sender_a,
                receiver: receiver_b,
            },
            Self {
                sender: sender_b,
                receiver: receiver_a,
            },
        )
    }

    pub fn send(&self, data: T) -> Result<(), SendError<T>> {
        self.sender.send(data)
    }

    pub fn receive(&self) -> Result<T, RecvError> {
        self.receiver.recv()
    }

    pub fn receive_timeout(&self, timeout: Duration) -> Result<T, RecvTimeoutError> {
        self.receiver.recv_timeout(timeout)
    }

    pub fn try_receive(&self) -> Result<T, TryRecvError> {
        self.receiver.try_recv()
    }
}
