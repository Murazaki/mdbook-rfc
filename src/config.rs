use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct RFCBookConfig {
    textFolder: String,
    vendorFolder: String,
    templateFolder: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BookConfig {
    srcFolder: String,
    buildFolder: String,
    preprocessors: Vec<String>,
}