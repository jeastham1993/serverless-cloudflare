use serde::{Deserialize, Serialize};
use thiserror::Error;
use worker::*;
use bcrypt::{hash, verify, DEFAULT_COST};
use crate::{auth::AuthenticationService};

#[derive(Error, Debug)]
pub enum UserErrors {
    #[error("Password is invalid")]
    InvalidPassword,
    #[error("Failure creating user")]
    UnknownFailure
}

#[derive(Serialize, Deserialize)]
struct User {
    username: String,
    password_hash: String,
}

impl User {
    fn new(username: String, password: String) -> Self {
        let password_hash = hash(password, DEFAULT_COST).unwrap();
        Self { username, password_hash }
    }


    fn verify_password(&self, password: &str) -> bool {
        verify(password, &self.password_hash).unwrap_or(false)
    }
}

#[derive(Serialize)]
pub struct UserDTO {
    username: String
}

#[derive(Deserialize)]
pub struct RegisterCommand {
    username: String,
    password: String,
}

impl RegisterCommand {
    pub async fn handle(&self, user_repository: &UserRepository) -> std::result::Result<UserDTO, UserErrors>{
        let user = User::new(self.username.clone(), self.password.clone());

        match user_repository.add_user(user).await {
            Ok(_) => Ok(UserDTO{
                username: self.username.clone()
            }),
            Err(e) => {
                tracing::info!("Failure");
                tracing::error!("{}", e);
                Err(UserErrors::UnknownFailure)
            },
        }
    }
}



#[derive(Deserialize)]
pub struct LoginCommand {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    token: String,
}

impl LoginCommand {
    pub async fn handle(&self, user_repository: &UserRepository, auth_service: &AuthenticationService) -> std::result::Result<LoginResponse, UserErrors>{
        if let Ok(user) = user_repository.get_user(&self.username).await {
            if user.verify_password(&self.password) {
                return match auth_service.generate_token_for(self.username.clone())  {
                    Ok(token) => Ok(LoginResponse{
                        token
                    }),
                    Err(_e) => Err(UserErrors::UnknownFailure),
                }
            }
        }

        Err(UserErrors::InvalidPassword)
    }
}

pub struct UserRepository {
    db: D1Database,
}

impl UserRepository {
    pub fn new(db: D1Database) -> Self {
        Self { db }
    }

    async fn add_user(&self, user: User) -> Result<()> {
        self.db
            .prepare("INSERT INTO users (username, password_hash) VALUES (?, ?)")
            .bind(&[user.username.into(), user.password_hash.into()])
            .unwrap()
            .run()
            .await?;
        Ok(())
    }

    async fn get_user(&self, username: &str) -> Result<User> {
        let user = self.db
            .prepare("SELECT username, password_hash FROM users WHERE username = ?")
            .bind(&[username.into()])
            .unwrap()
            .first::<User>(None)
            .await?;
        Ok(user.unwrap())
    }
}