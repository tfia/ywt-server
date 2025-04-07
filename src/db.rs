use mongodb::{Collection, Database};
use mongodb::bson::{doc, Document};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2
};

use crate::error::ApiResult;

pub async fn check_user_exists(
    db: &Database,
    username: &str,
) -> ApiResult<bool> {
    let collection: Collection<Document> = db.collection("users");
    let filter = doc! { "username": username };
    let user = collection.find_one(filter).await?;
    Ok(user.is_some())
}

pub async fn check_admin_exists(
    db: &Database,
    username: &str,
) -> ApiResult<bool> {
    let collection: Collection<Document> = db.collection("admins");
    let filter = doc! { "username": username };
    let admin = collection.find_one(filter).await?;
    Ok(admin.is_some())
}

pub async fn create_user(
    db: &Database,
    username: &str,
    email: &str,
    password: &str,
    created_at: &str,
    typ: &str, // "admins" or "users"
) -> ApiResult<()> {
    let collection = db.collection(typ);
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?.to_string();
    let user_doc = doc! {
        "username": username,
        "email": email,
        "password": password_hash,
        "created_at": created_at,
    };
    collection.insert_one(user_doc).await?;
    Ok(())
}