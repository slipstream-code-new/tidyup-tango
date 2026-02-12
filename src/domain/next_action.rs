use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::context::ContextId;
use super::project::ProjectId;
use super::todo_title::TodoTitle;
use super::user::UserId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NextActionId(Uuid);

impl Default for NextActionId {
    fn default() -> Self {
        Self::new()
    }
}

impl NextActionId {
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

/// A concrete, physical action ready to do. Has a context (required) and
/// an optional project link (projects come in Step 6, so project_id is
/// nullable for now).
///
/// State machine:
///   Active --complete()--> Completed
///
/// Completed actions are removed from the active Next Actions list.
#[derive(Debug, Clone)]
pub enum NextAction {
    Active {
        id: NextActionId,
        user_id: UserId,
        context_id: ContextId,
        project_id: Option<ProjectId>,
        title: TodoTitle,
        created_at: DateTime<Utc>,
    },
    Completed {
        id: NextActionId,
        user_id: UserId,
        context_id: ContextId,
        project_id: Option<ProjectId>,
        title: TodoTitle,
        created_at: DateTime<Utc>,
        completed_at: DateTime<Utc>,
    },
}

impl NextAction {
    pub fn new(user_id: UserId, context_id: ContextId, title: TodoTitle) -> Self {
        Self::Active {
            id: NextActionId::new(),
            user_id,
            context_id,
            project_id: None,
            title,
            created_at: Utc::now(),
        }
    }

    pub fn new_for_project(
        user_id: UserId,
        context_id: ContextId,
        project_id: ProjectId,
        title: TodoTitle,
    ) -> Self {
        Self::Active {
            id: NextActionId::new(),
            user_id,
            context_id,
            project_id: Some(project_id),
            title,
            created_at: Utc::now(),
        }
    }

    pub fn from_parts(
        id: NextActionId,
        user_id: UserId,
        context_id: ContextId,
        project_id: Option<ProjectId>,
        title: TodoTitle,
        created_at: DateTime<Utc>,
        completed_at: Option<DateTime<Utc>>,
    ) -> Self {
        match completed_at {
            Some(completed_at) => Self::Completed {
                id,
                user_id,
                context_id,
                project_id,
                title,
                created_at,
                completed_at,
            },
            None => Self::Active {
                id,
                user_id,
                context_id,
                project_id,
                title,
                created_at,
            },
        }
    }

    pub fn id(&self) -> &NextActionId {
        match self {
            Self::Active { id, .. } | Self::Completed { id, .. } => id,
        }
    }

    pub fn user_id(&self) -> &UserId {
        match self {
            Self::Active { user_id, .. } | Self::Completed { user_id, .. } => user_id,
        }
    }

    pub fn context_id(&self) -> &ContextId {
        match self {
            Self::Active { context_id, .. } | Self::Completed { context_id, .. } => context_id,
        }
    }

    pub fn project_id(&self) -> Option<&ProjectId> {
        match self {
            Self::Active { project_id, .. } | Self::Completed { project_id, .. } => {
                project_id.as_ref()
            }
        }
    }

    pub fn title(&self) -> &TodoTitle {
        match self {
            Self::Active { title, .. } | Self::Completed { title, .. } => title,
        }
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        match self {
            Self::Active { created_at, .. } | Self::Completed { created_at, .. } => created_at,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active { .. })
    }

    pub fn is_completed(&self) -> bool {
        matches!(self, Self::Completed { .. })
    }

    /// Transitions an Active next action to Completed.
    /// If already completed, returns self unchanged.
    pub fn complete(self) -> Self {
        match self {
            Self::Active {
                id,
                user_id,
                context_id,
                project_id,
                title,
                created_at,
            } => Self::Completed {
                id,
                user_id,
                context_id,
                project_id,
                title,
                created_at,
                completed_at: Utc::now(),
            },
            completed @ Self::Completed { .. } => completed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_action_id_is_unique() {
        let id1 = NextActionId::new();
        let id2 = NextActionId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn next_action_id_from_uuid_roundtrips() {
        let uuid = Uuid::new_v4();
        let id = NextActionId::from_uuid(uuid);
        assert_eq!(id.as_uuid(), &uuid);
    }

    #[test]
    fn new_next_action_is_active() {
        let user_id = UserId::new();
        let context_id = ContextId::new();
        let title = TodoTitle::parse("Buy groceries".to_string()).unwrap();
        let action = NextAction::new(user_id, context_id, title);

        assert!(action.is_active());
        assert!(!action.is_completed());
        assert_eq!(action.title().as_str(), "Buy groceries");
    }

    #[test]
    fn new_next_action_has_user_and_context() {
        let user_id = UserId::new();
        let context_id = ContextId::new();
        let title = TodoTitle::parse("Call dentist".to_string()).unwrap();
        let action = NextAction::new(user_id.clone(), context_id.clone(), title);

        assert_eq!(action.user_id(), &user_id);
        assert_eq!(action.context_id(), &context_id);
    }

    #[test]
    fn complete_transitions_active_to_completed() {
        let action = NextAction::new(
            UserId::new(),
            ContextId::new(),
            TodoTitle::parse("File taxes".to_string()).unwrap(),
        );

        let completed = action.complete();
        assert!(completed.is_completed());
        assert!(!completed.is_active());
    }

    #[test]
    fn complete_preserves_id_and_fields() {
        let user_id = UserId::new();
        let context_id = ContextId::new();
        let title = TodoTitle::parse("Send email".to_string()).unwrap();
        let action = NextAction::new(user_id.clone(), context_id.clone(), title);
        let id = action.id().clone();

        let completed = action.complete();
        assert_eq!(completed.id(), &id);
        assert_eq!(completed.user_id(), &user_id);
        assert_eq!(completed.context_id(), &context_id);
        assert_eq!(completed.title().as_str(), "Send email");
    }

    #[test]
    fn complete_is_noop_on_completed_action() {
        let action = NextAction::new(
            UserId::new(),
            ContextId::new(),
            TodoTitle::parse("Already done".to_string()).unwrap(),
        );
        let completed = action.complete();
        assert!(completed.is_completed());

        // Completing again should not change anything
        let still_completed = completed.complete();
        assert!(still_completed.is_completed());
    }

    #[test]
    fn from_parts_creates_active_when_no_completed_at() {
        let id = NextActionId::new();
        let user_id = UserId::new();
        let context_id = ContextId::new();
        let title = TodoTitle::parse("Test".to_string()).unwrap();
        let now = Utc::now();

        let action = NextAction::from_parts(
            id.clone(),
            user_id.clone(),
            context_id.clone(),
            None,
            title,
            now,
            None,
        );

        assert!(action.is_active());
        assert_eq!(action.id(), &id);
        assert_eq!(action.user_id(), &user_id);
        assert_eq!(action.context_id(), &context_id);
        assert_eq!(action.created_at(), &now);
    }

    #[test]
    fn from_parts_creates_completed_when_completed_at_present() {
        let id = NextActionId::new();
        let user_id = UserId::new();
        let context_id = ContextId::new();
        let title = TodoTitle::parse("Done task".to_string()).unwrap();
        let created = Utc::now();
        let completed = Utc::now();

        let action = NextAction::from_parts(
            id.clone(),
            user_id.clone(),
            context_id.clone(),
            None,
            title,
            created,
            Some(completed),
        );

        assert!(action.is_completed());
        assert_eq!(action.id(), &id);
    }
}
