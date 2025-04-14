use fast_chemail::is_valid_email;

use crate::error::{ApiResult, ApiError, ApiErrorType};

pub const MAX_USERNAME: usize = 32;
pub const MAX_EMAIL: usize = 64;
pub const MAX_PASSWORD: usize = 64;
pub const MIN_PASSWORD: usize = 8;

pub fn check_username(username: &str) -> ApiResult<()> {
    if username.len() > MAX_USERNAME || username.len() == 0 {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Invalid username".to_string(),
        ));
    }
    Ok(())
}

pub fn check_email(email: &str) -> ApiResult<()> {
    if email.len() > MAX_EMAIL || email.len() == 0 {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Invalid email".to_string(),
        ));
    }
    if !is_valid_email(&email) {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Invalid email".to_string(),
        ));
    }
    Ok(())
}

pub fn check_email_tsinghua(email: &str) -> ApiResult<()> {
    check_email(email)?;
    if !email.ends_with("@mails.tsinghua.edu.cn") && !email.ends_with("@tsinghua.edu.cn") {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Invalid email".to_string(),
        ));
    }
    Ok(())
}

pub fn check_password(password: &str) -> ApiResult<()> {
    if password.len() > MAX_PASSWORD || password.len() < MIN_PASSWORD {
        return Err(ApiError::new(
            ApiErrorType::InvalidRequest,
            "Invalid password".to_string(),
        ));
    }
    Ok(())
}