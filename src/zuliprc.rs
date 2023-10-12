use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ZulipRc {
    pub api: Api,
}

#[derive(Debug, Deserialize)]
pub struct Api {
    pub email: String,
    pub key: String,
    pub site: String,
}
