#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct RFCBookConfig {
    pub textFolder: String,
    pub vendorFolder: String,
    pub templateFolder: String,
    
    pub preprocessors: Vec<String>,
    pub packages: Vec<String>,
}

impl Default for RFCBookConfig {
    fn default() -> RFCBookConfig {
        RFCBookConfig {
            textFolder: "text".into(),
            vendorFolder: "public".into(),
            templateFolder: "template".into(),
            preprocessors: Vec::new(),
            packages: Vec::new(),
        }
    }
}