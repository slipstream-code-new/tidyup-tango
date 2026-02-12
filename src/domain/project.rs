use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::todo_title::TodoTitle;
use super::user::UserId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectId(Uuid);

impl Default for ProjectId {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectId {
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

/// A multi-step outcome requiring two or more actions. Each project
/// should have at least one next action to be considered "moving forward."
/// Projects without a next action are "stalled."
///
/// State machine:
///   Active --complete()--> Completed
///   Active --drop()-----> Dropped
///
/// Completed and Dropped are terminal states.
#[derive(Debug, Clone)]
pub enum Project {
    Active {
        id: ProjectId,
        user_id: UserId,
        title: TodoTitle,
        created_at: DateTime<Utc>,
    },
    Completed {
        id: ProjectId,
        user_id: UserId,
        title: TodoTitle,
        created_at: DateTime<Utc>,
        completed_at: DateTime<Utc>,
    },
    Dropped {
        id: ProjectId,
        user_id: UserId,
        title: TodoTitle,
        created_at: DateTime<Utc>,
        dropped_at: DateTime<Utc>,
    },
}

impl Project {
    pub fn new(user_id: UserId, title: TodoTitle) -> Self {
        Self::Active {
            id: ProjectId::new(),
            user_id,
            title,
            created_at: Utc::now(),
        }
    }

    pub fn from_parts(
        id: ProjectId,
        user_id: UserId,
        title: TodoTitle,
        created_at: DateTime<Utc>,
        completed_at: Option<DateTime<Utc>>,
        dropped_at: Option<DateTime<Utc>>,
    ) -> Self {
        if let Some(completed_at) = completed_at {
            Self::Completed {
                id,
                user_id,
                title,
                created_at,
                completed_at,
            }
        } else if let Some(dropped_at) = dropped_at {
            Self::Dropped {
                id,
                user_id,
                title,
                created_at,
                dropped_at,
            }
        } else {
            Self::Active {
                id,
                user_id,
                title,
                created_at,
            }
        }
    }

    pub fn id(&self) -> &ProjectId {
        match self {
            Self::Active { id, .. } | Self::Completed { id, .. } | Self::Dropped { id, .. } => id,
        }
    }

    pub fn user_id(&self) -> &UserId {
        match self {
            Self::Active { user_id, .. }
            | Self::Completed { user_id, .. }
            | Self::Dropped { user_id, .. } => user_id,
        }
    }

    pub fn title(&self) -> &TodoTitle {
        match self {
            Self::Active { title, .. }
            | Self::Completed { title, .. }
            | Self::Dropped { title, .. } => title,
        }
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        match self {
            Self::Active { created_at, .. }
            | Self::Completed { created_at, .. }
            | Self::Dropped { created_at, .. } => created_at,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active { .. })
    }

    pub fn is_completed(&self) -> bool {
        matches!(self, Self::Completed { .. })
    }

    pub fn is_dropped(&self) -> bool {
        matches!(self, Self::Dropped { .. })
    }

    /// Transitions an Active project to Completed.
    /// If already completed or dropped, returns self unchanged.
    pub fn complete(self) -> Self {
        match self {
            Self::Active {
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
            other => other,
        }
    }

    /// Transitions an Active project to Dropped.
    /// If already completed or dropped, returns self unchanged.
    pub fn drop_project(self) -> Self {
        match self {
            Self::Active {
                id,
                user_id,
                title,
                created_at,
            } => Self::Dropped {
                id,
                user_id,
                title,
                created_at,
                dropped_at: Utc::now(),
            },
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_id_is_unique() {
        let id1 = ProjectId::new();
        let id2 = ProjectId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn project_id_from_uuid_roundtrips() {
        let uuid = Uuid::new_v4();
        let id = ProjectId::from_uuid(uuid);
        assert_eq!(id.as_uuid(), &uuid);
    }

    #[test]
    fn new_project_is_active() {
        let user_id = UserId::new();
        let title = TodoTitle::parse("Launch website".to_string()).unwrap();
        let project = Project::new(user_id, title);

        assert!(project.is_active());
        assert!(!project.is_completed());
        assert!(!project.is_dropped());
        assert_eq!(project.title().as_str(), "Launch website");
    }

    #[test]
    fn new_project_has_user() {
        let user_id = UserId::new();
        let title = TodoTitle::parse("Plan trip".to_string()).unwrap();
        let project = Project::new(user_id.clone(), title);

        assert_eq!(project.user_id(), &user_id);
    }

    #[test]
    fn complete_transitions_active_to_completed() {
        let project = Project::new(
            UserId::new(),
            TodoTitle::parse("Ship feature".to_string()).unwrap(),
        );

        let completed = project.complete();
        assert!(completed.is_completed());
        assert!(!completed.is_active());
    }

    #[test]
    fn complete_preserves_id_and_fields() {
        let user_id = UserId::new();
        let title = TodoTitle::parse("Write book".to_string()).unwrap();
        let project = Project::new(user_id.clone(), title);
        let id = project.id().clone();

        let completed = project.complete();
        assert_eq!(completed.id(), &id);
        assert_eq!(completed.user_id(), &user_id);
        assert_eq!(completed.title().as_str(), "Write book");
    }

    #[test]
    fn complete_is_noop_on_completed_project() {
        let project = Project::new(
            UserId::new(),
            TodoTitle::parse("Already done".to_string()).unwrap(),
        );
        let completed = project.complete();
        let still_completed = completed.complete();
        assert!(still_completed.is_completed());
    }

    #[test]
    fn drop_transitions_active_to_dropped() {
        let project = Project::new(
            UserId::new(),
            TodoTitle::parse("Abandoned idea".to_string()).unwrap(),
        );

        let dropped = project.drop_project();
        assert!(dropped.is_dropped());
        assert!(!dropped.is_active());
    }

    #[test]
    fn drop_is_noop_on_completed_project() {
        let project = Project::new(
            UserId::new(),
            TodoTitle::parse("Finished work".to_string()).unwrap(),
        );
        let completed = project.complete();
        let still_completed = completed.drop_project();
        assert!(still_completed.is_completed());
    }

    #[test]
    fn from_parts_creates_active_when_no_timestamps() {
        let id = ProjectId::new();
        let user_id = UserId::new();
        let title = TodoTitle::parse("Test".to_string()).unwrap();
        let now = Utc::now();

        let project = Project::from_parts(id.clone(), user_id.clone(), title, now, None, None);

        assert!(project.is_active());
        assert_eq!(project.id(), &id);
        assert_eq!(project.user_id(), &user_id);
        assert_eq!(project.created_at(), &now);
    }

    #[test]
    fn from_parts_creates_completed_when_completed_at_present() {
        let id = ProjectId::new();
        let user_id = UserId::new();
        let title = TodoTitle::parse("Done".to_string()).unwrap();
        let created = Utc::now();
        let completed = Utc::now();

        let project =
            Project::from_parts(id.clone(), user_id, title, created, Some(completed), None);

        assert!(project.is_completed());
        assert_eq!(project.id(), &id);
    }

    #[test]
    fn from_parts_creates_dropped_when_dropped_at_present() {
        let id = ProjectId::new();
        let user_id = UserId::new();
        let title = TodoTitle::parse("Nope".to_string()).unwrap();
        let created = Utc::now();
        let dropped = Utc::now();

        let project = Project::from_parts(id.clone(), user_id, title, created, None, Some(dropped));

        assert!(project.is_dropped());
        assert_eq!(project.id(), &id);
    }

    #[test]
    fn from_parts_prefers_completed_over_dropped() {
        let id = ProjectId::new();
        let user_id = UserId::new();
        let title = TodoTitle::parse("Both".to_string()).unwrap();
        let created = Utc::now();
        let completed = Utc::now();
        let dropped = Utc::now();

        let project =
            Project::from_parts(id, user_id, title, created, Some(completed), Some(dropped));

        assert!(project.is_completed());
    }
}
