use crate::auth::AuthenticationService;
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio_postgres::{
    types::{FromSql, Type},
    Client,
};
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
    email_address: String,
    password_hash: String,
}

impl User {
    fn new(email_address: String, username: String, password: String) -> Self {
        let password_hash = hash(password, DEFAULT_COST).unwrap();
        Self {
            username,
            email_address,
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
    email_address: String,
}

#[derive(Deserialize)]
pub struct RegisterCommand {
    username: String,
    email: String,
    password: String,
}

impl RegisterCommand {
    pub async fn handle(
        &self,
        user_repository: &UserRepository,
    ) -> std::result::Result<UserDTO, UserErrors> {
        let user = User::new(
            self.email.clone(),
            self.username.clone(),
            self.password.clone(),
        );

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
                        email_address: self.email.clone(),
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

        let result = &self
            .client
            .query_typed(
                "INSERT INTO users (email_address, username, password_hash) VALUES ($1, $2, $3)",
                &[
                    (&user.email_address, Type::TEXT),
                    (&user.username, Type::TEXT),
                    (&user.password_hash, Type::TEXT),
                ],
            )
            .await;

        match result {
            Ok(_) => {
                let _ = &self
                    .queue
                    .send(UserDTO {
                        username: user.username.clone(),
                        email_address: user.email_address.clone()
                    })
                    .await;
            }
            Err(e) => {
                tracing::error!("Failure creating user: {}", e);

                return Ok(());
            }
        }

        Ok(())
    }

    async fn get_user(&self, username: &str) -> Result<Option<User>> {
        let result = &self
            .client
            .query_typed(
                "SELECT username, email_address, password_hash FROM users WHERE username = $1",
                &[(&username, Type::TEXT)],
            )
            .await;

        match result {
            Ok(result) => {
                if result.len() == 1 {
                    let row = result[0].clone();

                    return Ok(Some(User {
                        username: row.get(0),
                        email_address: row.get(1),
                        password_hash: row.get(2),
                    }));
                }
            }
            Err(e) => {
                tracing::error!("Failure getting user: {}", e);

                return Ok(None);
            }
        }

        Ok(None)
    }
}
