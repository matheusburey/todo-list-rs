use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::repository::user_repository::UserRepository;
use crate::types::database::AppState;
use crate::{config::settings::Settings, types::token::ITokenClaims};

pub async fn auth<B>(
    State(db): AppState,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|header| header.to_str().ok());

    let token = if let Some(auth_header) = auth_header {
        auth_header.split_once(' ').unwrap().1
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let claims = decode::<ITokenClaims>(
        token,
        &DecodingKey::from_secret(Settings::from_env().jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| (StatusCode::UNAUTHORIZED))?
    .claims;
    let user_repo = UserRepository::new(db.pool()).await;
    match user_repo.find_user(claims.sub).await {
        Ok(Some(user)) => {
            req.extensions_mut().insert(user);
            Ok(next.run(req).await)
        }
        Ok(None) => Err(StatusCode::UNAUTHORIZED),
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}
