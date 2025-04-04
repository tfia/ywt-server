use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub bind_address: String,
    pub bind_port: u16,
    pub mongo_uri: String,
    pub mongo_db: String,
    pub admin_username: String,
    pub admin_email: String,
}