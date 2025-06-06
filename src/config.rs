use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub bind_address: String,
    pub bind_port: u16,
    pub mongo_uri: String,
    pub mongo_db: String,
    pub admin_username: String,
    pub admin_email: String,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub smtp_username: String,
}