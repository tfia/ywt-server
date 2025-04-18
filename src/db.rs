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

pub struct AdminType;
pub struct UserType;

pub trait UserTypeTrait {
    const VALUE: &'static str;
}

impl UserTypeTrait for AdminType {
    const VALUE: &'static str = "admins";
}

impl UserTypeTrait for UserType {
    const VALUE: &'static str = "tmp_users";
}

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

pub async fn check_email_exists(
    db: &Database,
    email: &str,
) -> ApiResult<bool> {
    let collection: Collection<Document> = db.collection("users");
    let filter = doc! { "email": email };
    let user = collection.find_one(filter).await?;
    Ok(user.is_some())
}

pub async fn create_user<T: UserTypeTrait>(
    db: &Database,
    username: &str,
    email: &str,
    password: &str,
    created_at: &str,
) -> ApiResult<()> {
    let typ = T::VALUE;
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

pub async fn activate_user(
    db: &Database,
    username: &str,
) -> ApiResult<()> {
    let tmp_collection: Collection<Document> = db.collection("tmp_users");
    let users_collection = db.collection("users");
    let filter = doc! { "username": username };
    let user = tmp_collection.find_one(filter.clone()).await?;
    if let Some(user_doc) = user {
        let email = user_doc.get_str("email")?;
        let password = user_doc.get_str("password")?;
        let created_at = user_doc.get_str("created_at")?;
        users_collection.insert_one(doc! {
            "username": username,
            "email": email,
            "password": password,
            "created_at": created_at,
        }).await?;
        tmp_collection.delete_one(filter).await?;
    }
    Ok(())
}