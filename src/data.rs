//! Mod for handling both local level saving, as well as fetching online levels

use std::{collections::HashMap, fs::read, path::PathBuf};

use base64::{Engine, prelude::BASE64_STANDARD};
use macroquad::prelude::{info, warn};
use nanoserde::{DeBin, SerBin};
use quad_net::http_request::Request;

use crate::level::Level;

pub const SERVER_BASE_ADRESS: &str = "http://127.0.0.1:5462";

pub struct Data {
    pub local: LocalData,
    pub online_levels: Vec<String>,
    pub cached_online_levels: HashMap<String, Level>,
    pub list_request: Option<Request>,
    pub fetch_requests: Vec<(Request, String)>,
}
impl Data {
    pub fn load() -> Self {
        Self {
            local: LocalData::load(),
            online_levels: Vec::new(),
            cached_online_levels: HashMap::new(),
            fetch_requests: Vec::new(),
            list_request: Some(
                quad_net::http_request::RequestBuilder::new(
                    &(SERVER_BASE_ADRESS.to_string() + "/list"),
                )
                .send(),
            ),
        }
    }
    pub fn update(&mut self) {
        if let Some(request) = &mut self.list_request {
            if let Some(result) = request.try_recv() {
                self.list_request = None;
                match result {
                    Ok(data) => {
                        self.online_levels = data.split(",").map(|f| f.to_string()).collect();
                        info!("fetched level list: {:?}", self.online_levels);
                    }
                    Err(_) => {
                        warn!("level list couldn't be fetched");
                    }
                }
            }
        }
        self.fetch_requests.retain_mut(|(request, name)| {
            if let Some(result) = request.try_recv() {
                match result {
                    Ok(data) => {
                        fn deserialize(buffer: &[u8]) -> Option<Level> {
                            Level::de_bin(&mut 0, buffer).ok()
                        }
                        let Ok(decoded) = BASE64_STANDARD.decode(data) else {
                            warn!("level '{}' couldn't be base64 decoded", *name);
                            return false;
                        };
                        let Some(level) = deserialize(&decoded) else {
                            warn!("level '{}' couldn't be deserialized to level data", *name);
                            return false;
                        };
                        self.cached_online_levels.insert(name.clone(), level);
                    }
                    Err(_) => {
                        warn!("level by name '{}' couldn't be downloaded", *name);
                    }
                }
                return false;
            }
            true
        });
    }
}

#[derive(DeBin, SerBin, Default)]
pub struct LocalData {
    pub user_levels: Vec<(String, Level)>,
}
impl LocalData {
    fn get_save_path() -> Option<PathBuf> {
        let path = std::env::current_exe().ok()?;
        let parent = path.parent()?;
        let path = parent.join("save.wa");
        Some(path)
    }
    pub fn load() -> Self {
        fn deserialize(buffer: &[u8]) -> Option<LocalData> {
            LocalData::de_bin(&mut 0, buffer).ok()
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(path) = Self::get_save_path() else {
                return Self::default();
            };
            let Ok(buffer) = read(path) else {
                return Self::default();
            };
            deserialize(&buffer).unwrap_or_default()
        }

        #[cfg(target_arch = "wasm32")]
        {
            let storage = quad_storage::LocalStorage::default();

            let Some(data) = storage.get("save.wa") else {
                return Self::default();
            };
            let Ok(buffer) = BASE64_STANDARD.decode(&data) else {
                return Self::default();
            };
            deserialize(&buffer).unwrap_or_default()
        }
    }
    pub fn store(&self) {
        fn serialize(data: &LocalData) -> Vec<u8> {
            let mut buffer = Vec::new();
            data.ser_bin(&mut buffer);
            buffer
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::fs::write;

            let Some(path) = Self::get_save_path() else {
                return;
            };
            let data = serialize(self);
            write(path, data).unwrap();
        }

        #[cfg(target_arch = "wasm32")]
        {
            let mut storage = quad_storage::LocalStorage::default();
            let data = serialize(&self);
            let buffer = BASE64_STANDARD.encode(&data);
            storage.set("save.wa", &buffer);
        }
    }
}
