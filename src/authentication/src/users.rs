use crate::auth::AuthenticationService;
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio_postgres::{types::FromSql, Client};
use tokio_postgres_utils::FromRow;
use worker::*;

#[derive(Error, Debug)]
pub enum UserErrors {
    #[error("Exists")]
    Exists,
    #[error("Password is invalid")]
    InvalidPassword,
    #[error("Failure creating user")]
    UnknownFailure,
}

#[derive(Serialize, Deserialize, FromRow)]
struct User {
    username: String,
    password_hash: String,
}

impl User {
    fn new(username: String, password: String) -> Self {
        let password_hash = hash(password, DEFAULT_COST).unwrap();
        Self {
            username,
            password_hash,
        }
    }

    fn verify_password(&self, password: &str) -> bool {
        verify(password, &self.password_hash).unwrap_or(false)
    }
}

#[derive(Serialize)]
pub struct UserDTO {
    username: String,
}

#[derive(Deserialize)]
pub struct RegisterCommand {
    username: String,
    password: String,
}

impl RegisterCommand {
    pub async fn handle(
        &self,
        user_repository: &UserRepository,
    ) -> std::result::Result<UserDTO, UserErrors> {
        let user = User::new(self.username.clone(), self.password.clone());

        tracing::info!(
            "Attempting to create user with details: {:?} {:?}",
            &user.username,
            &user.password_hash
        );

        match user_repository.get_user(&user.username).await {
            Ok(existing_user) => match existing_user {
                Some(_) => {
                    tracing::info!("User exists");
                    Err(UserErrors::Exists)
                }
                None => match user_repository.add_user(user).await {
                    Ok(_) => Ok(UserDTO {
                        username: self.username.clone(),
                    }),
                    Err(e) => {
                        tracing::info!("Failure");
                        tracing::error!("{}", e);
                        Err(UserErrors::UnknownFailure)
                    }
                },
            },
            Err(_) => Err(UserErrors::UnknownFailure),
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
    pub async fn handle(
        &self,
        user_repository: &UserRepository,
        auth_service: &AuthenticationService,
    ) -> std::result::Result<LoginResponse, UserErrors> {
        if let Ok(user) = user_repository.get_user(&self.username).await {
            return match user {
                Some(user) => {
                    if user.verify_password(&self.password) {
                        return match auth_service.generate_token_for(self.username.clone()) {
                            Ok(token) => Ok(LoginResponse { token }),
                            Err(_e) => Err(UserErrors::UnknownFailure),
                        };
                    }

                    Err(UserErrors::UnknownFailure)
                }
                None => Err(UserErrors::UnknownFailure),
            };
        }

        Err(UserErrors::InvalidPassword)
    }
}

pub struct UserRepository {
    client: Client,
    queue: Queue,
}

impl UserRepository {
    pub fn new(client: Client, queue: Queue) -> Self {
        Self { client, queue }
    }

    async fn add_user(&self, user: User) -> Result<()> {
        tracing::info!("Creating user");

        let _result = &self
            .client
            .query(
                "INSERT INTO users (username, password_hash) VALUES ($1, $2)",
                &[&user.username, &user.password_hash],
            )
            .await
            .expect("Insert to complete successfully");

        let _ = &self
            .queue
            .send(UserDTO {
                username: user.username.clone(),
            })
            .await;

        Ok(())
    }

    async fn get_user(&self, username: &str) -> Result<Option<User>> {
        let result = &self
            .client
            .query_opt(
                "SELECT username, password_hash FROM users WHERE username = $1",
                &[&username],
            )
            .await
            .expect("Insert to complete successfully");

        match result {
            Some(row) => {
                let user: User = row.into();

                Ok(Some(user))
            }
            None => Ok(None),
        }
    }
}
