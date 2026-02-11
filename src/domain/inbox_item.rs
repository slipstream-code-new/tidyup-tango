use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::todo_title::TodoTitle;
use super::user::UserId;

#[derive(Debug, Clone, PartialEq)]
pub struct InboxItemId(Uuid);

impl Default for InboxItemId {
    fn default() -> Self {
        Self::new()
    }
}

impl InboxItemId {
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
pub struct InboxItem {
    id: InboxItemId,
    user_id: UserId,
    title: TodoTitle,
    created_at: DateTime<Utc>,
}

impl InboxItem {
    pub fn new(user_id: UserId, title: TodoTitle) -> Self {
        Self {
            id: InboxItemId::new(),
            user_id,
            title,
            created_at: Utc::now(),
        }
    }

    pub fn from_parts(
        id: InboxItemId,
        user_id: UserId,
        title: TodoTitle,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            user_id,
            title,
            created_at,
        }
    }

    pub fn id(&self) -> &InboxItemId {
        &self.id
    }

    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    pub fn title(&self) -> &TodoTitle {
        &self.title
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_inbox_item_has_title_and_user() {
        let user_id = UserId::new();
        let title = TodoTitle::parse("Buy groceries".to_string()).unwrap();
        let item = InboxItem::new(user_id.clone(), title);

        assert_eq!(item.title().as_str(), "Buy groceries");
        assert_eq!(item.user_id(), &user_id);
    }

    #[test]
    fn inbox_item_id_is_unique() {
        let id1 = InboxItemId::new();
        let id2 = InboxItemId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn inbox_item_from_parts_preserves_fields() {
        let id = InboxItemId::new();
        let user_id = UserId::new();
        let title = TodoTitle::parse("Test".to_string()).unwrap();
        let now = Utc::now();

        let item = InboxItem::from_parts(id.clone(), user_id.clone(), title, now);

        assert_eq!(item.id(), &id);
        assert_eq!(item.user_id(), &user_id);
        assert_eq!(item.title().as_str(), "Test");
        assert_eq!(item.created_at(), &now);
    }
}
