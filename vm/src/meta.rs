// TODO: flesh out

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    // TODO: flesh out
    pub specs: Specs,
    pub network: Network,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Specs {
    // TODO: flesh out
    pub drives: Vec<Drive>,
    pub features: Features,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Features {
    // TODO: flesh out
    pub uefi: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Drive {
    pub count: usize,
    pub size: String,
    #[serde(rename = "type")]
    pub ty: DriveType,
    pub category: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DriveType {
    #[serde(rename = "SSD")]
    Ssd,
    #[serde(rename = "HDD")]
    Hdd,
    #[serde(rename = "NVME")]
    Nvme,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Network {
    // TODO: flesh out
    pub interfaces: Vec<NetworkInterface>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NetworkInterface {
    pub name: String,
    pub mac: String, // TODO: mac address type?
    pub bond: String,
}
