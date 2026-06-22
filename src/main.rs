mod auth;
mod db;
mod templates;

use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Form, Multipart, Path, State, ConnectInfo},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
    routing::{get, post, put},
    Json, Router,
};
use chrono::Datelike;
use serde::Deserialize;
use serde_json::json;
use templates::Templates;
use tower_http::services::ServeDir;
use std::path::PathBuf;
use std::net::SocketAddr;

struct AppState {
    db: db::Database,
    templates: Templates,
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct RegisterForm {
    username: String,
    password: String,
    captcha_answer: String,
}

#[derive(Deserialize)]
struct MemoForm {
    content: String,
    visibility: Option<String>,
    visibility_password: Option<String>,
}

fn render_markdown(text: &str) -> String {
    let parser = pulldown_cmark::Parser::new(text);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    html
}

fn format_bytes(bytes: i64) -> String {
    if bytes == 0 {
        return "0 Bytes".to_string();
    }
    let k = 1024.0;
    let sizes = ["Bytes", "KB", "MB", "GB"];
    let i = (bytes as f64).log(k).floor() as usize;
    let i = std::cmp::min(i, sizes.len() - 1);
    let val = (bytes as f64) / k.powi(i as i32);
    format!("{:.1} {}", val, sizes[i])
}

fn process_memo_content(
    db: &db::Database,
    content: &str,
) -> (String, Vec<serde_json::Value>) {
    let mut resources = Vec::new();
    let mut cleaned_content = String::new();
    let mut remaining = content;
    
    while !remaining.is_empty() {
        if let Some(pos) = remaining.find("/resources/") {
            let mut start_idx = None;
            let search_start = if pos > 200 { pos - 200 } else { 0 };
            let before = &remaining[search_start..pos];
            if let Some(bracket_pos) = before.rfind('[') {
                let actual_bracket_pos = search_start + bracket_pos;
                if actual_bracket_pos > 0 && &remaining[actual_bracket_pos - 1..actual_bracket_pos] == "!" {
                    start_idx = Some(actual_bracket_pos - 1);
                } else {
                    start_idx = Some(actual_bracket_pos);
                }
            }
            
            let id_start = pos + "/resources/".len();
            let sub = &remaining[id_start..];
            if let Some(end_bracket_idx) = sub.find(')') {
                let id_str = &sub[..end_bracket_idx];
                if let Ok(id) = id_str.parse::<i64>() {
                    if let Ok(Some(r)) = db.get_resource_public(id) {
                        let is_img = r.3.starts_with("image/");
                        let size_formatted = format_bytes(r.4);
                        resources.push(json!({
                            "id": r.0,
                            "filename": r.1,
                            "original_name": r.2,
                            "mime_type": r.3,
                            "is_image": is_img,
                            "size": size_formatted,
                        }));
                    }
                    
                    if let Some(start) = start_idx {
                        cleaned_content.push_str(&remaining[..start]);
                    } else {
                        cleaned_content.push_str(&remaining[..pos]);
                    }
                    
                    let link_end = id_start + end_bracket_idx + 1;
                    remaining = &remaining[link_end..];
                    continue;
                }
            }
            
            cleaned_content.push_str(&remaining[..pos + "/resources/".len()]);
            remaining = &remaining[pos + "/resources/".len()..];
        } else {
            cleaned_content.push_str(remaining);
            break;
        }
    }
    
    (cleaned_content.trim().to_string(), resources)
}

fn extract_first_image_id(content: &str) -> Option<i64> {
    let mut remaining = content;
    while let Some(pos) = remaining.find("![") {
        let sub = &remaining[pos..];
        if let Some(url_pos) = sub.find("](/resources/") {
            let start = url_pos + "](/resources/".len();
            if let Some(end_pos) = sub[start..].find(')') {
                let id_str = &sub[start..start + end_pos];
                if let Ok(id) = id_str.parse::<i64>() {
                    return Some(id);
                }
            }
        }
        if sub.len() > 2 {
            remaining = &sub[2..];
        } else {
            break;
        }
    }
    None
}

fn extract_title(content: &str) -> (String, String) {
    let trimmed = content.trim();
    let raw_title = trimmed.lines().next().unwrap_or("").to_string();
    let clean = raw_title.trim_start_matches(|c| c == '#' || c == ' ' || c == '*' || c == '_').to_string();
    (clean, trimmed.to_string())
}

fn extract_tags(content: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let mut in_word = false;
    let mut tag_start = 0;
    let chars: Vec<char> = content.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c == '#' && !in_word && (i == 0 || chars[i-1].is_whitespace() || chars[i-1] == '(') {
            in_word = true;
            tag_start = i + 1;
        } else if in_word {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                continue;
            } else {
                let tag_name: String = chars[tag_start..i].iter().collect();
                if !tag_name.is_empty() {
                    tags.push(tag_name.to_lowercase());
                }
                in_word = false;
            }
        }
    }
    if in_word {
        let tag_name: String = chars[tag_start..].iter().collect();
        if !tag_name.is_empty() {
            tags.push(tag_name.to_lowercase());
        }
    }
    tags.sort();
    tags.dedup();
    tags
}

fn get_month_name(month: u32) -> &'static str {
    match month {
        1 => "January", 2 => "February", 3 => "March", 4 => "April",
        5 => "May", 6 => "June", 7 => "July", 8 => "August",
        9 => "September", 10 => "October", 11 => "November", 12 => "December",
        _ => "Unknown",
    }
}

fn generate_calendar(year: i32, month: u32, memo_dates: &[String]) -> Vec<serde_json::Value> {
    use chrono::Datelike;
    let first_day = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let last_day = {
        if month == 12 {
            chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap().pred_opt().unwrap()
        } else {
            chrono::NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap().pred_opt().unwrap()
        }
    };
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let weekday = first_day.weekday().num_days_from_monday();
    let mut weeks: Vec<Vec<serde_json::Value>> = Vec::new();
    let mut current_week = Vec::new();
    for _ in 0..weekday {
        current_week.push(json!({"day": 0, "date": "", "has_memos": false, "is_today": false, "is_current_month": false}));
    }
    let mut day = first_day;
    while day <= last_day {
        let date_str = day.format("%Y-%m-%d").to_string();
        let has_memos = memo_dates.contains(&date_str);
        let is_today = date_str == today;
        current_week.push(json!({
            "day": day.day(),
            "date": date_str,
            "has_memos": has_memos,
            "is_today": is_today,
            "is_current_month": true,
        }));
        if current_week.len() == 7 {
            weeks.push(current_week);
            current_week = Vec::new();
        }
        day = day.succ_opt().unwrap();
    }
    if !current_week.is_empty() {
        while current_week.len() < 7 {
            current_week.push(json!({"day": 0, "date": "", "has_memos": false, "is_today": false, "is_current_month": false}));
        }
        weeks.push(current_week);
    }
    weeks.into_iter().map(|w| json!(w)).collect()
}

fn get_date_label(date: &str) -> String {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let yesterday = (chrono::Utc::now() - chrono::Duration::days(1)).format("%Y-%m-%d").to_string();
    if date == today {
        "Today".to_string()
    } else if date == yesterday {
        "Yesterday".to_string()
    } else if let Ok(dt) = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        dt.format("%b %d, %Y").to_string()
    } else {
        date.to_string()
    }
}

fn relative_time(datetime: &str) -> String {
    let now = chrono::Utc::now();
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(datetime, "%Y-%m-%d %H:%M:%S") {
        let dt_utc = dt.and_utc();
        let seconds = (now - dt_utc).num_seconds().max(0);
        if seconds < 60 {
            format!("{}s", seconds)
        } else if seconds < 3600 {
            format!("{}m", seconds / 60)
        } else if seconds < 86400 {
            format!("{}h", seconds / 3600)
        } else if seconds < 172800 {
            "yesterday".to_string()
        } else {
            format!("{}d", seconds / 86400)
        }
    } else {
        datetime.to_string()
    }
}

fn avatar_char(username: &str) -> String {
    username.chars().next().unwrap_or('?').to_uppercase().to_string()
}

fn generate_captcha_data() -> (String, String) {
    use base64::Engine as _;
    let mut captcha = captcha::Captcha::new();
    captcha.add_chars(5);
    captcha.view(220, 80);
    let chars = captcha.chars_as_string();
    let png_bytes = captcha.as_png().unwrap_or_default();
    let base64_image = base64::engine::general_purpose::STANDARD.encode(&png_bytes);
    let img_data_url = format!("data:image/png;base64,{}", base64_image);
    (chars, img_data_url)
}

fn get_client_ip(headers: &HeaderMap, connect_info: &ConnectInfo<SocketAddr>) -> String {
    if let Some(xff) = headers.get("x-forwarded-for").and_then(|h| h.to_str().ok()) {
        if let Some(ip) = xff.split(',').next() {
            let trimmed = ip.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }
    if let Some(real_ip) = headers.get("x-real-ip").and_then(|h| h.to_str().ok()) {
        let trimmed = real_ip.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    connect_info.ip().to_string()
}

fn group_memos_by_date(db: &db::Database, memos: &[(i64, String, String, String, String)]) -> Vec<serde_json::Value> {
    use std::collections::BTreeMap;
    let mut groups: BTreeMap<String, Vec<serde_json::Value>> = BTreeMap::new();
    for (id, title, content, visibility, created_at) in memos {
        let date = created_at[..10].to_string();
        let (cleaned_content, resources) = process_memo_content(db, content);
        let content_html = render_markdown(&cleaned_content);
        let created_at_relative = relative_time(created_at);
        let tags = db.get_memo_tags(*id).unwrap_or_default();
        groups.entry(date).or_default().push(json!({
            "id": id,
            "title": title,
            "content": content,
            "content_html": content_html,
            "visibility": visibility,
            "created_at": created_at,
            "created_at_relative": created_at_relative,
            "resources": resources,
            "tags": tags,
        }));
    }
    groups.into_iter().rev().map(|(date, memo_list)| {
        json!({
            "date": date,
            "label": get_date_label(&date),
            "memos": memo_list,
        })
    }).collect()
}

fn get_session_user_id(headers: &HeaderMap) -> Option<i64> {
    let cookie = headers.get(header::COOKIE)?.to_str().ok()?;
    for pair in cookie.split(';') {
        let pair = pair.trim();
        if let Some((name, value)) = pair.split_once('=') {
            if name.trim() == "session" {
                return auth::verify_session_token(value.trim());
            }
        }
    }
    None
}

fn is_valid_session(headers: &HeaderMap, db: &db::Database) -> bool {
    match get_session_user_id(headers) {
        Some(id) => db.get_user_by_id(id).ok().flatten().is_some(),
        None => false,
    }
}

fn redirect_to_app() -> Redirect {
    Redirect::to("/app")
}

fn session_cookie(user_id: i64) -> (HeaderMap, StatusCode) {
    let token = auth::create_session_token(user_id);
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        format!(
            "session={}; HttpOnly; Path=/; SameSite=Lax; Max-Age=604800",
            token
        )
        .parse()
        .unwrap(),
    );
    (headers, StatusCode::FOUND)
}

fn logout_cookie() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        "session=; HttpOnly; Path=/; SameSite=Lax; Max-Age=0"
            .parse()
            .unwrap(),
    );
    headers
}

async fn get_login(headers: HeaderMap, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if is_valid_session(&headers, &state.db) {
        return redirect_to_app().into_response();
    }
    state.templates.render("login", &json!({})).into_response()
}

async fn post_login(
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<AppState>>,
    Form(form): Form<LoginForm>,
) -> impl IntoResponse {
    if is_valid_session(&headers, &state.db) {
        return redirect_to_app().into_response();
    }
    let client_ip = get_client_ip(&headers, &ConnectInfo(addr));
    let allowed = state.db.check_and_record_rate_limit(&client_ip, "login", 20, 3600).unwrap_or(true);
    if !allowed {
        return state.templates.render("login", &json!({"error": "Too many login attempts. Please try again later."})).into_response();
    }
    match state.db.get_user_by_username(&form.username) {
        Ok(Some((id, _, hash))) => match auth::verify_password(&form.password, &hash) {
            Ok(true) => {
                let (mut resp_headers, status) = session_cookie(id);
                resp_headers.insert(header::LOCATION, "/app".parse().unwrap());
                (status, resp_headers).into_response()
            }
            _ => state.templates.render("login", &json!({"error": "Invalid credentials"})).into_response(),
        },
        _ => state.templates.render("login", &json!({"error": "Invalid credentials"})).into_response(),
    }
}

async fn get_register(headers: HeaderMap, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if is_valid_session(&headers, &state.db) {
        return redirect_to_app().into_response();
    }
    let (answer, img_url) = generate_captcha_data();
    let token = auth::create_captcha_token(&answer);
    let cookie = format!("captcha={}; HttpOnly; Path=/; SameSite=Lax; Max-Age=300", token);
    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(header::SET_COOKIE, cookie.parse().unwrap());
    
    (resp_headers, state.templates.render("register", &json!({
        "captcha_image": img_url,
    }))).into_response()
}

async fn post_register(
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<AppState>>,
    Form(form): Form<RegisterForm>,
) -> impl IntoResponse {
    if is_valid_session(&headers, &state.db) {
        return redirect_to_app().into_response();
    }
    let client_ip = get_client_ip(&headers, &ConnectInfo(addr));

    let allowed = state.db.check_and_record_rate_limit(&client_ip, "signup", 5, 3600).unwrap_or(true);
    if !allowed {
        let (answer, img_url) = generate_captcha_data();
        let token = auth::create_captcha_token(&answer);
        let cookie = format!("captcha={}; HttpOnly; Path=/; SameSite=Lax; Max-Age=300", token);
        let mut resp_headers = HeaderMap::new();
        resp_headers.insert(header::SET_COOKIE, cookie.parse().unwrap());
        return (resp_headers, state.templates.render("register", &json!({
            "error": "Rate limit exceeded (5 registrations/hour). Please try again later.",
            "captcha_image": img_url,
        }))).into_response();
    }

    let captcha_cookie = headers.get(header::COOKIE)
        .and_then(|c| c.to_str().ok())
        .and_then(|c| c.split(';').find(|s| s.trim().starts_with("captcha=")))
        .map(|s| s.trim()["captcha=".len()..].to_string());

    let is_captcha_valid = match captcha_cookie {
        Some(token) => auth::verify_captcha_token(&token, &form.captcha_answer),
        None => false,
    };

    if !is_captcha_valid {
        let (answer, img_url) = generate_captcha_data();
        let token = auth::create_captcha_token(&answer);
        let cookie = format!("captcha={}; HttpOnly; Path=/; SameSite=Lax; Max-Age=300", token);
        let mut resp_headers = HeaderMap::new();
        resp_headers.insert(header::SET_COOKIE, cookie.parse().unwrap());
        return (resp_headers, state.templates.render("register", &json!({
            "error": "Incorrect or expired captcha answer",
            "captcha_image": img_url,
        }))).into_response();
    }

    if form.username.trim().is_empty() || form.password.len() < 4 {
        let (answer, img_url) = generate_captcha_data();
        let token = auth::create_captcha_token(&answer);
        let cookie = format!("captcha={}; HttpOnly; Path=/; SameSite=Lax; Max-Age=300", token);
        let mut resp_headers = HeaderMap::new();
        resp_headers.insert(header::SET_COOKIE, cookie.parse().unwrap());
        return (resp_headers, state.templates.render("register", &json!({
            "error": "Username required and password must be at least 4 characters",
            "captcha_image": img_url,
        }))).into_response();
    }

    match auth::hash_password(&form.password) {
        Ok(hash) => match state.db.create_user(form.username.trim(), &hash) {
            Ok(user_id) => {
                let (mut resp_headers, status) = session_cookie(user_id);
                resp_headers.append(
                    header::SET_COOKIE,
                    "captcha=; HttpOnly; Path=/; SameSite=Lax; Max-Age=0".parse().unwrap()
                );
                resp_headers.insert(header::LOCATION, "/app".parse().unwrap());
                (status, resp_headers).into_response()
            }
            Err(e) => {
                let msg = if e.to_string().contains("UNIQUE") { "Username already taken" } else { "Registration failed" };
                let (answer, img_url) = generate_captcha_data();
                let token = auth::create_captcha_token(&answer);
                let cookie = format!("captcha={}; HttpOnly; Path=/; SameSite=Lax; Max-Age=300", token);
                let mut resp_headers = HeaderMap::new();
                resp_headers.insert(header::SET_COOKIE, cookie.parse().unwrap());
                (resp_headers, state.templates.render("register", &json!({
                    "error": msg,
                    "captcha_image": img_url,
                }))).into_response()
            }
        },
        Err(_) => {
            let (answer, img_url) = generate_captcha_data();
            let token = auth::create_captcha_token(&answer);
            let cookie = format!("captcha={}; HttpOnly; Path=/; SameSite=Lax; Max-Age=300", token);
            let mut resp_headers = HeaderMap::new();
            resp_headers.insert(header::SET_COOKIE, cookie.parse().unwrap());
            (resp_headers, state.templates.render("register", &json!({
                "error": "Registration failed",
                "captcha_image": img_url,
            }))).into_response()
        }
    }
}

#[derive(Deserialize)]
struct SharePasswordForm {
    password: Option<String>,
}

async fn get_share_note(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let current_user_id = get_session_user_id(&headers);
    let note = match state.db.get_memo_public(id) {
        Ok(Some(n)) => n,
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    let (memo_id, title, content, visibility, _password_hash, created_at, owner_id, owner_username) = note;

    let is_owner = current_user_id == Some(owner_id);

    if visibility == "private" {
        if !is_owner {
            return (StatusCode::FORBIDDEN, "This note is private.").into_response();
        }
    } else if visibility == "protected" {
        if !is_owner {
            let cookie_name = format!("note_auth_{}", memo_id);
            let has_access = headers.get(header::COOKIE)
                .and_then(|c| c.to_str().ok())
                .and_then(|c| c.split(';').find(|s| s.trim().starts_with(&format!("{}=", cookie_name))))
                .map(|s| s.trim()[cookie_name.len() + 1..].to_string())
                .map(|token| auth::verify_note_auth_token(&token, memo_id))
                .unwrap_or(false);

            if !has_access {
                return state.templates.render("share_password", &json!({
                    "id": memo_id,
                })).into_response();
            }
        }
    }

    let avatar = avatar_char(&owner_username);
    let (cleaned_content, resources) = process_memo_content(&state.db, &content);
    let content_html = render_markdown(&cleaned_content);
    let tags = state.db.get_memo_tags(memo_id).unwrap_or_default();
    
    state.templates.render("share_note", &json!({
        "id": memo_id,
        "title": title,
        "content_html": content_html,
        "visibility": visibility,
        "created_at": created_at,
        "username": owner_username,
        "avatar": avatar,
        "tags": tags,
        "resources": resources,
    })).into_response()
}

async fn post_share_note(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Form(form): Form<SharePasswordForm>,
) -> impl IntoResponse {
    let note = match state.db.get_memo_public(id) {
        Ok(Some(n)) => n,
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    let (memo_id, _title, _content, visibility, password_hash, _created_at, _owner_id, _owner_username) = note;

    if visibility != "protected" {
        return Redirect::to(&format!("/share/{}", memo_id)).into_response();
    }

    let password = form.password.unwrap_or_default();
    let hash = password_hash.unwrap_or_default();

    match auth::verify_password(&password, &hash) {
        Ok(true) => {
            let token = auth::create_note_auth_token(memo_id);
            let cookie_name = format!("note_auth_{}", memo_id);
            let cookie = format!("{}={}; HttpOnly; Path=/; SameSite=Lax; Max-Age=604800", cookie_name, token);
            let mut headers = HeaderMap::new();
            headers.insert(header::SET_COOKIE, cookie.parse().unwrap());
            headers.insert(header::LOCATION, format!("/share/{}", memo_id).parse().unwrap());
            (StatusCode::FOUND, headers).into_response()
        }
        _ => {
            state.templates.render("share_password", &json!({
                "id": memo_id,
                "error": "Incorrect password. Please try again.",
            })).into_response()
        }
    }
}

async fn get_logout() -> impl IntoResponse {
    let headers = logout_cookie();
    (headers, Redirect::to("/login"))
}

async fn get_app(headers: HeaderMap, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let user_id = match get_session_user_id(&headers) {
        Some(id) => id,
        None => return Redirect::to("/login").into_response(),
    };
    let user = match state.db.get_user_by_id(user_id) {
        Ok(Some(u)) => u,
        _ => return Redirect::to("/login").into_response(),
    };
    let avatar = avatar_char(&user.1);
    let memos = state.db.get_memos(user_id).unwrap_or_default();
    let memo_groups = group_memos_by_date(&state.db, &memos);
    state.templates.render("timeline", &json!({
        "username": user.1,
        "avatar": avatar,
        "memo_groups": memo_groups,
    })).into_response()
}

async fn post_memos(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<MemoForm>,
) -> impl IntoResponse {
    let user_id = match get_session_user_id(&headers) {
        Some(id) => id,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let raw = form.content.trim().to_string();
    if raw.is_empty() {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let mut visibility = form.visibility.as_deref().unwrap_or("private");
    visibility = match visibility { "public" | "protected" => visibility, _ => "private" };
    
    let mut password_hash = None;
    let mut vis_str = visibility.to_string();
    if visibility == "protected" {
        if let Some(pwd) = form.visibility_password.as_deref().filter(|p| !p.trim().is_empty()) {
            if let Ok(hash) = auth::hash_password(pwd) {
                password_hash = Some(hash);
            } else {
                vis_str = "private".to_string();
            }
        } else {
            vis_str = "private".to_string();
        }
    }
    let visibility = vis_str.as_str();

    let (title, content) = extract_title(&raw);
    let tags = extract_tags(&content);
    let created_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let (cleaned_content, resources) = process_memo_content(&state.db, &content);
    let content_html = render_markdown(&cleaned_content);
    let created_at_relative = relative_time(&created_at);
    match state.db.create_memo(user_id, &title, &content, visibility, password_hash.as_deref()) {
        Ok(memo_id) => {
            state.db.set_memo_tags(memo_id, &tags).ok();
            let user = match state.db.get_user_by_id(user_id) {
                Ok(Some(u)) => u,
                _ => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };
            let avatar = avatar_char(&user.1);
            let mut response_headers = HeaderMap::new();
            response_headers.insert(
                header::HeaderName::from_static("hx-trigger"),
                "memoUpdated".parse().unwrap(),
            );
            (response_headers, state.templates.render("memo_fragment", &json!({
                "id": memo_id,
                "content": content,
                "content_html": content_html,
                "created_at": created_at,
                "created_at_relative": created_at_relative,
                "visibility": visibility,
                "username": user.1,
                "avatar": avatar,
                "resources": resources,
                "tags": tags,
            }))).into_response()
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn put_memos(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Form(form): Form<MemoForm>,
) -> impl IntoResponse {
    let user_id = match get_session_user_id(&headers) {
        Some(id) => id,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let raw = form.content.trim().to_string();
    if raw.is_empty() {
        return StatusCode::BAD_REQUEST.into_response();
    }
    
    let current_memo = state.db.get_memo_by_id(id, user_id).ok().flatten();
    let current_visibility = current_memo.as_ref().map(|m| m.3.as_str()).unwrap_or("private");
    
    let mut visibility = form.visibility.as_deref().unwrap_or("private");
    visibility = match visibility { "public" | "protected" => visibility, _ => "private" };
    
    let mut password_hash = None;
    let mut vis_str = visibility.to_string();
    if visibility == "protected" {
        if let Some(pwd) = form.visibility_password.as_deref().filter(|p| !p.trim().is_empty()) {
            if let Ok(hash) = auth::hash_password(pwd) {
                password_hash = Some(hash);
            }
        }
        if password_hash.is_none() && current_visibility == "protected" {
            if let Ok(Some(existing_memo)) = state.db.get_memo_public(id) {
                password_hash = existing_memo.4;
            }
        }
        if password_hash.is_none() {
            vis_str = "private".to_string();
        }
    }
    let visibility = vis_str.as_str();

    let (_title, content) = extract_title(&raw);
    let tags = extract_tags(&content);
    match state.db.update_memo(id, user_id, &content) {
        Ok(()) => {
            state.db.update_memo_visibility(id, user_id, visibility, password_hash.as_deref()).ok();
            state.db.set_memo_tags(id, &tags).ok();
            let user = match state.db.get_user_by_id(user_id) {
                Ok(Some(u)) => u,
                _ => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };
            let avatar = avatar_char(&user.1);
            let memo = state.db.get_memo_by_id(id, user_id).ok().flatten();
            let created_at = memo.as_ref().map(|m| m.4.clone()).unwrap_or_default();
            let (cleaned_content, resources) = process_memo_content(&state.db, &content);
            let content_html = render_markdown(&cleaned_content);
            let created_at_relative = relative_time(&created_at);
            let mut response_headers = HeaderMap::new();
            response_headers.insert(
                header::HeaderName::from_static("hx-trigger"),
                "memoUpdated".parse().unwrap(),
            );
            (response_headers, state.templates.render("memo_fragment", &json!({
                "id": id,
                "content": content,
                "content_html": content_html,
                "created_at": created_at,
                "created_at_relative": created_at_relative,
                "visibility": visibility,
                "username": user.1,
                "avatar": avatar,
                "resources": resources,
                "tags": tags,
            }))).into_response()
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn delete_memo(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let user_id = match get_session_user_id(&headers) {
        Some(id) => id,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };
    match state.db.delete_memo(id, user_id) {
        Ok(()) => {
            let mut resp_headers = HeaderMap::new();
            resp_headers.insert(
                header::HeaderName::from_static("hx-trigger"),
                "memoUpdated".parse().unwrap(),
            );
            (resp_headers, "").into_response()
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

fn resource_storage_dir() -> PathBuf {
    let base = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "data/minimamemosa.db".to_string());
    let dir = std::path::Path::new(&base).parent().unwrap_or(std::path::Path::new("data"));
    dir.join("resources")
}

async fn post_resources(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let user_id = match get_session_user_id(&headers) {
        Some(id) => id,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let storage = resource_storage_dir();
    tokio::fs::create_dir_all(&storage).await.ok();
    let mut resources = Vec::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        let original_name = field.file_name().unwrap_or("file").to_string();
        let mime_type = field.content_type().unwrap_or("application/octet-stream").to_string();
        let data: Bytes = match field.bytes().await {
            Ok(d) => d,
            Err(_) => continue,
        };
        let size = data.len() as i64;
        let ext = std::path::Path::new(&original_name).extension()
            .and_then(|e| e.to_str()).unwrap_or("bin");
        let filename = format!("{}.{}", uuid_v4(), ext);
        let filepath = storage.join(&filename);
        if tokio::fs::write(&filepath, &data).await.is_err() {
            continue;
        }
        match state.db.create_resource(user_id, &filename, &original_name, &mime_type, size) {
            Ok(id) => {
                let is_image = mime_type.starts_with("image/");
                let markdown = if is_image {
                    format!("![{}](/resources/{})", original_name, id)
                } else {
                    format!("[{}](/resources/{})", original_name, id)
                };
                resources.push(json!({
                    "id": id,
                    "markdown": markdown,
                    "filename": filename,
                    "original_name": original_name,
                    "mime_type": mime_type,
                    "size": size,
                }));
            }
            Err(_) => {
                tokio::fs::remove_file(&filepath).await.ok();
            }
        }
    }
    (StatusCode::OK, Json(json!({"resources": resources}))).into_response()
}

fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    format!("{:016x}{:016x}", (nanos >> 64) as u64, nanos as u64)
}

async fn get_resource(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let current_user_id = get_session_user_id(&headers);
    let res = match state.db.get_resource_public(id) {
        Ok(Some(r)) => r,
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    let (res_id, filename, _original_name, mime_type, _size, owner_id) = res;

    let mut authorized = false;
    if Some(owner_id) == current_user_id {
        authorized = true;
    } else {
        let ref_pattern = format!("/resources/{}", res_id);
        if let Ok(memos) = state.db.get_memos_referencing_resource(&ref_pattern) {
            for (memo_id, visibility, memo_owner_id) in memos {
                if visibility == "public" {
                    authorized = true;
                    break;
                } else if visibility == "protected" {
                    let is_memo_owner = current_user_id == Some(memo_owner_id);
                    if is_memo_owner {
                        authorized = true;
                        break;
                    }
                    let cookie_name = format!("note_auth_{}", memo_id);
                    let has_access = headers.get(header::COOKIE)
                        .and_then(|c| c.to_str().ok())
                        .and_then(|c| c.split(';').find(|s| s.trim().starts_with(&format!("{}=", cookie_name))))
                        .map(|s| s.trim()[cookie_name.len() + 1..].to_string())
                        .map(|token| auth::verify_note_auth_token(&token, memo_id))
                        .unwrap_or(false);
                    if has_access {
                        authorized = true;
                        break;
                    }
                } else if visibility == "private" {
                    if current_user_id == Some(memo_owner_id) {
                        authorized = true;
                        break;
                    }
                }
            }
        }
    }

    if !authorized {
        return StatusCode::FORBIDDEN.into_response();
    }

    let filepath = resource_storage_dir().join(&filename);
    let data = match tokio::fs::read(&filepath).await {
        Ok(d) => d,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };
    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(
        header::CONTENT_TYPE,
        mime_type.parse().unwrap_or(header::HeaderValue::from_static("application/octet-stream")),
    );
    resp_headers.insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("public, max-age=31536000"),
    );
    (resp_headers, data).into_response()
}

async fn bulk_delete_resources(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let user_id = match get_session_user_id(&headers) {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "unauthorized"}))).into_response(),
    };
    let ids = match payload.get("ids").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return (StatusCode::BAD_REQUEST, Json(json!({"error": "ids required"}))).into_response(),
    };
    let storage = resource_storage_dir();
    let mut deleted = 0;
    for id_val in ids {
        if let Some(id) = id_val.as_i64() {
            if let Ok(Some(res)) = state.db.get_resource(id, user_id) {
                let filepath = storage.join(&res.1);
                tokio::fs::remove_file(&filepath).await.ok();
                if state.db.delete_resource(id, user_id).is_ok() {
                    deleted += 1;
                }
            }
        }
    }
    (StatusCode::OK, Json(json!({"deleted": deleted}))).into_response()
}

async fn delete_resource(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let user_id = match get_session_user_id(&headers) {
        Some(id) => id,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let res = match state.db.get_resource(id, user_id) {
        Ok(Some(r)) => r,
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    let filepath = resource_storage_dir().join(&res.1);
    tokio::fs::remove_file(&filepath).await.ok();
    match state.db.delete_resource(id, user_id) {
        Ok(()) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn get_resources_feed(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let user_id = match get_session_user_id(&headers) {
        Some(id) => id,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let resources = state.db.get_resources(user_id).unwrap_or_default();
    let resource_list: Vec<serde_json::Value> = resources.iter().map(|(id, filename, original_name, mime_type, size, created_at)| {
        let size_str = if *size < 1024 { format!("{} B", size) }
            else if *size < 1048576 { format!("{:.1} KB", *size as f64 / 1024.0) }
            else { format!("{:.1} MB", *size as f64 / 1048576.0) };
        let is_image = mime_type.starts_with("image/");
        json!({
            "id": id,
            "filename": filename,
            "original_name": original_name,
            "mime_type": mime_type,
            "size_str": size_str,
            "is_image": is_image,
            "created_at": created_at,
        })
    }).collect();
    state.templates.render("resources_panel", &json!({
        "resources": resource_list,
    })).into_response()
}

async fn get_notes_panel(headers: HeaderMap, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let user_id = match get_session_user_id(&headers) {
        Some(id) => id,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let user = match state.db.get_user_by_id(user_id) {
        Ok(Some(u)) => u,
        _ => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let username = user.1;
    let avatar = avatar_char(&username);
    let memos = state.db.get_sidebar_memos(user_id, 100).unwrap_or_default();
    let notes: Vec<serde_json::Value> = memos.iter().map(|(id, _title, content, visibility, created_at)| {
        let (title, _) = extract_title(content);
        let tags = extract_tags(content);
        let first_image_id = extract_first_image_id(content);
        let human_date = match chrono::NaiveDateTime::parse_from_str(created_at, "%Y-%m-%d %H:%M:%S") {
            Ok(dt) => dt.format("%b %d, %Y %I:%M %p").to_string(),
            Err(_) => created_at.clone(),
        };
        json!({
            "id": id,
            "title": title,
            "tags": tags,
            "first_image_id": first_image_id,
            "created_at": human_date,
            "visibility": visibility,
        })
    }).collect();
    state.templates.render("notes_panel", &json!({
        "notes": notes,
        "username": username,
        "avatar": avatar,
    })).into_response()
}

async fn get_note_detail(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let user_id = match get_session_user_id(&headers) {
        Some(id) => id,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };
    match state.db.get_memo_by_id(id, user_id) {
        Ok(Some((_id, _title, content, _visibility, created_at))) => {
            let (cleaned_content, resources) = process_memo_content(&state.db, &content);
            let content_html = render_markdown(&cleaned_content);
            state.templates.render("note_detail", &json!({
                "content_html": content_html,
                "created_at": created_at,
                "resources": resources,
            })).into_response()
        }
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn get_memos_feed(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let user_id = match get_session_user_id(&headers) {
        Some(id) => id,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let user = match state.db.get_user_by_id(user_id) {
        Ok(Some(u)) => u,
        _ => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let avatar = avatar_char(&user.1);
    let memos = state.db.get_memos(user_id).unwrap_or_default();
    let memo_groups = group_memos_by_date(&state.db, &memos);
    state.templates.render("memos_feed", &json!({
        "memo_groups": memo_groups,
        "username": user.1,
        "avatar": avatar,
    })).into_response()
}

async fn get_search(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let user_id = match get_session_user_id(&headers) {
        Some(id) => id,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let memos = if let Some(tag) = params.get("tag") {
        state.db.get_memos_by_tag(user_id, tag).unwrap_or_default()
    } else if let Some(date) = params.get("date") {
        state.db.get_memos_by_date(user_id, date).unwrap_or_default()
    } else if let Some(q) = params.get("q") {
        if q.is_empty() {
            state.db.get_memos(user_id).unwrap_or_default()
        } else {
            state.db.search_memos(user_id, q).unwrap_or_default()
        }
    } else {
        state.db.get_memos(user_id).unwrap_or_default()
    };
    let user = match state.db.get_user_by_id(user_id) {
        Ok(Some(u)) => u,
        _ => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let avatar = avatar_char(&user.1);
    let memo_groups = group_memos_by_date(&state.db, &memos);
    state.templates.render("memos_feed", &json!({
        "memo_groups": memo_groups,
        "username": user.1,
        "avatar": avatar,
    })).into_response()
}

async fn get_memos_json(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let user_id = match get_session_user_id(&headers) {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, Json(json!([]))).into_response(),
    };
    let memos = if let Some(q) = params.get("q").filter(|s| !s.is_empty()) {
        state.db.search_memos(user_id, q).unwrap_or_default()
    } else {
        state.db.get_memos(user_id).unwrap_or_default()
    };
    let list: Vec<serde_json::Value> = memos.iter().map(|(id, title, content, visibility, _created_at)| {
        json!({"id": id, "title": title, "content": content, "visibility": visibility})
    }).collect();
    (StatusCode::OK, Json(json!(list))).into_response()
}

async fn get_sidebar_timeline(headers: HeaderMap, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let user_id = match get_session_user_id(&headers) {
        Some(id) => id,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let now = chrono::Utc::now();
    let year = now.year();
    let month = now.month();
    let memo_dates = state.db.get_memo_dates_in_month(user_id, year, month).unwrap_or_default();
    let calendar_weeks = generate_calendar(year, month, &memo_dates);
    let tags = state.db.get_user_tags(user_id).unwrap_or_default();
    let tag_list: Vec<serde_json::Value> = tags.iter().map(|(name, count)| {
        json!({"name": name, "count": count})
    }).collect();
    state.templates.render("sidebar_timeline", &json!({
        "year": year,
        "month": month,
        "month_label": format!("{} {}", get_month_name(month), year),
        "calendar_weeks": calendar_weeks,
        "tags": tag_list,
    })).into_response()
}

#[tokio::main]
async fn main() {
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "data/minimamemosa.db".to_string());
    if let Some(parent) = std::path::Path::new(&db_path).parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let state = Arc::new(AppState {
        db: db::Database::new(&db_path).expect("Failed to initialize database"),
        templates: Templates::new(),
    });
    let app = Router::new()
        .route("/", get(|| async { Redirect::to("/app") }))
        .route("/login", get(get_login).post(post_login))
        .route("/register", get(get_register).post(post_register))
        .route("/share/:id", get(get_share_note).post(post_share_note))
        .route("/logout", get(get_logout))
        .route("/app", get(get_app))
        .route("/memos", post(post_memos))
        .route("/memos/:id", put(put_memos).delete(delete_memo))
        .route("/resources", post(post_resources))
        .route("/resources/bulk-delete", post(bulk_delete_resources))
        .route("/resources/:id", get(get_resource).delete(delete_resource))
        .route("/resources-feed", get(get_resources_feed))
        .route("/notes-panel", get(get_notes_panel))
        .route("/note/:id", get(get_note_detail))
        .route("/memos-feed", get(get_memos_feed))
        .route("/search", get(get_search))
        .route("/memos-json", get(get_memos_json))
        .route("/sidebar-timeline", get(get_sidebar_timeline))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);
    let addr = "0.0.0.0:3000";
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Error: Cannot bind to {}: {}", addr, e);
            eprintln!("Kill any stale process: pkill -f minimamemosa && cargo run");
            std::process::exit(1);
        }
    };
    println!("MinimaMemosa listening on http://{}", addr);
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}
