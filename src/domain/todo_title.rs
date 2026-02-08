const MAX_TITLE_LENGTH: usize = 300;

#[derive(Debug, Clone, PartialEq)]
pub struct TodoTitle(String);

impl TodoTitle {
    pub fn parse(input: String) -> Result<Self, TodoTitleError> {
        let trimmed = input.trim().to_string();
        if trimmed.is_empty() {
            return Err(TodoTitleError::Empty);
        }
        if trimmed.len() > MAX_TITLE_LENGTH {
            return Err(TodoTitleError::TooLong {
                max: MAX_TITLE_LENGTH,
                actual: trimmed.len(),
            });
        }
        Ok(Self(trimmed))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum TodoTitleError {
    #[error("Todo title cannot be empty")]
    Empty,
    #[error("Todo title too long (max {max}, got {actual})")]
    TooLong { max: usize, actual: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_title_is_accepted() {
        let result = TodoTitle::parse("Buy groceries".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "Buy groceries");
    }

    #[test]
    fn empty_title_is_rejected() {
        let result = TodoTitle::parse("".to_string());
        assert_eq!(result, Err(TodoTitleError::Empty));
    }

    #[test]
    fn whitespace_only_title_is_rejected() {
        let result = TodoTitle::parse("   ".to_string());
        assert_eq!(result, Err(TodoTitleError::Empty));
    }

    #[test]
    fn title_is_trimmed() {
        let result = TodoTitle::parse("  Buy groceries  ".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "Buy groceries");
    }

    #[test]
    fn title_at_max_length_is_accepted() {
        let title = "a".repeat(MAX_TITLE_LENGTH);
        let result = TodoTitle::parse(title.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), title);
    }

    #[test]
    fn title_exceeding_max_length_is_rejected() {
        let title = "a".repeat(MAX_TITLE_LENGTH + 1);
        let result = TodoTitle::parse(title);
        assert_eq!(
            result,
            Err(TodoTitleError::TooLong {
                max: MAX_TITLE_LENGTH,
                actual: MAX_TITLE_LENGTH + 1,
            })
        );
    }

    #[test]
    fn title_with_special_characters_is_accepted() {
        let result = TodoTitle::parse("Buy milk & eggs <3".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "Buy milk & eggs <3");
    }
}
