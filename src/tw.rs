use config::Config;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::monitors::devices::USBMon;
use crate::monitors::filechanges::FileChanges;
use crate::monitors::network::NETMon;
use crate::monitors::ssh_burn_file::SSHBurnMon;

pub enum Monitors {
    USBMon(USBMon),
    NetMon(NETMon),
    SSHBurnMon(SSHBurnMon),
    FileChanges(FileChanges)
}
pub struct AppState {
    pub mon_usb: bool,
    pub detection_triggered: bool,
    pub monitors: Arc<Mutex<Vec<Monitors>>>,
    pub settings_map: HashMap<String, String>,
}

impl AppState {
    pub fn new() -> Self {
        let config = Config::builder();
        let settings = config
            .add_source(config::File::with_name("config/Settings.toml"))
            .build()
            .unwrap();
        let settings_map = settings
            .try_deserialize::<HashMap<String, String>>()
            .unwrap();

        let mut monitors: Vec<Monitors> = vec![];

        if settings_map.get("usb_mon_enabled").unwrap().eq_ignore_ascii_case("true") {
            monitors.push(Monitors::USBMon(USBMon::new(settings_map.clone())));
        }
        if settings_map.get("net_mon_enabled").unwrap().eq_ignore_ascii_case("true") {
            monitors.push(Monitors::NetMon(NETMon::new(settings_map.clone())));
        }
        if settings_map.get("burn_file_mon_enabled").unwrap().eq_ignore_ascii_case("true") {
            monitors.push(Monitors::SSHBurnMon(SSHBurnMon::new(settings_map.clone())));
        }

        if settings_map.get("fs_mon_enabled").unwrap().eq_ignore_ascii_case("true") {
            monitors.push(Monitors::FileChanges(FileChanges::new(settings_map.clone())));
        }

        AppState {
            mon_usb: true,
            detection_triggered: false,
            monitors: Arc::new(Mutex::new(monitors)),
            settings_map,
        }
    }

    pub async fn run(&mut self) {
        loop {
            let mut binding = self.monitors.lock();
            let bind = binding.as_mut().unwrap();
            for i in bind.iter_mut() {
                match i {
                    Monitors::USBMon(e) => {
                        e.check().await;
                    }
                    Monitors::NetMon(e) => {
                        e.check().await;
                    }
                    Monitors::SSHBurnMon(e) => {
                        e.check().await;
                    }
                    Monitors::FileChanges(e) => {
                        e.check().await;
                    }
                }

                tokio::time::sleep(Duration::new(0, 500000000)).await;
            }
        }
    }
}

pub trait EventMonitor {
    async fn check(&mut self);
}
