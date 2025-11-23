//! Mod for handling both local level saving, as well as fetching online levels

use std::{collections::HashMap, fs::read, path::PathBuf};

use base64::{Engine, prelude::BASE64_STANDARD};
use macroquad::prelude::warn;
use nanoserde::{DeBin, SerBin};
use quad_net::http_request::Request;

use crate::{level::Level, utils::DEBUG_ARGS};

pub enum NetworkResult {
    Success,
    Fail(String),
}
#[derive(PartialEq, Eq)]
pub enum LevelSorting {
    Name,
    Time,
    Downloads,
}
pub struct Data {
    pub local: LocalData,
    pub online_levels: Vec<(String, u32, String, u64)>,
    pub cached_online_levels: HashMap<String, Level>,
    pub list_request: Option<Request>,
    pub fetch_requests: Vec<(Request, String)>,
    pub failed_list_request: bool,
    pub uploading: Option<Request>,
    pub upload_result: Option<NetworkResult>,
    pub download_result: Option<(String, NetworkResult)>,
    pub verified_levels: HashMap<String, bool>,
    pub sorting: LevelSorting,
}
impl Data {
    pub fn rename_level(&mut self, index: usize, new_name: String) -> bool {
        if !self
            .local
            .user_levels
            .iter()
            .enumerate()
            .filter(|(i, _)| &index != i)
            .any(|(_, f)| f.0 == new_name)
        {
            let old_name = &self.local.user_levels[index].0;
            if let Some(value) = self.verified_levels.remove(old_name) {
                self.verified_levels.insert(new_name.clone(), value);
            }
            self.local.user_levels[index].0 = new_name;
            true
        } else {
            false
        }
    }
    pub fn upload_level(&mut self, level: Level, name: String, author: String) {
        let mut buffer = Vec::new();
        level.ser_bin(&mut buffer);
        let base64 = BASE64_STANDARD.encode(buffer);
        self.uploading = Some(
            quad_net::http_request::RequestBuilder::new(&format!(
                "{}/upload/{name}-{author}",
                DEBUG_ARGS.server_base_url
            ))
            .body(&base64)
            .method(quad_net::http_request::Method::Post)
            .send(),
        );
    }
    pub fn download_level(&mut self, name: &String) {
        self.fetch_requests.push((
            quad_net::http_request::RequestBuilder::new(&format!(
                "{}/get/{name}",
                DEBUG_ARGS.server_base_url
            ))
            .send(),
            name.to_string(),
        ));
    }
    pub fn update_level_list(&mut self) {
        self.list_request = Some(
            quad_net::http_request::RequestBuilder::new(
                &(DEBUG_ARGS.server_base_url.to_string() + "/list"),
            )
            .send(),
        );
    }
    pub fn load() -> Self {
        Self {
            local: LocalData::load(),
            online_levels: Vec::new(),
            cached_online_levels: HashMap::new(),
            fetch_requests: Vec::new(),
            list_request: Some(
                quad_net::http_request::RequestBuilder::new(
                    &(DEBUG_ARGS.server_base_url.to_string() + "/list"),
                )
                .send(),
            ),
            uploading: None,
            failed_list_request: false,
            upload_result: None,
            download_result: None,
            verified_levels: HashMap::new(),
            sorting: LevelSorting::Downloads,
        }
    }
    pub fn sort(&mut self) {
        match self.sorting {
            LevelSorting::Downloads => {
                self.online_levels.sort_by(|a, b| b.1.cmp(&a.1));
            }
            LevelSorting::Name => {
                self.online_levels
                    .sort_by(|a, b| a.0.to_ascii_lowercase().cmp(&b.0.to_ascii_lowercase()));
            }
            LevelSorting::Time => {
                self.online_levels.sort_by(|a, b| b.3.cmp(&a.3));
            }
        }
    }
    pub fn update(&mut self) {
        if let Some(request) = &mut self.uploading {
            if let Some(result) = request.try_recv() {
                self.uploading = None;
                self.upload_result = Some(match result {
                    Ok(data) => {
                        if data.starts_with("error:") {
                            let msg = data.trim_start_matches("error:").to_string();
                            NetworkResult::Fail(msg)
                        } else {
                            NetworkResult::Success
                        }
                    }
                    Err(e) => NetworkResult::Fail(e.to_string()),
                })
            }
        }
        if let Some(request) = &mut self.list_request {
            if let Some(result) = request.try_recv() {
                self.list_request = None;
                match result {
                    Ok(data) => {
                        if !data.is_empty() {
                            self.online_levels = data
                                .split(",")
                                .map(|f| {
                                    let (name, info) = f.split_once("_").unwrap();
                                    let [downloads, date, timestamp] =
                                        info.split("-").collect::<Vec<&str>>()[..]
                                    else {
                                        panic!("bad level info data! {f}");
                                    };
                                    let downloads: u32 = downloads.parse().unwrap();
                                    let timestamp: u64 = timestamp.parse().unwrap();
                                    let name = name.to_string();
                                    (name, downloads, date.to_string(), timestamp)
                                })
                                .collect();
                        }
                        self.failed_list_request = false;
                        self.sort();
                        //info!("fetched level list: {:?}", self.online_levels);
                    }
                    Err(_) => {
                        self.failed_list_request = true;
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
                        if data.starts_with("error:") {
                            let msg = data.trim_start_matches("error:").to_string();
                            self.download_result =
                                Some((name.to_string(), NetworkResult::Fail(msg)));
                        } else {
                            let Ok(decoded) = BASE64_STANDARD.decode(data) else {
                                let e = format!("level '{}' couldn't be base64 decoded", *name);
                                self.download_result =
                                    Some((name.to_string(), NetworkResult::Fail(e.to_string())));
                                return false;
                            };
                            let Some(level) = deserialize(&decoded) else {
                                let e = format!(
                                    "level '{}' couldn't be deserialized to level data",
                                    *name
                                );
                                self.download_result =
                                    Some((name.to_string(), NetworkResult::Fail(e.to_string())));
                                return false;
                            };
                            self.cached_online_levels.insert(name.clone(), level);
                            self.download_result = Some((name.to_string(), NetworkResult::Success));
                        }
                    }
                    Err(e) => {
                        self.download_result =
                            Some((name.to_string(), NetworkResult::Fail(e.to_string())));
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
    pub completed_online_levels: Vec<String>,
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
