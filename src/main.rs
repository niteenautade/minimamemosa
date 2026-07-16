mod auth;
mod db;
mod templates;

use std::sync::Arc;

use axum::{
    body::{Bytes, Body},
    extract::{Form, Multipart, Path, State, ConnectInfo},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
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
    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
    options.insert(pulldown_cmark::Options::ENABLE_TABLES);
    options.insert(pulldown_cmark::Options::ENABLE_TASKLISTS);

    let parser = pulldown_cmark::Parser::new_ext(text, options).map(|event| match event {
        pulldown_cmark::Event::SoftBreak => pulldown_cmark::Event::HardBreak,
        _ => event,
    });
    
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
    let mut resources: Vec<serde_json::Value> = Vec::new();
    let mut cleaned = String::with_capacity(content.len());
    let mut last_end = 0;
    let mut search_pos = 0;

    while search_pos < content.len() {
        let remaining = &content[search_pos..];
        if let Some(pos) = remaining.find("/resources/") {
            let abs_pos = search_pos + pos;
            let id_start = abs_pos + "/resources/".len();
            let after = &content[id_start..];

            let id_len = after.chars().take_while(|c| c.is_ascii_digit()).count();
            if id_len > 0 {
                let id: i64 = after[..id_len].parse().unwrap();
                let after_id = &after[id_len..];

                let ref_end_offset = after_id.find(')')
                    .or_else(|| after_id.find('>'))
                    .or_else(|| after_id.find('"'));

                if let Some(eo) = ref_end_offset {
                    let full_ref_end = id_start + id_len + eo + 1;
                    let resource = db.get_resource_public(id).ok().flatten();

                    if let Some(r) = resource {
                        let is_img = r.3.starts_with("image/");
                        resources.push(json!({
                            "id": r.0,
                            "filename": r.1,
                            "original_name": r.2,
                            "mime_type": r.3,
                            "is_image": is_img,
                            "size": format_bytes(r.4),
                        }));
                        cleaned.push_str(&content[last_end..full_ref_end]);
                    } else {
                        let before = &content[last_end..abs_pos];
                        let mut ref_start = None;
                        if let Some(cb) = before.rfind(']') {
                            let before_cb = &before[..cb];
                            if let Some(ob) = before_cb.rfind('[') {
                                ref_start = Some(last_end + ob);
                            }
                        }
                        if ref_start.is_none() {
                            if let Some(ts) = before.rfind('<') {
                                ref_start = Some(last_end + ts);
                            }
                        }
                        if let Some(rs) = ref_start {
                            cleaned.push_str(&content[last_end..rs]);
                        } else {
                            cleaned.push_str(&content[last_end..abs_pos]);
                        }
                    }

                    last_end = full_ref_end;
                    search_pos = full_ref_end;
                    continue;
                }
            }
            search_pos = abs_pos + "/resources/".len();
        } else {
            break;
        }
    }

    if last_end < content.len() {
        cleaned.push_str(&content[last_end..]);
    }

    (cleaned.trim().to_string(), resources)
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

fn strip_html(s: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    let mut tag_buf = String::new();
    for c in s.chars() {
        match c {
            '<' => {
                if in_tag {
                    out.push('<');
                    out.push_str(&tag_buf);
                }
                in_tag = true; 
                tag_buf.clear(); 
            }
            '>' => {
                if in_tag {
                    in_tag = false;
                    let t = tag_buf.trim();
                    if t == "p" || t == "/p" || t == "br" || t == "br/" || t == "/div" || t.starts_with("br ") || t.starts_with("/h") || t.starts_with("h") {
                        out.push('\n');
                    }
                } else {
                    out.push('>');
                }
            }
            _ => {
                if in_tag { tag_buf.push(c); }
                else { out.push(c); }
            }
        }
    }
    if in_tag {
        out.push('<');
        out.push_str(&tag_buf);
    }
    out
}

fn extract_title(content: &str) -> (String, String) {
    let trimmed = content.trim();
    let text = strip_html(trimmed);
    let first_line = text.lines().find(|l| !l.trim().is_empty()).unwrap_or("").trim().to_string();
    let clean = first_line.trim_start_matches(|c| c == '#' || c == ' ' || c == '*' || c == '_').trim().to_string();
    let final_title = if clean.is_empty() { "Note".to_string() } else { clean };
    (final_title, trimmed.to_string())
}

fn extract_tags(content: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let mut in_word = false;
    let mut tag_start = 0;
    
    // Strip HTML to prevent tags like <p> from interfering with hashtag boundaries.
    // Also handle HTML entities that might act as whitespace.
    let clean_content = strip_html(content).replace("&nbsp;", " ");
    
    let chars: Vec<char> = clean_content.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c == '#' && !in_word && (i == 0 || !chars[i-1].is_alphanumeric()) {
            in_word = true;
            tag_start = i + 1;
        } else if in_word {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                continue;
            } else {
                let tag_name: String = chars[tag_start..i].iter().collect();
                let tag_name = tag_name.trim_end_matches('_');
                if !tag_name.is_empty() {
                    tags.push(tag_name.to_lowercase());
                }
                in_word = false;
            }
        }
    }
    if in_word {
        let tag_name: String = chars[tag_start..chars.len()].iter().collect();
        let tag_name = tag_name.trim_end_matches('_');
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

fn generate_calendar(year: i32, month: u32, memo_dates: &[String], selected_date: Option<&str>) -> Vec<serde_json::Value> {
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
        current_week.push(json!({"day": 0, "date": "", "has_memos": false, "is_today": false, "is_selected": false, "is_future": false, "is_current_month": false}));
    }
    let mut day = first_day;
    while day <= last_day {
        let date_str = day.format("%Y-%m-%d").to_string();
        let has_memos = memo_dates.contains(&date_str);
        let is_today = date_str == today;
        let is_selected = selected_date.map_or(false, |sd| date_str == sd);
        let is_future = date_str > today;
        current_week.push(json!({
            "day": day.day(),
            "date": date_str,
            "has_memos": has_memos,
            "is_today": is_today,
            "is_selected": is_selected,
            "is_future": is_future,
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
            current_week.push(json!({"day": 0, "date": "", "has_memos": false, "is_today": false, "is_selected": false, "is_future": false, "is_current_month": false}));
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
    Redirect::to("/app/timeline")
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

#[derive(serde::Deserialize)]
struct SharePasswordJson {
    password: Option<String>,
}

async fn put_share_note(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<SharePasswordJson>,
) -> impl IntoResponse {
    let user_id = match get_session_user_id(&headers) {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };

    let _memo = match state.db.get_memo_by_id(id, user_id) {
        Ok(Some(m)) => m,
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "Note not found"}))).into_response(),
    };

    let password = match body.password.as_deref().filter(|p| p.len() >= 4) {
        Some(pwd) => pwd.to_string(),
        None => return (StatusCode::BAD_REQUEST, Json(json!({"error": "Password must be at least 4 characters"}))).into_response(),
    };

    let password_hash = match auth::hash_password(&password) {
        Ok(h) => h,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to hash password"}))).into_response(),
    };

    match state.db.update_memo_visibility(id, user_id, "protected", Some(&password_hash)) {
        Ok(()) => (StatusCode::OK, Json(json!({"url": format!("/share/{}", id)}))).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to update note"}))).into_response(),
    }
}

async fn get_logout() -> impl IntoResponse {
    let headers = logout_cookie();
    (headers, Redirect::to("/login"))
}

async fn render_app_page(
    user_id: i64,
    state: &Arc<AppState>,
    active_panel: &str,
    selected_note_id: Option<i64>,
) -> Response<Body> {
    let user = match state.db.get_user_by_id(user_id) {
        Ok(Some(u)) => u,
        _ => return Redirect::to("/login").into_response(),
    };
    let username = user.1.clone();
    let avatar = avatar_char(&username);
    let limit: i64 = 20;
    let memos = state.db.get_memos_paginated(user_id, limit + 1, 0).unwrap_or_default();
    let has_more = memos.len() as i64 > limit;
    let page_memos: Vec<_> = memos.into_iter().take(limit as usize).collect();
    let memo_groups = group_memos_by_date(&state.db, &page_memos);
    let mut ctx = json!({
        "username": username,
        "avatar": avatar,
        "memo_groups": memo_groups,
        "active_panel": active_panel,
        "next_offset": if has_more { Some(limit) } else { None },
    });
    if let Some(note_id) = selected_note_id {
        if let Ok(Some(memo)) = state.db.get_memo_by_id(note_id, user_id) {
            let (id, title, content, visibility, created_at) = memo;
            let (cleaned_content, resources) = process_memo_content(&state.db, &content);
            let content_html = render_markdown(&cleaned_content);
            let created_at_relative = relative_time(&created_at);
            let tags = state.db.get_memo_tags(id).unwrap_or_default();
            ctx["selected_note"] = json!({
                "id": id,
                "title": title,
                "content": content,
                "content_html": content_html,
                "visibility": visibility,
                "created_at": created_at,
                "created_at_relative": created_at_relative,
                "tags": tags,
                "resources": resources,
                "username": username,
            });
        }
    }
    state.templates.render("timeline", &ctx).into_response()
}

async fn get_app_root(headers: HeaderMap, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let user_id = match get_session_user_id(&headers) {
        Some(id) => id,
        None => return Redirect::to("/login").into_response(),
    };
    render_app_page(user_id, &state, "notes", None).await
}

async fn get_app_timeline(headers: HeaderMap, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let user_id = match get_session_user_id(&headers) {
        Some(id) => id,
        None => return Redirect::to("/login").into_response(),
    };
    render_app_page(user_id, &state, "notes", None).await
}

async fn get_app_notes() -> impl IntoResponse {
    Redirect::to("/app/timeline")
}

async fn get_app_resources(headers: HeaderMap, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let user_id = match get_session_user_id(&headers) {
        Some(id) => id,
        None => return Redirect::to("/login").into_response(),
    };
    render_app_page(user_id, &state, "resources", None).await
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

async fn bulk_delete_memos(
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
    let mut deleted = 0;
    for id_val in ids {
        if let Some(id) = id_val.as_i64() {
            if state.db.delete_memo(id, user_id).is_ok() {
                deleted += 1;
            }
        }
    }
    (StatusCode::OK, Json(json!({"deleted": deleted}))).into_response()
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
    let mut replaced = Vec::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        let original_name = field.file_name().unwrap_or("file").to_string();
        let mime_type = field.content_type().unwrap_or("application/octet-stream").to_string();
        let data: Bytes = match field.bytes().await {
            Ok(d) => d,
            Err(_) => continue,
        };
        let size = data.len() as i64;

        if let Ok(Some((old_id, old_filename))) = state.db.find_resource_by_original_name(user_id, &original_name) {
            let old_filepath = storage.join(&old_filename);
            tokio::fs::remove_file(&old_filepath).await.ok();
            let _ = state.db.delete_resource(old_id, user_id);
            replaced.push(json!({"id": old_id, "original_name": &original_name}));
        }

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
    (StatusCode::OK, Json(json!({"resources": resources, "replaced": replaced}))).into_response()
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
        header::HeaderValue::from_static("no-cache, must-revalidate"),
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
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let user_id = match get_session_user_id(&headers) {
        Some(id) => id,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);
    let limit: i64 = 30;
    let resources = state.db.get_resources_paginated(user_id, limit + 1, offset).unwrap_or_default();
    let has_more = resources.len() as i64 > limit;
    let page: Vec<_> = resources.into_iter().take(limit as usize).collect();
    let resource_list: Vec<serde_json::Value> = page.iter().map(|(id, filename, original_name, mime_type, size, created_at)| {
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
    let partial = offset > 0;
    state.templates.render("resources_panel", &json!({
        "resources": resource_list,
        "offset": offset,
        "next_offset": if has_more { Some(offset + limit) } else { None },
        "partial": partial,
    })).into_response()
}

async fn get_memo_fragment(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let user_id = match get_session_user_id(&headers) {
        Some(id) => id,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let user = match state.db.get_user_by_id(user_id) {
        Ok(Some(u)) => u,
        _ => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let username = user.1;
    match state.db.get_memo_by_id(id, user_id) {
        Ok(Some((_id, _title, content, visibility, created_at))) => {
            let (cleaned_content, resources) = process_memo_content(&state.db, &content);
            let content_html = render_markdown(&cleaned_content);
            let created_at_relative = relative_time(&created_at);
            let tags = state.db.get_memo_tags(id).unwrap_or_default();
            state.templates.render("memo_fragment", &json!({
                "id": id,
                "content": content,
                "content_html": content_html,
                "visibility": visibility,
                "created_at": created_at,
                "created_at_relative": created_at_relative,
                "resources": resources,
                "tags": tags,
                "username": username,
            })).into_response()
        }
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn get_notes_panel(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
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
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);
    let limit: i64 = 30;
    let memos = state.db.get_memos_paginated(user_id, limit + 1, offset).unwrap_or_default();
    let has_more = memos.len() as i64 > limit;
    let page_memos: Vec<_> = memos.into_iter().take(limit as usize).collect();
    let notes: Vec<serde_json::Value> = page_memos.iter().map(|(id, _title, content, visibility, created_at)| {
        let (title, _) = extract_title(content);
        let tags = extract_tags(content);
        let first_image_id = extract_first_image_id(content);
        let search_text = {
            let s = strip_html(content);
            if s.len() > 500 { s[..500].to_string() } else { s }
        };
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
            "search_text": search_text,
        })
    }).collect();
    let partial = offset > 0;
    state.templates.render("notes_panel", &json!({
        "notes": notes,
        "username": username,
        "avatar": avatar,
        "offset": offset,
        "next_offset": if has_more { Some(offset + limit) } else { None },
        "partial": partial,
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

async fn get_memo_edit_form(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let user_id = match get_session_user_id(&headers) {
        Some(id) => id,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let memo = match state.db.get_memo_by_id(id, user_id) {
        Ok(Some(m)) => m,
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    let (id_v, _title, content, visibility, _created_at) = memo;
    let has_password = state.db.get_memo_has_password(id_v, user_id);
    state.templates.render("memo_edit_form", &json!({
        "id": id_v,
        "content": content,
        "visibility": visibility,
        "has_password": has_password,
    })).into_response()
}

async fn get_memos_feed(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
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
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);
    let limit: i64 = 20;
    let memos = state.db.get_memos_paginated(user_id, limit + 1, offset).unwrap_or_default();
    let has_more = memos.len() as i64 > limit;
    let page_memos: Vec<_> = memos.into_iter().take(limit as usize).collect();
    let memo_groups = group_memos_by_date(&state.db, &page_memos);
    state.templates.render("memos_feed", &json!({
        "memo_groups": memo_groups,
        "username": user.1,
        "avatar": avatar,
        "offset": offset,
        "next_offset": if has_more { Some(offset + limit) } else { None },
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

async fn get_calendar(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let user_id = match get_session_user_id(&headers) {
        Some(id) => id,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let now = chrono::Utc::now();
    let year = now.year();
    let month = now.month();
    let memo_dates = if let Some(q) = params.get("q").filter(|s| !s.is_empty()) {
        state.db.get_memo_dates_in_month_for_query(user_id, year, month, q).unwrap_or_default()
    } else {
        state.db.get_memo_dates_in_month(user_id, year, month).unwrap_or_default()
    };
    let selected_date = params.get("selected_date").map(|s| s.as_str());
    let calendar_weeks = generate_calendar(year, month, &memo_dates, selected_date);
    state.templates.render("calendar", &json!({
        "month_label": format!("{} {}", get_month_name(month), year),
        "calendar_weeks": calendar_weeks,
    })).into_response()
}

async fn get_unified_sidebar(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
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
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);
    let limit: i64 = 30;
    let memos = state.db.get_memos_paginated(user_id, limit + 1, offset).unwrap_or_default();
    let has_more = memos.len() as i64 > limit;
    let page_memos: Vec<_> = memos.into_iter().take(limit as usize).collect();
    let notes: Vec<serde_json::Value> = page_memos.iter().map(|(id, _title, content, visibility, created_at)| {
        let (title, _) = extract_title(content);
        let tags = extract_tags(content);
        let first_image_id = extract_first_image_id(content);
        let search_text = {
            let s = strip_html(content);
            if s.len() > 500 { s[..500].to_string() } else { s }
        };
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
            "search_text": search_text,
        })
    }).collect();
    let partial = offset > 0;

    // Calendar data
    let now = chrono::Utc::now();
    let year = now.year();
    let month = now.month();
    let memo_dates = state.db.get_memo_dates_in_month(user_id, year, month).unwrap_or_default();
    let selected_date = params.get("selected_date").map(|s| s.as_str());
    let calendar_weeks = generate_calendar(year, month, &memo_dates, selected_date);

    // Tags
    let tags = state.db.get_user_tags(user_id).unwrap_or_default();
    let tag_list: Vec<serde_json::Value> = tags.iter().map(|(name, count)| {
        json!({"name": name, "count": count})
    }).collect();

    state.templates.render("unified_sidebar", &json!({
        "notes": notes,
        "username": username,
        "avatar": avatar,
        "month_label": format!("{} {}", get_month_name(month), year),
        "calendar_weeks": calendar_weeks,
        "tags": tag_list,
        "offset": offset,
        "next_offset": if has_more { Some(offset + limit) } else { None },
        "partial": partial,
    })).into_response()
}

async fn get_sidebar_timeline(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let user_id = match get_session_user_id(&headers) {
        Some(id) => id,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let now = chrono::Utc::now();
    let year = now.year();
    let month = now.month();
    let memo_dates = state.db.get_memo_dates_in_month(user_id, year, month).unwrap_or_default();
    let selected_date = params.get("selected_date").or_else(|| params.get("date")).map(|s| s.as_str());
    let calendar_weeks = generate_calendar(year, month, &memo_dates, selected_date);
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
        .route("/notes/:id/share", put(put_share_note))
        .route("/logout", get(get_logout))
        .route("/app", get(get_app_root))
        .route("/app/timeline", get(get_app_timeline))
        .route("/app/notes", get(get_app_notes))
        .route("/app/resources", get(get_app_resources))
        .route("/memos", post(post_memos))
        .route("/memos/bulk-delete", post(bulk_delete_memos))
        .route("/memos/:id", put(put_memos).delete(delete_memo))
        .route("/memos/:id/edit", get(get_memo_edit_form))
        .route("/resources", post(post_resources))
        .route("/resources/bulk-delete", post(bulk_delete_resources))
        .route("/resources/:id", get(get_resource).delete(delete_resource))
        .route("/resources-feed", get(get_resources_feed))
        .route("/notes-panel", get(get_notes_panel))
        .route("/note/:id", get(get_note_detail))
        .route("/memos/:id/fragment", get(get_memo_fragment))
        .route("/memos-feed", get(get_memos_feed))
        .route("/search", get(get_search))
        .route("/memos-json", get(get_memos_json))
        .route("/unified-sidebar", get(get_unified_sidebar))
        .route("/sidebar-timeline", get(get_sidebar_timeline))
        .route("/calendar", get(get_calendar))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = match tokio::net::TcpListener::bind(&addr).await {
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── render_markdown Tests ──

    #[test]
    fn test_render_markdown_bold_text() {
        let result = render_markdown("**bold**");
        assert!(result.contains("<strong>bold</strong>"));
    }

    #[test]
    fn test_render_markdown_italic_text() {
        let result = render_markdown("*italic*");
        assert!(result.contains("<em>italic</em>"));
    }

    #[test]
    fn test_render_markdown_code_block() {
        let result = render_markdown("```rust\nfn main() {}\n```");
        assert!(result.contains("<code"));
        assert!(result.contains("fn main()"));
    }

    #[test]
    fn test_render_markdown_link() {
        let result = render_markdown("[link](https://example.com)");
        assert!(result.contains("<a href=\"https://example.com\""));
    }

    #[test]
    fn test_render_markdown_image() {
        let result = render_markdown("![alt](/image.png)");
        assert!(result.contains("<img src=\"/image.png\" alt=\"alt\""));
    }

    #[test]
    fn test_render_markdown_heading() {
        let result = render_markdown("# Heading");
        assert!(result.contains("<h1>Heading</h1>"));
    }

    #[test]
    fn test_render_markdown_list() {
        let result = render_markdown("- item\n- item2");
        assert!(result.contains("<ul>"));
        assert!(result.contains("<li>item</li>"));
    }

    #[test]
    fn test_render_markdown_empty() {
        let result = render_markdown("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_render_markdown_paragraph() {
        let result = render_markdown("Hello world");
        assert!(result.contains("<p>Hello world</p>"));
    }

    #[test]
    fn test_render_markdown_raw_html() {
        let result = render_markdown("<ins>inserted</ins>");
        assert!(result.contains("<ins>inserted</ins>"));
    }

    // ── format_bytes Tests ──

    #[test]
    fn test_format_bytes_zero() {
        assert_eq!(format_bytes(0), "0 Bytes");
    }

    #[test]
    fn test_format_bytes_bytes() {
        assert_eq!(format_bytes(500), "500.0 Bytes");
    }

    #[test]
    fn test_format_bytes_kb() {
        assert_eq!(format_bytes(2048), "2.0 KB");
    }

    #[test]
    fn test_format_bytes_mb() {
        assert_eq!(format_bytes(1048576), "1.0 MB");
    }

    #[test]
    fn test_format_bytes_gb() {
        assert_eq!(format_bytes(1073741824), "1.0 GB");
    }

    #[test]
    fn test_format_bytes_rounding() {
        assert_eq!(format_bytes(1536), "1.5 KB");
    }

    #[test]
    fn test_format_bytes_one_byte() {
        assert_eq!(format_bytes(1), "1.0 Bytes");
    }

    #[test]
    fn test_format_bytes_negative() {
        let result = format_bytes(-100);
        assert!(result.contains("-"));
    }

    // ── strip_html Tests ──

    #[test]
    fn test_strip_html_simple() {
        assert_eq!(strip_html("<p>hello</p>"), "\nhello\n");
    }

    #[test]
    fn test_strip_html_no_tags() {
        assert_eq!(strip_html("hello world"), "hello world");
    }

    #[test]
    fn test_strip_html_nested() {
        assert_eq!(strip_html("<div><p>text</p></div>"), "\ntext\n\n");
    }

    #[test]
    fn test_strip_html_br() {
        let result = strip_html("line1<br>line2");
        assert_eq!(result, "line1\nline2");
    }

    #[test]
    fn test_strip_html_self_closing() {
        let result = strip_html("<p>a</p><br/><p>b</p>");
        assert_eq!(result, "\na\n\n\nb\n");
    }

    #[test]
    fn test_strip_html_empty() {
        assert_eq!(strip_html(""), "");
    }

    #[test]
    fn test_strip_html_only_tags() {
        assert_eq!(strip_html("<p></p>"), "\n\n");
    }

    // ── extract_title Tests ──

    #[test]
    fn test_extract_title_plain_text() {
        let (title, content) = extract_title("Hello world\nsecond line");
        assert_eq!(title, "Hello world");
        assert_eq!(content, "Hello world\nsecond line");
    }

    #[test]
    fn test_extract_title_markdown_heading() {
        let (title, _content) = extract_title("## Hello world\ncontent");
        assert_eq!(title, "Hello world");
    }

    #[test]
    fn test_extract_title_with_bold() {
        let (title, _) = extract_title("**Bold Title**");
        assert_eq!(title, "Bold Title**");
    }

    #[test]
    fn test_extract_title_empty_content() {
        let (title, content) = extract_title("");
        assert_eq!(title, "Note");
        assert_eq!(content, "");
    }

    #[test]
    fn test_extract_title_only_newlines() {
        let (title, _) = extract_title("\n\n\n");
        assert_eq!(title, "Note");
    }

    #[test]
    fn test_extract_title_whitespace() {
        let (title, _) = extract_title("   ");
        assert_eq!(title, "Note");
    }

    #[test]
    fn test_extract_title_html_content() {
        let (title, _) = extract_title("<p>Hello world</p>");
        assert_eq!(title, "Hello world");
    }

    #[test]
    fn test_extract_title_special_chars() {
        let (title, _) = extract_title("___Title___");
        assert_eq!(title, "Title___");
    }

    #[test]
    fn test_extract_title_returns_trimmed_content() {
        let (_, content) = extract_title("  hello  ");
        assert_eq!(content, "hello");
    }

    #[test]
    fn test_extract_title_multiline_first_line() {
        let (title, _) = extract_title("First line\nSecond line");
        assert_eq!(title, "First line");
    }

    // ── extract_tags Tests ──

    #[test]
    fn test_extract_tags_simple() {
        let tags = extract_tags("hello #world this is #rust");
        assert_eq!(tags, vec!["rust".to_string(), "world".to_string()]);
    }

    #[test]
    fn test_extract_tags_first_in_html() {
        let tags = extract_tags("<p>#abc #xyz</p>");
        assert_eq!(tags, vec!["abc".to_string(), "xyz".to_string()]);
        
        let tags2 = extract_tags("<div>&nbsp;#abc #xyz</div>");
        assert_eq!(tags2, vec!["abc".to_string(), "xyz".to_string()]);
    }

    #[test]
    fn test_extract_tags_nested_unclosed_html() {
        let tags = extract_tags("Detect if any error using\n${{<%[%'\"}}%\\\n#portswigger #ssti");
        assert_eq!(tags, vec!["portswigger".to_string(), "ssti".to_string()]);
        
        let tags2 = extract_tags("<p>Detect<br/>${{<%[%'\"}}%\\<br/>#portswigger #ssti</p>");
        assert_eq!(tags2, vec!["portswigger".to_string(), "ssti".to_string()]);
    }

    #[test]
    fn test_extract_tags_no_tags() {
        let tags: Vec<String> = extract_tags("hello world");
        assert!(tags.is_empty());
    }

    #[test]
    fn test_extract_tags_duplicate_dedup() {
        let tags = extract_tags("#rust #rust #RUST");
        assert_eq!(tags, vec!["rust".to_string()]);
    }

    #[test]
    fn test_extract_tags_with_underscore() {
        let tags = extract_tags("#my_tag");
        assert_eq!(tags, vec!["my_tag".to_string()]);
    }

    #[test]
    fn test_extract_tags_with_hyphen() {
        let tags = extract_tags("#my-tag");
        assert_eq!(tags, vec!["my-tag".to_string()]);
    }

    #[test]
    fn test_extract_tags_numbers() {
        let tags = extract_tags("#tag123");
        assert_eq!(tags, vec!["tag123".to_string()]);
    }

    #[test]
    fn test_extract_tags_hash_no_name() {
        let tags: Vec<String> = extract_tags("#");
        assert!(tags.is_empty());
    }

    #[test]
    fn test_extract_tags_hash_mid_word() {
        let tags: Vec<String> = extract_tags("abc#def");
        assert!(tags.is_empty());
    }

    #[test]
    fn test_extract_tags_parenthesized() {
        let tags = extract_tags("(#tag)");
        assert_eq!(tags, vec!["tag".to_string()]);
    }

    #[test]
    fn test_extract_tags_empty_content() {
        let tags: Vec<String> = extract_tags("");
        assert!(tags.is_empty());
    }

    #[test]
    fn test_extract_tags_sorted() {
        let tags = extract_tags("#zebra #apple #banana");
        assert_eq!(tags, vec!["apple", "banana", "zebra"]);
    }

    #[test]
    fn test_extract_tags_escaped_and_trailing_underscore() {
        // Turndown will escape a hashtag at the start of a block like \n\#pentest
        // We should extract pentest correctly, and strip trailing underscores like methodology_
        let tags = extract_tags("\\#pentest #bugbounty #methodology_");
        assert_eq!(tags, vec!["bugbounty", "methodology", "pentest"]);
    }

    // ── relative_time Tests ──

    #[test]
    fn test_relative_time_seconds() {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let result = relative_time(&now);
        assert_eq!(result, "0s");
    }

    #[test]
    fn test_relative_time_minutes() {
        let past = (chrono::Utc::now() - chrono::Duration::minutes(5)).format("%Y-%m-%d %H:%M:%S").to_string();
        let result = relative_time(&past);
        assert_eq!(result, "5m");
    }

    #[test]
    fn test_relative_time_hours() {
        let past = (chrono::Utc::now() - chrono::Duration::hours(3)).format("%Y-%m-%d %H:%M:%S").to_string();
        let result = relative_time(&past);
        assert_eq!(result, "3h");
    }

    #[test]
    fn test_relative_time_yesterday() {
        let past = (chrono::Utc::now() - chrono::Duration::hours(25)).format("%Y-%m-%d %H:%M:%S").to_string();
        let result = relative_time(&past);
        assert_eq!(result, "yesterday");
    }

    #[test]
    fn test_relative_time_days() {
        let past = (chrono::Utc::now() - chrono::Duration::days(5)).format("%Y-%m-%d %H:%M:%S").to_string();
        assert_eq!(relative_time(&past), "5d");
    }

    #[test]
    fn test_relative_time_future() {
        let future = (chrono::Utc::now() + chrono::Duration::hours(1)).format("%Y-%m-%d %H:%M:%S").to_string();
        let result = relative_time(&future);
        assert_eq!(result, "0s");
    }

    #[test]
    fn test_relative_time_invalid_format() {
        let result = relative_time("not-a-date");
        assert_eq!(result, "not-a-date");
    }

    #[test]
    fn test_relative_time_empty() {
        assert_eq!(relative_time(""), "");
    }

    // ── get_date_label Tests ──

    #[test]
    fn test_get_date_label_today() {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        assert_eq!(get_date_label(&today), "Today");
    }

    #[test]
    fn test_get_date_label_yesterday() {
        let yesterday = (chrono::Utc::now() - chrono::Duration::days(1)).format("%Y-%m-%d").to_string();
        assert_eq!(get_date_label(&yesterday), "Yesterday");
    }

    #[test]
    fn test_get_date_label_other_date() {
        let result = get_date_label("2024-01-15");
        assert_eq!(result, "Jan 15, 2024");
    }

    #[test]
    fn test_get_date_label_invalid() {
        let result = get_date_label("not-a-date");
        assert_eq!(result, "not-a-date");
    }

    #[test]
    fn test_get_date_label_empty() {
        assert_eq!(get_date_label(""), "");
    }

    // ── avatar_char Tests ──

    #[test]
    fn test_avatar_char_lowercase() {
        assert_eq!(avatar_char("alice"), "A");
    }

    #[test]
    fn test_avatar_char_uppercase() {
        assert_eq!(avatar_char("Bob"), "B");
    }

    #[test]
    fn test_avatar_char_empty() {
        assert_eq!(avatar_char(""), "?");
    }

    #[test]
    fn test_avatar_char_numbers() {
        assert_eq!(avatar_char("123"), "1");
    }

    #[test]
    fn test_avatar_char_unicode() {
        assert_eq!(avatar_char("ñoño").chars().count(), 1);
    }

    // ── get_month_name Tests ──

    #[test]
    fn test_get_month_name_all() {
        assert_eq!(get_month_name(1), "January");
        assert_eq!(get_month_name(2), "February");
        assert_eq!(get_month_name(3), "March");
        assert_eq!(get_month_name(4), "April");
        assert_eq!(get_month_name(5), "May");
        assert_eq!(get_month_name(6), "June");
        assert_eq!(get_month_name(7), "July");
        assert_eq!(get_month_name(8), "August");
        assert_eq!(get_month_name(9), "September");
        assert_eq!(get_month_name(10), "October");
        assert_eq!(get_month_name(11), "November");
        assert_eq!(get_month_name(12), "December");
    }

    #[test]
    fn test_get_month_name_invalid() {
        assert_eq!(get_month_name(0), "Unknown");
        assert_eq!(get_month_name(13), "Unknown");
    }

    // ── extract_first_image_id Tests ──

    #[test]
    fn test_extract_first_image_id_found() {
        let content = "text ![alt](/resources/42) more text";
        assert_eq!(extract_first_image_id(content), Some(42));
    }

    #[test]
    fn test_extract_first_image_id_no_image() {
        let content = "no image here";
        assert_eq!(extract_first_image_id(content), None);
    }

    #[test]
    fn test_extract_first_image_id_multiple() {
        let content = "![a](/resources/1) ![b](/resources/2)";
        assert_eq!(extract_first_image_id(content), Some(1));
    }

    #[test]
    fn test_extract_first_image_id_non_numeric() {
        let content = "![a](/resources/abc)";
        assert_eq!(extract_first_image_id(content), None);
    }

    #[test]
    fn test_extract_first_image_id_no_closing_paren() {
        let content = "![a](/resources/42";
        assert_eq!(extract_first_image_id(content), None);
    }

    #[test]
    fn test_extract_first_image_id_empty() {
        assert_eq!(extract_first_image_id(""), None);
    }

    // ── uuid_v4 Tests ──

    #[test]
    fn test_uuid_v4_format() {
        let id = uuid_v4();
        assert!(!id.is_empty());
        assert_eq!(id.len(), 32);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_uuid_v4_unique() {
        let a = uuid_v4();
        let b = uuid_v4();
        assert_ne!(a, b);
    }

    #[test]
    fn test_uuid_v4_hex_only() {
        let id = uuid_v4();
        assert!(id.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')));
    }

    // ── get_client_ip Tests ──

    #[test]
    fn test_get_client_ip_x_forwarded_for() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "203.0.113.1, 10.0.0.1".parse().unwrap());
        let addr = "127.0.0.1:3000".parse::<SocketAddr>().unwrap();
        let ip = get_client_ip(&headers, &ConnectInfo(addr));
        assert_eq!(ip, "203.0.113.1");
    }

    #[test]
    fn test_get_client_ip_x_real_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", "203.0.113.2".parse().unwrap());
        let addr = "127.0.0.1:3000".parse::<SocketAddr>().unwrap();
        let ip = get_client_ip(&headers, &ConnectInfo(addr));
        assert_eq!(ip, "203.0.113.2");
    }

    #[test]
    fn test_get_client_ip_fallback_to_socket() {
        let headers = HeaderMap::new();
        let addr = "10.0.0.5:5000".parse::<SocketAddr>().unwrap();
        let ip = get_client_ip(&headers, &ConnectInfo(addr));
        assert_eq!(ip, "10.0.0.5");
    }

    #[test]
    fn test_get_client_ip_xff_priority() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "1.2.3.4".parse().unwrap());
        headers.insert("x-real-ip", "5.6.7.8".parse().unwrap());
        let addr = "127.0.0.1:3000".parse::<SocketAddr>().unwrap();
        let ip = get_client_ip(&headers, &ConnectInfo(addr));
        assert_eq!(ip, "1.2.3.4");
    }

    #[test]
    fn test_get_client_ip_xff_empty_skips() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "".parse().unwrap());
        headers.insert("x-real-ip", "10.0.0.1".parse().unwrap());
        let addr = "127.0.0.1:3000".parse::<SocketAddr>().unwrap();
        let ip = get_client_ip(&headers, &ConnectInfo(addr));
        assert_eq!(ip, "10.0.0.1");
    }

    // ── generate_calendar Tests ──

    #[test]
    fn test_generate_calendar_structure() {
        let weeks = generate_calendar(2024, 1, &[], None);
        assert!(!weeks.is_empty());
        let week = weeks[0].as_array().unwrap();
        assert_eq!(week.len(), 7);
    }

    #[test]
    fn test_generate_calendar_has_memos() {
        let weeks = generate_calendar(2024, 1, &["2024-01-15".to_string()], None);
        let has_marked = weeks.iter().any(|w| {
            w.as_array().unwrap().iter().any(|d| {
                d["has_memos"].as_bool().unwrap_or(false)
            })
        });
        assert!(has_marked);
    }

    #[test]
    fn test_generate_calendar_no_memos() {
        let weeks = generate_calendar(2024, 1, &[], None);
        let has_any_memos = weeks.iter().any(|w| {
            w.as_array().unwrap().iter().any(|d| d["has_memos"].as_bool().unwrap_or(false))
        });
        assert!(!has_any_memos);
    }

    #[test]
    fn test_generate_calendar_december() {
        let weeks = generate_calendar(2024, 12, &[], None);
        assert!(!weeks.is_empty());
    }

    #[test]
    fn test_generate_calendar_padding_days() {
        let weeks = generate_calendar(2024, 1, &[], None);
        for w in &weeks {
            let arr = w.as_array().unwrap();
            assert_eq!(arr.len(), 7);
        }
    }

    #[test]
    fn test_generate_calendar_selected_date() {
        let weeks = generate_calendar(2024, 1, &[], Some("2024-01-15"));
        let has_selected = weeks.iter().any(|w| {
            w.as_array().unwrap().iter().any(|d| d["is_selected"].as_bool().unwrap_or(false))
        });
        assert!(has_selected);
    }

    // ── process_memo_content Tests (basic without DB) ──

    #[test]
    fn test_process_memo_content_no_refs() {
        let db = db::Database::new(":memory:").unwrap();
        let (content, resources) = process_memo_content(&db, "Hello world");
        assert_eq!(content, "Hello world");
        assert!(resources.is_empty());
    }

    #[test]
    fn test_process_memo_content_empty() {
        let db = db::Database::new(":memory:").unwrap();
        let (content, resources) = process_memo_content(&db, "");
        assert_eq!(content, "");
        assert!(resources.is_empty());
    }

    #[test]
    fn test_process_memo_content_ref_not_found() {
        let db = db::Database::new(":memory:").unwrap();
        let (content, resources) = process_memo_content(&db, "see [file](/resources/999)");
        assert_eq!(content, "see");
        assert!(resources.is_empty());
    }

    #[test]
    fn test_render_markdown_nbsp_blank_lines() {
        // Single blank line between paragraphs (one &nbsp; paragraph)
        let md = "Line 1\n\n&nbsp;\n\nLine 2";
        let html = render_markdown(md);
        assert!(html.contains("Line 1"), "Should contain Line 1");
        assert!(html.contains("Line 2"), "Should contain Line 2");
        // Should have 3 <p> tags: Line 1, &nbsp;, Line 2
        let p_count = html.matches("<p>").count();
        assert_eq!(p_count, 3, "Expected 3 paragraphs, got {}: {}", p_count, html);
    }

    #[test]
    fn test_render_markdown_multiple_nbsp_blank_lines() {
        // Two blank lines between paragraphs
        let md = "Line 1\n\n&nbsp;\n\n&nbsp;\n\nLine 2";
        let html = render_markdown(md);
        let p_count = html.matches("<p>").count();
        assert_eq!(p_count, 4, "Expected 4 paragraphs, got {}: {}", p_count, html);
    }

    #[test]
    fn test_render_markdown_complex_blank_lines() {
        // Line 1, one blank, Line 2, two blanks, Line 3
        let md = "Line 1\n\n&nbsp;\n\nLine 2\n\n&nbsp;\n\n&nbsp;\n\nLine 3";
        let html = render_markdown(md);
        let p_count = html.matches("<p>").count();
        assert_eq!(p_count, 6, "Expected 6 paragraphs (3 text + 3 nbsp), got {}: {}", p_count, html);
        assert!(html.contains("Line 1"));
        assert!(html.contains("Line 2"));
        assert!(html.contains("Line 3"));
    }

    #[test]
    fn test_render_markdown_hard_breaks() {
        let md = "Line 1<br><br><br>Line 2";
        let html = render_markdown(md);
        println!("HTML for raw br: {}", html);
    }
}
