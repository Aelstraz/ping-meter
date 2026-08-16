use std::{
    process::Command,
    time::{Duration, SystemTime},
};

use eframe::egui;

use crate::{ChannelMessage, DualChannel, Settings};

pub struct Pinger {
    settings: Settings,
}

impl Pinger {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    pub fn start_loop(&mut self, ui_channel: &DualChannel<ChannelMessage>, ctx: &egui::Context) {
        loop {
            let ping_duration = self.ping(ui_channel);
            ctx.request_repaint();

            let remaining_time = Duration::from_secs_f32(self.settings.ping_interval)
                .checked_sub(ping_duration)
                .unwrap_or_default();

            match ui_channel.receive_timeout(remaining_time) {
                Ok(val) => self.on_receive_channel_data(val),
                Err(_) => {}
            }
        }
    }

    fn ping(&mut self, ui_channel: &DualChannel<ChannelMessage>) -> Duration {
        // Configure flags based on OS
        let packet_count_flag;
        let timeout_flag;

        if cfg!(target_os = "windows") {
            packet_count_flag = format!("-n {}", 1);
            //windows uses milliseconds
            timeout_flag = format!("-w {}", self.settings.ping_timeout * 1000.0);
        } else {
            packet_count_flag = format!("-c {}", 1);
            timeout_flag = format!("-W {}", self.settings.ping_timeout);
        };

        let start_time = SystemTime::now();

        let output = Command::new("ping")
            .arg(packet_count_flag)
            .arg(timeout_flag)
            .arg(&self.settings.ping_address)
            .output();

        let duration;
        match output {
            Ok(out) => {
                if out.status.success() {
                    duration = SystemTime::now()
                        .duration_since(start_time)
                        .unwrap_or_default();
                    let _ = ui_channel.send(ChannelMessage::Update {
                        ping_address: self.settings.ping_address.clone(),
                        ping_result: Ok(duration.as_millis() as f64),
                    });
                } else {
                    duration = SystemTime::now()
                        .duration_since(start_time)
                        .unwrap_or_default();
                    let _ = ui_channel.send(ChannelMessage::Update {
                        ping_address: self.settings.ping_address.clone(),
                        ping_result: Err(()),
                    });
                }
            }
            Err(e) => {
                duration = Duration::default();
                eprintln!("Failed to ping: {}", e);
            }
        }

        return duration;
    }

    fn on_receive_channel_data(&mut self, data: ChannelMessage) {
        match data {
            ChannelMessage::Settings { settings } => {
                self.settings = settings;
            }
            _ => {}
        }
    }
}
