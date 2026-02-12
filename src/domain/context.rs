use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::user::UserId;

const MAX_CONTEXT_NAME_LENGTH: usize = 50;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextId(Uuid);

impl Default for ContextId {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextId {
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

#[derive(Debug, Clone, PartialEq)]
pub struct ContextName(String);

impl ContextName {
    /// Parses and normalizes a context name to canonical form.
    ///
    /// Normalization steps:
    /// 1. Trim whitespace
    /// 2. Strip all leading `@` characters
    /// 3. Validate remainder is non-empty
    /// 4. Prepend exactly one `@`
    /// 5. Validate total length (including `@`) is <= 50 chars
    pub fn parse(input: String) -> Result<Self, ContextNameError> {
        let trimmed = input.trim();
        let stripped = trimmed.trim_start_matches('@');
        if stripped.is_empty() {
            return Err(ContextNameError::Empty);
        }
        let canonical = format!("@{stripped}");
        if canonical.len() > MAX_CONTEXT_NAME_LENGTH {
            return Err(ContextNameError::TooLong {
                max: MAX_CONTEXT_NAME_LENGTH,
                actual: canonical.len(),
            });
        }
        Ok(Self(canonical))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ContextNameError {
    #[error("Context name cannot be empty")]
    Empty,
    #[error("Context name too long (max {max}, got {actual})")]
    TooLong { max: usize, actual: usize },
}

#[derive(Debug, Clone)]
pub struct Context {
    id: ContextId,
    user_id: UserId,
    name: ContextName,
    position: i32,
    created_at: DateTime<Utc>,
}

impl Context {
    pub fn new(user_id: UserId, name: ContextName, position: i32) -> Self {
        Self {
            id: ContextId::new(),
            user_id,
            name,
            position,
            created_at: Utc::now(),
        }
    }

    pub fn from_parts(
        id: ContextId,
        user_id: UserId,
        name: ContextName,
        position: i32,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            user_id,
            name,
            position,
            created_at,
        }
    }

    pub fn id(&self) -> &ContextId {
        &self.id
    }

    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    pub fn name(&self) -> &ContextName {
        &self.name
    }

    pub fn position(&self) -> i32 {
        self.position
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- ContextId ----

    #[test]
    fn context_id_is_unique() {
        let id1 = ContextId::new();
        let id2 = ContextId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn context_id_from_uuid_roundtrips() {
        let uuid = Uuid::new_v4();
        let id = ContextId::from_uuid(uuid);
        assert_eq!(id.as_uuid(), &uuid);
    }

    // ---- ContextName ----

    #[test]
    fn valid_context_name_with_at_prefix_is_accepted() {
        let result = ContextName::parse("@computer".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "@computer");
    }

    #[test]
    fn context_name_without_at_prefix_gets_one_prepended() {
        let result = ContextName::parse("office".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "@office");
    }

    #[test]
    fn context_name_with_multiple_at_prefixes_normalizes_to_one() {
        let result = ContextName::parse("@@computer".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "@computer");
    }

    #[test]
    fn empty_context_name_is_rejected() {
        let result = ContextName::parse("".to_string());
        assert_eq!(result, Err(ContextNameError::Empty));
    }

    #[test]
    fn whitespace_only_context_name_is_rejected() {
        let result = ContextName::parse("   ".to_string());
        assert_eq!(result, Err(ContextNameError::Empty));
    }

    #[test]
    fn at_sign_only_context_name_is_rejected() {
        let result = ContextName::parse("@".to_string());
        assert_eq!(result, Err(ContextNameError::Empty));
    }

    #[test]
    fn multiple_at_signs_only_context_name_is_rejected() {
        let result = ContextName::parse("@@@".to_string());
        assert_eq!(result, Err(ContextNameError::Empty));
    }

    #[test]
    fn context_name_is_trimmed() {
        let result = ContextName::parse("  @home  ".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "@home");
    }

    #[test]
    fn context_name_without_prefix_is_trimmed_and_prefixed() {
        let result = ContextName::parse("  computer  ".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "@computer");
    }

    #[test]
    fn context_name_at_max_length_is_accepted() {
        // Max is 50 chars total including the @ prefix
        let base = "a".repeat(MAX_CONTEXT_NAME_LENGTH - 1); // 49 chars
        let result = ContextName::parse(base);
        assert!(result.is_ok());
        // Result is @<49 a's> = 50 chars total
        assert_eq!(result.unwrap().as_str().len(), MAX_CONTEXT_NAME_LENGTH);
    }

    #[test]
    fn context_name_exceeding_max_length_is_rejected() {
        // 50 chars of base + @ prefix = 51 total, exceeds limit
        let base = "a".repeat(MAX_CONTEXT_NAME_LENGTH);
        let result = ContextName::parse(base);
        assert_eq!(
            result,
            Err(ContextNameError::TooLong {
                max: MAX_CONTEXT_NAME_LENGTH,
                actual: MAX_CONTEXT_NAME_LENGTH + 1,
            })
        );
    }

    // ---- Context ----

    #[test]
    fn new_context_has_name_user_and_position() {
        let user_id = UserId::new();
        let name = ContextName::parse("@computer".to_string()).unwrap();
        let context = Context::new(user_id.clone(), name, 0);

        assert_eq!(context.name().as_str(), "@computer");
        assert_eq!(context.user_id(), &user_id);
        assert_eq!(context.position(), 0);
    }

    #[test]
    fn context_from_parts_preserves_fields() {
        let id = ContextId::new();
        let user_id = UserId::new();
        let name = ContextName::parse("@home".to_string()).unwrap();
        let now = Utc::now();

        let context = Context::from_parts(id.clone(), user_id.clone(), name, 3, now);

        assert_eq!(context.id(), &id);
        assert_eq!(context.user_id(), &user_id);
        assert_eq!(context.name().as_str(), "@home");
        assert_eq!(context.position(), 3);
        assert_eq!(context.created_at(), &now);
    }
}
