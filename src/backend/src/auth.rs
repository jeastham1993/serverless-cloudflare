use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use worker::Date;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub struct AuthenticationService {
    jwt_secret: String
}

impl AuthenticationService {
    pub fn new (jwt_secret: String) -> Self {
        AuthenticationService{
            jwt_secret
        }
    }

    pub fn generate_token_for(&self, username: String) -> std::result::Result<String, ()> {
        let claims = Claims {
            sub: username,
            exp: (Date::now().as_millis() / 1000) as usize + 3600, // 1 hour expiration
        };

        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(&self.jwt_secret.as_ref()))
            .map_err(|_e| ())?;

        Ok(token)
    }
    
    pub fn verify_jwt_token(&self, token: &str) -> Result<Claims, ()> {
        tracing::info!("Verifying JWT");
        let token_data = decode::<Claims>(token, &DecodingKey::from_secret(&self.jwt_secret.as_ref()), &Validation::default())
            .map_err(|e|{
                tracing::error!("{}", e);
                ()
            })?;
        Ok(token_data.claims)
    }
}