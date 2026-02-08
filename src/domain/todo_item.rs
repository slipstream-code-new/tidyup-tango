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
}
