use serde::{Serialize, Deserialize};

pub const SECRET: &str = "ywt_secret";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub username: String,
    pub iat: usize,
    pub exp: usize,
}