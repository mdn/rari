use std::collections::HashMap;

use once_cell::sync::OnceCell;
use rari_types::globals::content_root;
use rari_utils::io::read_to_string;
use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
pub struct InterfaceData {
    pub inh: String,
}

pub static JSON_DATA_INTERFACE: OnceCell<HashMap<String, InterfaceData>> = OnceCell::new();

pub fn json_data_interface() -> &'static HashMap<String, InterfaceData> {
    JSON_DATA_INTERFACE.get_or_init(|| {
        let f = content_root().join("jsondata/InterfaceData.json");
        let json_str = read_to_string(f).expect("unable to read interface data json");
        let mut interface_data: Vec<HashMap<String, InterfaceData>> =
            serde_json::from_str(&json_str).expect("unable to parse interface data json");
        interface_data.pop().unwrap_or_default()
    })
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct GroupData {
    #[serde(default)]
    pub overview: Vec<String>,
    #[serde(default)]
    pub guides: Vec<String>,
    #[serde(default)]
    pub interfaces: Vec<String>,
    #[serde(default)]
    pub methods: Vec<String>,
    #[serde(default)]
    pub properties: Vec<String>,
    #[serde(default)]
    pub events: Vec<String>,
    #[serde(default)]
    pub tutorial: Vec<String>,
}

pub static JSON_DATA_GROUP: OnceCell<HashMap<String, GroupData>> = OnceCell::new();

pub fn json_data_group() -> &'static HashMap<String, GroupData> {
    JSON_DATA_GROUP.get_or_init(|| {
        let f = content_root().join("jsondata/GroupData.json");
        let json_str = read_to_string(f).expect("unable to read interface data json");
        let mut interface_data: Vec<HashMap<String, GroupData>> =
            serde_json::from_str(&json_str).expect("unable to parse group data json");
        interface_data.pop().unwrap_or_default()
    })
}
