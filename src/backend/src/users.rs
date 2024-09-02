use serde::{Deserialize, Serialize};
use worker::*;
use bcrypt::{hash, verify, DEFAULT_COST};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub username: String,
    password_hash: String,
}

impl User {
    pub fn new(username: String, password: String) -> Self {
        let password_hash = hash(password, DEFAULT_COST).unwrap();
        Self { username, password_hash }
    }

    pub fn verify_password(&self, password: &str) -> bool {
        verify(password, &self.password_hash).unwrap_or(false)
    }
}

pub struct UserRepository {
    db: D1Database,
}

impl UserRepository {
    pub fn new(db: D1Database) -> Self {
        Self { db }
    }

    pub async fn add_user(&self, user: User) -> Result<()> {
        self.db
            .prepare("INSERT INTO users (username, password_hash) VALUES (?, ?)")
            .bind(&[user.username.into(), user.password_hash.into()])
            .unwrap()
            .run()
            .await?;
        Ok(())
    }

    pub async fn get_user(&self, username: &str) -> Result<User> {
        let user = self.db
            .prepare("SELECT username, password_hash FROM users WHERE username = ?")
            .bind(&[username.into()])
            .unwrap()
            .first::<User>(None)
            .await?;
        Ok(user.unwrap())
    }
}