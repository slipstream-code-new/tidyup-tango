use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::todo_title::TodoTitle;
use super::user::UserId;

#[derive(Debug, Clone, PartialEq)]
pub struct TodoItemId(Uuid);

impl Default for TodoItemId {
    fn default() -> Self {
        Self::new()
    }
}

impl TodoItemId {
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

#[derive(Debug, Clone)]
pub enum TodoItem {
    Pending {
        id: TodoItemId,
        user_id: UserId,
        title: TodoTitle,
        created_at: DateTime<Utc>,
    },
    Completed {
        id: TodoItemId,
        user_id: UserId,
        title: TodoTitle,
        created_at: DateTime<Utc>,
        completed_at: DateTime<Utc>,
    },
}

impl TodoItem {
    pub fn new_pending(user_id: UserId, title: TodoTitle) -> Self {
        Self::Pending {
            id: TodoItemId::new(),
            user_id,
            title,
            created_at: Utc::now(),
        }
    }

    pub fn id(&self) -> &TodoItemId {
        match self {
            Self::Pending { id, .. } | Self::Completed { id, .. } => id,
        }
    }

    pub fn user_id(&self) -> &UserId {
        match self {
            Self::Pending { user_id, .. } | Self::Completed { user_id, .. } => user_id,
        }
    }

    pub fn title(&self) -> &TodoTitle {
        match self {
            Self::Pending { title, .. } | Self::Completed { title, .. } => title,
        }
    }

    pub fn is_completed(&self) -> bool {
        matches!(self, Self::Completed { .. })
    }

    /// Transition a Pending item to Completed. No-op if already completed.
    pub fn complete(self) -> Self {
        match self {
            Self::Pending {
                id,
                user_id,
                title,
                created_at,
            } => Self::Completed {
                id,
                user_id,
                title,
                created_at,
                completed_at: Utc::now(),
            },
            completed @ Self::Completed { .. } => completed,
        }
    }

    /// Transition a Completed item back to Pending. No-op if already pending.
    pub fn uncomplete(self) -> Self {
        match self {
            Self::Completed {
                id,
                user_id,
                title,
                created_at,
                ..
            } => Self::Pending {
                id,
                user_id,
                title,
                created_at,
            },
            pending @ Self::Pending { .. } => pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pending_creates_pending_item() {
        let user_id = UserId::new();
        let title = TodoTitle::parse("Buy groceries".to_string()).unwrap();
        let item = TodoItem::new_pending(user_id, title);

        assert!(!item.is_completed());
        assert_eq!(item.title().as_str(), "Buy groceries");
    }

    #[test]
    fn id_returns_the_item_id() {
        let user_id = UserId::new();
        let title = TodoTitle::parse("Test item".to_string()).unwrap();
        let item = TodoItem::new_pending(user_id, title);

        let _uuid = item.id().as_uuid();
    }

    #[test]
    fn complete_transitions_pending_to_completed() {
        let user_id = UserId::new();
        let title = TodoTitle::parse("Test".to_string()).unwrap();
        let item = TodoItem::new_pending(user_id, title);

        assert!(!item.is_completed());
        let completed = item.complete();
        assert!(completed.is_completed());
        assert_eq!(completed.title().as_str(), "Test");
    }

    #[test]
    fn complete_is_noop_on_completed_item() {
        let user_id = UserId::new();
        let title = TodoTitle::parse("Test".to_string()).unwrap();
        let item = TodoItem::new_pending(user_id, title).complete();

        assert!(item.is_completed());
        let still_completed = item.complete();
        assert!(still_completed.is_completed());
    }

    #[test]
    fn uncomplete_transitions_completed_to_pending() {
        let user_id = UserId::new();
        let title = TodoTitle::parse("Test".to_string()).unwrap();
        let item = TodoItem::new_pending(user_id, title).complete();

        assert!(item.is_completed());
        let pending = item.uncomplete();
        assert!(!pending.is_completed());
        assert_eq!(pending.title().as_str(), "Test");
    }

    #[test]
    fn uncomplete_is_noop_on_pending_item() {
        let user_id = UserId::new();
        let title = TodoTitle::parse("Test".to_string()).unwrap();
        let item = TodoItem::new_pending(user_id, title);

        assert!(!item.is_completed());
        let still_pending = item.uncomplete();
        assert!(!still_pending.is_completed());
    }

    #[test]
    fn complete_preserves_id_and_user_id() {
        let user_id = UserId::new();
        let title = TodoTitle::parse("Test".to_string()).unwrap();
        let item = TodoItem::new_pending(user_id.clone(), title);
        let item_id = item.id().clone();

        let completed = item.complete();
        assert_eq!(completed.id(), &item_id);
        assert_eq!(completed.user_id(), &user_id);
    }
}
