use uuid::Uuid;

use super::email::ValidatedEmail;

/// Uniquely identifies a user. Newtype around Uuid to prevent mixing
/// with other UUID-based identifiers (e.g., TodoItemId).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserId(Uuid);

impl UserId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

/// A user in the system. Represents an authenticated user with valid credentials.
#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub email: ValidatedEmail,
}
