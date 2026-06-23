use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

fn session_secret() -> String {
    std::env::var("SESSION_SECRET").unwrap_or_else(|_| "minimamemosa-default-secret-change-me".to_string())
}

fn get_hmac() -> HmacSha256 {
    HmacSha256::new_from_slice(session_secret().as_bytes()).expect("HMAC key")
}

pub fn create_session_token(user_id: i64) -> String {
    let expiry = chrono::Utc::now().timestamp() + 86400 * 7;
    let payload = format!("{}:{}", user_id, expiry);
    let mut mac = get_hmac();
    mac.update(payload.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());
    format!("{}:{}", payload, signature)
}

pub fn verify_session_token(token: &str) -> Option<i64> {
    let parts: Vec<&str> = token.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    let user_id = parts[0].parse::<i64>().ok()?;
    let expiry = parts[1].parse::<i64>().ok()?;
    let signature = parts[2];
    let now = chrono::Utc::now().timestamp();
    if now > expiry {
        return None;
    }
    let payload = format!("{}:{}", user_id, expiry);
    let mut mac = get_hmac();
    mac.update(payload.as_bytes());
    let expected = hex::encode(mac.finalize().into_bytes());
    if signature == expected { Some(user_id) } else { None }
}

pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    bcrypt::verify(password, hash)
}

pub fn create_captcha_token(answer: &str) -> String {
    let expiry = chrono::Utc::now().timestamp() + 300;
    let payload = format!("{}:{}", answer, expiry);
    let mut mac = get_hmac();
    mac.update(payload.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());
    format!("{}:{}", payload, signature)
}

pub fn verify_captcha_token(token: &str, user_answer: &str) -> bool {
    let parts: Vec<&str> = token.split(':').collect();
    if parts.len() != 3 {
        return false;
    }
    let answer = parts[0];
    let expiry = match parts[1].parse::<i64>() {
        Ok(val) => val,
        Err(_) => return false,
    };
    let signature = parts[2];
    let now = chrono::Utc::now().timestamp();
    if now > expiry {
        return false;
    }
    if answer.to_lowercase() != user_answer.to_lowercase() {
        return false;
    }
    let payload = format!("{}:{}", answer, expiry);
    let mut mac = get_hmac();
    mac.update(payload.as_bytes());
    let expected = hex::encode(mac.finalize().into_bytes());
    signature == expected
}

pub fn create_note_auth_token(memo_id: i64) -> String {
    let expiry = chrono::Utc::now().timestamp() + 86400 * 7; // valid for 7 days
    let payload = format!("{}:{}", memo_id, expiry);
    let mut mac = get_hmac();
    mac.update(payload.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());
    format!("{}:{}", payload, signature)
}

pub fn verify_note_auth_token(token: &str, expected_memo_id: i64) -> bool {
    let parts: Vec<&str> = token.split(':').collect();
    if parts.len() != 3 {
        return false;
    }
    let memo_id = match parts[0].parse::<i64>() {
        Ok(v) => v,
        Err(_) => return false,
    };
    if memo_id != expected_memo_id {
        return false;
    }
    let expiry = match parts[1].parse::<i64>() {
        Ok(v) => v,
        Err(_) => return false,
    };
    let signature = parts[2];
    let now = chrono::Utc::now().timestamp();
    if now > expiry {
        return false;
    }
    let payload = format!("{}:{}", memo_id, expiry);
    let mut mac = get_hmac();
    mac.update(payload.as_bytes());
    let expected = hex::encode(mac.finalize().into_bytes());
    signature == expected
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static AUTH_LOCK: Mutex<()> = Mutex::new(());

    fn setup_secret() -> std::sync::MutexGuard<'static, ()> {
        let guard = AUTH_LOCK.lock().unwrap();
        std::env::set_var("SESSION_SECRET", "test-secret-key-for-testing-only");
        guard
    }

    // ── Session Token Tests ──

    #[test]
    fn test_create_and_verify_session_token() {
        let _g = setup_secret();
        let token = create_session_token(42);
        let user_id = verify_session_token(&token);
        assert_eq!(user_id, Some(42));
    }

    #[test]
    fn test_verify_session_token_invalid_format() {
        let _g = setup_secret();
        assert_eq!(verify_session_token("invalid"), None);
        assert_eq!(verify_session_token("a:b"), None);
        assert_eq!(verify_session_token("a:b:c:d"), None);
    }

    #[test]
    fn test_verify_session_token_non_numeric_user_id() {
        let _g = setup_secret();
        let token = format!("abc:{}:somesig", chrono::Utc::now().timestamp() + 3600);
        assert_eq!(verify_session_token(&token), None);
    }

    #[test]
    fn test_verify_session_token_expired() {
        let _g = setup_secret();
        let past = chrono::Utc::now().timestamp() - 1;
        let payload = format!("1:{}", past);
        let mut mac = get_hmac();
        mac.update(payload.as_bytes());
        let sig = hex::encode(mac.finalize().into_bytes());
        let token = format!("{}:{}", payload, sig);
        assert_eq!(verify_session_token(&token), None);
    }

    #[test]
    fn test_verify_session_token_tampered() {
        let _g = setup_secret();
        let token = create_session_token(42);
        let tampered = token.replace("42", "99");
        assert_eq!(verify_session_token(&tampered), None);
    }

    #[test]
    fn test_session_token_user_id_zero() {
        let _g = setup_secret();
        let token = create_session_token(0);
        let user_id = verify_session_token(&token);
        assert_eq!(user_id, Some(0));
    }

    #[test]
    fn test_session_token_user_id_negative() {
        let _g = setup_secret();
        let token = create_session_token(-1);
        let user_id = verify_session_token(&token);
        assert_eq!(user_id, Some(-1));
    }

    // ── Password Hashing Tests ──

    #[test]
    fn test_hash_and_verify_password() {
        let hash = hash_password("my_password123").unwrap();
        assert!(verify_password("my_password123", &hash).unwrap());
    }

    #[test]
    fn test_verify_wrong_password() {
        let hash = hash_password("correct_password").unwrap();
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_verify_password_empty() {
        let hash = hash_password("pwd").unwrap();
        assert!(!verify_password("", &hash).unwrap());
    }

    #[test]
    fn test_verify_password_against_invalid_hash() {
        let result = verify_password("pwd", "not-a-valid-bcrypt-hash");
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_and_verify_unicode() {
        let hash = hash_password("pässwörd🦀").unwrap();
        assert!(verify_password("pässwörd🦀", &hash).unwrap());
    }

    // ── CAPTCHA Token Tests ──

    #[test]
    fn test_create_and_verify_captcha_token() {
        let _g = setup_secret();
        let token = create_captcha_token("AbCdE");
        assert!(verify_captcha_token(&token, "AbCdE"));
    }

    #[test]
    fn test_captcha_case_insensitive() {
        let _g = setup_secret();
        let token = create_captcha_token("AbCdE");
        assert!(verify_captcha_token(&token, "abcde"));
        assert!(verify_captcha_token(&token, "ABCDE"));
    }

    #[test]
    fn test_captcha_wrong_answer() {
        let _g = setup_secret();
        let token = create_captcha_token("AbCdE");
        assert!(!verify_captcha_token(&token, "Wrong"));
    }

    #[test]
    fn test_captcha_empty_answer() {
        let _g = setup_secret();
        let token = create_captcha_token("AbCdE");
        assert!(!verify_captcha_token(&token, ""));
    }

    #[test]
    fn test_captcha_invalid_token_format() {
        let _g = setup_secret();
        assert!(!verify_captcha_token("invalid", "a"));
        assert!(!verify_captcha_token("a:b", "a"));
        assert!(!verify_captcha_token("a:b:c:d", "a"));
    }

    #[test]
    fn test_captcha_non_numeric_expiry() {
        let _g = setup_secret();
        let token = format!("answer:notanumber:sig");
        assert!(!verify_captcha_token(&token, "answer"));
    }

    #[test]
    fn test_captcha_tampered() {
        let _g = setup_secret();
        let token = create_captcha_token("AbCdE");
        let tampered = token.replace("AbCdE", "XxXxX");
        assert!(!verify_captcha_token(&tampered, "XxXxX"));
    }

    #[test]
    fn test_captcha_different_secret_fails() {
        let _g = AUTH_LOCK.lock().unwrap();
        std::env::set_var("SESSION_SECRET", "first-secret");
        let token = create_captcha_token("abc");
        std::env::set_var("SESSION_SECRET", "second-secret");
        assert!(!verify_captcha_token(&token, "abc"));
        std::env::set_var("SESSION_SECRET", "test-secret-key-for-testing-only");
    }

    // ── Note Auth Token Tests ──

    #[test]
    fn test_create_and_verify_note_auth_token() {
        let _g = setup_secret();
        let token = create_note_auth_token(123);
        assert!(verify_note_auth_token(&token, 123));
    }

    #[test]
    fn test_note_auth_wrong_memo_id() {
        let _g = setup_secret();
        let token = create_note_auth_token(123);
        assert!(!verify_note_auth_token(&token, 456));
    }

    #[test]
    fn test_note_auth_invalid_format() {
        let _g = setup_secret();
        assert!(!verify_note_auth_token("invalid", 1));
        assert!(!verify_note_auth_token("a:b", 1));
    }

    #[test]
    fn test_note_auth_non_numeric() {
        let _g = setup_secret();
        let token = format!("abc:{}:sig", chrono::Utc::now().timestamp() + 3600);
        assert!(!verify_note_auth_token(&token, 1));
    }

    #[test]
    fn test_note_auth_tampered() {
        let _g = setup_secret();
        let token = create_note_auth_token(123);
        let tampered = token.replace("123", "999");
        assert!(!verify_note_auth_token(&tampered, 123));
        assert!(!verify_note_auth_token(&tampered, 999));
    }
}

