use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::todo_title::TodoTitle;
use super::user::UserId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaitingForId(Uuid);

impl Default for WaitingForId {
    fn default() -> Self {
        Self::new()
    }
}

impl WaitingForId {
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

/// Who or what the user is waiting on. Non-empty, max 100 chars, trimmed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaitingOn(String);

const WAITING_ON_MAX_LENGTH: usize = 100;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum WaitingOnError {
    #[error("Waiting-on cannot be empty")]
    Empty,
    #[error("Waiting-on too long (max {max}, got {actual})")]
    TooLong { max: usize, actual: usize },
}

impl WaitingOn {
    pub fn parse(input: String) -> Result<Self, WaitingOnError> {
        let trimmed = input.trim().to_string();
        if trimmed.is_empty() {
            return Err(WaitingOnError::Empty);
        }
        if trimmed.len() > WAITING_ON_MAX_LENGTH {
            return Err(WaitingOnError::TooLong {
                max: WAITING_ON_MAX_LENGTH,
                actual: trimmed.len(),
            });
        }
        Ok(Self(trimmed))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// An item the user is waiting on from someone else.
///
/// State machine:
///   Active --resolve()--> Resolved (received)
///
/// Resolved items are removed from the active Waiting For list.
#[derive(Debug, Clone)]
pub enum WaitingForItem {
    Active {
        id: WaitingForId,
        user_id: UserId,
        title: TodoTitle,
        waiting_on: WaitingOn,
        created_at: DateTime<Utc>,
    },
    Resolved {
        id: WaitingForId,
        user_id: UserId,
        title: TodoTitle,
        waiting_on: WaitingOn,
        created_at: DateTime<Utc>,
        resolved_at: DateTime<Utc>,
    },
}

impl WaitingForItem {
    pub fn new(user_id: UserId, title: TodoTitle, waiting_on: WaitingOn) -> Self {
        Self::Active {
            id: WaitingForId::new(),
            user_id,
            title,
            waiting_on,
            created_at: Utc::now(),
        }
    }

    pub fn from_parts(
        id: WaitingForId,
        user_id: UserId,
        title: TodoTitle,
        waiting_on: WaitingOn,
        created_at: DateTime<Utc>,
        resolved_at: Option<DateTime<Utc>>,
    ) -> Self {
        match resolved_at {
            Some(resolved_at) => Self::Resolved {
                id,
                user_id,
                title,
                waiting_on,
                created_at,
                resolved_at,
            },
            None => Self::Active {
                id,
                user_id,
                title,
                waiting_on,
                created_at,
            },
        }
    }

    pub fn id(&self) -> &WaitingForId {
        match self {
            Self::Active { id, .. } | Self::Resolved { id, .. } => id,
        }
    }

    pub fn user_id(&self) -> &UserId {
        match self {
            Self::Active { user_id, .. } | Self::Resolved { user_id, .. } => user_id,
        }
    }

    pub fn title(&self) -> &TodoTitle {
        match self {
            Self::Active { title, .. } | Self::Resolved { title, .. } => title,
        }
    }

    pub fn waiting_on(&self) -> &WaitingOn {
        match self {
            Self::Active { waiting_on, .. } | Self::Resolved { waiting_on, .. } => waiting_on,
        }
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        match self {
            Self::Active { created_at, .. } | Self::Resolved { created_at, .. } => created_at,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active { .. })
    }

    pub fn is_resolved(&self) -> bool {
        matches!(self, Self::Resolved { .. })
    }

    /// Transitions an Active waiting-for item to Resolved (received).
    /// If already resolved, returns self unchanged.
    pub fn resolve(self) -> Self {
        match self {
            Self::Active {
                id,
                user_id,
                title,
                waiting_on,
                created_at,
            } => Self::Resolved {
                id,
                user_id,
                title,
                waiting_on,
                created_at,
                resolved_at: Utc::now(),
            },
            resolved @ Self::Resolved { .. } => resolved,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waiting_for_id_is_unique() {
        let id1 = WaitingForId::new();
        let id2 = WaitingForId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn waiting_for_id_from_uuid_roundtrips() {
        let uuid = Uuid::new_v4();
        let id = WaitingForId::from_uuid(uuid);
        assert_eq!(id.as_uuid(), &uuid);
    }

    #[test]
    fn valid_waiting_on_is_accepted() {
        let waiting_on = WaitingOn::parse("Alice".to_string()).unwrap();
        assert_eq!(waiting_on.as_str(), "Alice");
    }

    #[test]
    fn waiting_on_is_trimmed() {
        let waiting_on = WaitingOn::parse("  Bob  ".to_string()).unwrap();
        assert_eq!(waiting_on.as_str(), "Bob");
    }

    #[test]
    fn empty_waiting_on_is_rejected() {
        let result = WaitingOn::parse("".to_string());
        assert!(matches!(result, Err(WaitingOnError::Empty)));
    }

    #[test]
    fn whitespace_only_waiting_on_is_rejected() {
        let result = WaitingOn::parse("   ".to_string());
        assert!(matches!(result, Err(WaitingOnError::Empty)));
    }

    #[test]
    fn waiting_on_at_max_length_is_accepted() {
        let name = "a".repeat(100);
        let waiting_on = WaitingOn::parse(name.clone()).unwrap();
        assert_eq!(waiting_on.as_str(), &name);
    }

    #[test]
    fn waiting_on_exceeding_max_length_is_rejected() {
        let name = "a".repeat(101);
        let result = WaitingOn::parse(name);
        assert!(matches!(
            result,
            Err(WaitingOnError::TooLong { max: 100, .. })
        ));
    }

    #[test]
    fn new_waiting_for_item_is_active() {
        let user_id = UserId::new();
        let title = TodoTitle::parse("Fix server bug".to_string()).unwrap();
        let waiting_on = WaitingOn::parse("DevOps team".to_string()).unwrap();
        let item = WaitingForItem::new(user_id, title, waiting_on);

        assert!(item.is_active());
        assert!(!item.is_resolved());
        assert_eq!(item.title().as_str(), "Fix server bug");
        assert_eq!(item.waiting_on().as_str(), "DevOps team");
    }

    #[test]
    fn new_waiting_for_item_has_user() {
        let user_id = UserId::new();
        let title = TodoTitle::parse("Get approval".to_string()).unwrap();
        let waiting_on = WaitingOn::parse("Manager".to_string()).unwrap();
        let item = WaitingForItem::new(user_id.clone(), title, waiting_on);

        assert_eq!(item.user_id(), &user_id);
    }

    #[test]
    fn resolve_transitions_active_to_resolved() {
        let item = WaitingForItem::new(
            UserId::new(),
            TodoTitle::parse("Receive package".to_string()).unwrap(),
            WaitingOn::parse("Amazon".to_string()).unwrap(),
        );

        let resolved = item.resolve();
        assert!(resolved.is_resolved());
        assert!(!resolved.is_active());
    }

    #[test]
    fn resolve_preserves_id_and_fields() {
        let user_id = UserId::new();
        let title = TodoTitle::parse("Get quote".to_string()).unwrap();
        let waiting_on = WaitingOn::parse("Contractor".to_string()).unwrap();
        let item = WaitingForItem::new(user_id.clone(), title, waiting_on);
        let id = item.id().clone();

        let resolved = item.resolve();
        assert_eq!(resolved.id(), &id);
        assert_eq!(resolved.user_id(), &user_id);
        assert_eq!(resolved.title().as_str(), "Get quote");
        assert_eq!(resolved.waiting_on().as_str(), "Contractor");
    }

    #[test]
    fn resolve_is_noop_on_resolved_item() {
        let item = WaitingForItem::new(
            UserId::new(),
            TodoTitle::parse("Already received".to_string()).unwrap(),
            WaitingOn::parse("Vendor".to_string()).unwrap(),
        );
        let resolved = item.resolve();
        let still_resolved = resolved.resolve();
        assert!(still_resolved.is_resolved());
    }

    #[test]
    fn from_parts_creates_active_when_no_resolved_at() {
        let id = WaitingForId::new();
        let user_id = UserId::new();
        let title = TodoTitle::parse("Test".to_string()).unwrap();
        let waiting_on = WaitingOn::parse("Someone".to_string()).unwrap();
        let now = Utc::now();

        let item =
            WaitingForItem::from_parts(id.clone(), user_id.clone(), title, waiting_on, now, None);

        assert!(item.is_active());
        assert_eq!(item.id(), &id);
        assert_eq!(item.user_id(), &user_id);
        assert_eq!(item.created_at(), &now);
    }

    #[test]
    fn from_parts_creates_resolved_when_resolved_at_present() {
        let id = WaitingForId::new();
        let user_id = UserId::new();
        let title = TodoTitle::parse("Done".to_string()).unwrap();
        let waiting_on = WaitingOn::parse("Other".to_string()).unwrap();
        let created = Utc::now();
        let resolved = Utc::now();

        let item = WaitingForItem::from_parts(
            id.clone(),
            user_id,
            title,
            waiting_on,
            created,
            Some(resolved),
        );

        assert!(item.is_resolved());
        assert_eq!(item.id(), &id);
    }
}
