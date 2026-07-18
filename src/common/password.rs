use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand_core::OsRng;

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| e.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    let Ok(parsed_hash) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

// `#[cfg(test)]` means this whole module only exists when running
// `cargo test` — it adds nothing to your actual shipped binary.
#[cfg(test)]
mod tests {
    // Pulls in everything from the parent module (`hash_password`,
    // `verify_password`) — a very common pattern for unit test modules.
    use super::*;

    #[test]
    fn correct_password_verifies_successfully() {
        let hash = hash_password("correct-horse-battery-staple").unwrap();
        assert!(verify_password("correct-horse-battery-staple", &hash));
    }

    #[test]
    fn wrong_password_fails_verification() {
        let hash = hash_password("correct-horse-battery-staple").unwrap();
        assert!(!verify_password("wrong-password", &hash));
    }

    #[test]
    fn same_password_produces_different_hashes_each_time() {
        // Because of random salting (Lesson from way back on argon2!),
        // hashing the same password twice should NEVER produce an
        // identical stored hash.
        let hash1 = hash_password("same-password").unwrap();
        let hash2 = hash_password("same-password").unwrap();
        assert_ne!(hash1, hash2);
    }
}