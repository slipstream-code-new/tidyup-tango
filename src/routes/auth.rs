use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::Redirect;
use tower_sessions::Session;
use uuid::Uuid;

use crate::domain::UserId;

/// Extractor that requires authentication.
/// If the user is not authenticated, redirects to /login.
pub struct AuthenticatedUser(pub UserId);

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = Redirect;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| Redirect::to("/login"))?;

        let user_id: Option<String> = session
            .get("user_id")
            .await
            .map_err(|_| Redirect::to("/login"))?;

        match user_id {
            Some(id_str) => {
                let uuid = Uuid::parse_str(&id_str).map_err(|_| Redirect::to("/login"))?;
                Ok(AuthenticatedUser(UserId::from_uuid(uuid)))
            }
            None => Err(Redirect::to("/login")),
        }
    }
}
