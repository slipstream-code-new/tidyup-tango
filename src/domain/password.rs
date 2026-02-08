use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

/// A raw password that has been validated but not yet hashed.
/// This type exists only at the boundary -- it is consumed when hashing.
#[derive(Debug, PartialEq)]
pub struct Password(String);

impl Password {
    pub fn parse(input: String) -> Result<Self, PasswordError> {
        if input.is_empty() {
            return Err(PasswordError::Empty);
        }

        if input.len() < 8 {
            return Err(PasswordError::TooShort);
        }

        if input.len() > 128 {
            return Err(PasswordError::TooLong);
        }

        Ok(Self(input))
    }

    /// Hash the password with Argon2. Consumes self to prevent reuse
    /// of the plaintext password.
    pub fn hash(self) -> Result<PasswordHash_, anyhow::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(self.0.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {e}"))?;
        Ok(PasswordHash_(hash.to_string()))
    }
}

/// A hashed password stored in the database.
/// The raw password is never recoverable from this type.
#[derive(Debug, Clone)]
pub struct PasswordHash_(String);

impl PasswordHash_ {
    pub fn from_phc(phc_string: String) -> Self {
        Self(phc_string)
    }

    pub fn verify(&self, candidate: &str) -> bool {
        let Ok(parsed_hash) = PasswordHash::new(&self.0) else {
            return false;
        };
        Argon2::default()
            .verify_password(candidate.as_bytes(), &parsed_hash)
            .is_ok()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PasswordError {
    #[error("Password is empty")]
    Empty,
    #[error("Password must be at least 8 characters")]
    TooShort,
    #[error("Password is too long")]
    TooLong,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_password_is_accepted() {
        let result = Password::parse("securepassword123".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn password_exactly_8_chars_is_accepted() {
        let result = Password::parse("12345678".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn empty_password_is_rejected() {
        let result = Password::parse("".to_string());
        assert_eq!(result, Err(PasswordError::Empty));
    }

    #[test]
    fn short_password_is_rejected() {
        let result = Password::parse("1234567".to_string());
        assert_eq!(result, Err(PasswordError::TooShort));
    }

    #[test]
    fn very_long_password_is_rejected() {
        let long = "a".repeat(129);
        let result = Password::parse(long);
        assert_eq!(result, Err(PasswordError::TooLong));
    }

    #[test]
    fn password_hash_and_verify_roundtrip() {
        let password = Password::parse("securepassword123".to_string()).unwrap();
        let hash = password.hash().unwrap();

        assert!(hash.verify("securepassword123"));
        assert!(!hash.verify("wrongpassword"));
    }
}
