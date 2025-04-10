use lazy_static::lazy_static;
use std::sync::RwLock;

lazy_static! {
    pub static ref SMTP_USERNAME: RwLock<String> = RwLock::new(String::new());
}