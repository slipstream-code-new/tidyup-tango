use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::todo_title::TodoTitle;
use super::user::UserId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SomedayMaybeId(Uuid);

impl Default for SomedayMaybeId {
    fn default() -> Self {
        Self::new()
    }
}

impl SomedayMaybeId {
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

/// A parked idea -- not committed to, but not forgotten.
/// No state machine: items are created, optionally edited, and either
/// activated (moved to inbox) or deleted.
#[derive(Debug, Clone)]
pub struct SomedayMaybeItem {
    id: SomedayMaybeId,
    user_id: UserId,
    title: TodoTitle,
    created_at: DateTime<Utc>,
}

impl SomedayMaybeItem {
    pub fn new(user_id: UserId, title: TodoTitle) -> Self {
        Self {
            id: SomedayMaybeId::new(),
            user_id,
            title,
            created_at: Utc::now(),
        }
    }

    pub fn from_parts(
        id: SomedayMaybeId,
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

    pub fn id(&self) -> &SomedayMaybeId {
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
    fn someday_maybe_id_is_unique() {
        let id1 = SomedayMaybeId::new();
        let id2 = SomedayMaybeId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn someday_maybe_id_from_uuid_roundtrips() {
        let uuid = Uuid::new_v4();
        let id = SomedayMaybeId::from_uuid(uuid);
        assert_eq!(id.as_uuid(), &uuid);
    }

    #[test]
    fn new_someday_maybe_item_has_correct_fields() {
        let user_id = UserId::new();
        let title = TodoTitle::parse("Learn Esperanto".to_string()).unwrap();
        let item = SomedayMaybeItem::new(user_id.clone(), title);

        assert_eq!(item.user_id(), &user_id);
        assert_eq!(item.title().as_str(), "Learn Esperanto");
    }

    #[test]
    fn from_parts_reconstructs_item() {
        let id = SomedayMaybeId::new();
        let user_id = UserId::new();
        let title = TodoTitle::parse("Build a treehouse".to_string()).unwrap();
        let now = Utc::now();

        let item = SomedayMaybeItem::from_parts(id.clone(), user_id.clone(), title, now);

        assert_eq!(item.id(), &id);
        assert_eq!(item.user_id(), &user_id);
        assert_eq!(item.title().as_str(), "Build a treehouse");
        assert_eq!(item.created_at(), &now);
    }
}
