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

