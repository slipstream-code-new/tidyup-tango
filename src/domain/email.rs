/// A validated email address.
///
/// Constructed via `ValidatedEmail::parse()`. The email is normalized to lowercase
/// (case-insensitive per US-2). This type carries proof of validation -- once you
/// have a `ValidatedEmail`, you know it passed format checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedEmail(String);

impl ValidatedEmail {
    pub fn parse(input: String) -> Result<Self, EmailValidationError> {
        let trimmed = input.trim().to_string();

        if trimmed.is_empty() {
            return Err(EmailValidationError::Empty);
        }

        if trimmed.len() > 254 {
            return Err(EmailValidationError::TooLong);
        }

        if !trimmed.contains('@') {
            return Err(EmailValidationError::MissingAtSymbol);
        }

        let parts: Vec<&str> = trimmed.splitn(2, '@').collect();
        let local = parts[0];
        let domain = parts[1];

        if local.is_empty() || domain.is_empty() {
            return Err(EmailValidationError::MissingAtSymbol);
        }

        if !domain.contains('.') {
            return Err(EmailValidationError::MissingAtSymbol);
        }

        Ok(Self(trimmed.to_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ValidatedEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ValidatedEmail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EmailValidationError {
    #[error("Email address is empty")]
    Empty,
    #[error("Email address is missing '@' or domain")]
    MissingAtSymbol,
    #[error("Email address is too long")]
    TooLong,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_email_is_accepted() {
        let email = ValidatedEmail::parse("user@example.com".to_string());
        assert!(email.is_ok());
        assert_eq!(email.unwrap().as_str(), "user@example.com");
    }

    #[test]
    fn email_is_normalized_to_lowercase() {
        let email = ValidatedEmail::parse("User@Example.COM".to_string()).unwrap();
        assert_eq!(email.as_str(), "user@example.com");
    }

    #[test]
    fn email_with_plus_tag_is_valid() {
        let email = ValidatedEmail::parse("user+tag@example.com".to_string());
        assert!(email.is_ok());
    }

    #[test]
    fn empty_email_is_rejected() {
        let result = ValidatedEmail::parse("".to_string());
        assert_eq!(result, Err(EmailValidationError::Empty));
    }

    #[test]
    fn whitespace_only_email_is_rejected() {
        let result = ValidatedEmail::parse("   ".to_string());
        assert_eq!(result, Err(EmailValidationError::Empty));
    }

    #[test]
    fn email_without_at_symbol_is_rejected() {
        let result = ValidatedEmail::parse("userexample.com".to_string());
        assert_eq!(result, Err(EmailValidationError::MissingAtSymbol));
    }

    #[test]
    fn email_without_domain_dot_is_rejected() {
        let result = ValidatedEmail::parse("user@example".to_string());
        assert_eq!(result, Err(EmailValidationError::MissingAtSymbol));
    }

    #[test]
    fn email_with_empty_local_part_is_rejected() {
        let result = ValidatedEmail::parse("@example.com".to_string());
        assert_eq!(result, Err(EmailValidationError::MissingAtSymbol));
    }

    #[test]
    fn email_too_long_is_rejected() {
        let long_local = "a".repeat(250);
        let email = format!("{long_local}@example.com");
        let result = ValidatedEmail::parse(email);
        assert_eq!(result, Err(EmailValidationError::TooLong));
    }

    #[test]
    fn email_is_trimmed() {
        let email = ValidatedEmail::parse("  user@example.com  ".to_string()).unwrap();
        assert_eq!(email.as_str(), "user@example.com");
    }
}
