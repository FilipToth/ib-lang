use std::{collections::HashMap, env, time::Duration};

use axum::{
    extract::Request,
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use reqwest::header::AUTHORIZATION;

/// The longest uid that is accepted. Firebase's own cap, so nothing it can
/// issue is turned away by the length alone.
const MAX_UID_LEN: usize = 128;

/// The token from an `Authorization: Bearer ...` header, if there is one.
///
/// HTTP allows bytes in a header value that a rust string cannot hold, and
/// anybody can send them, so a header that is not text is simply not a bearer
/// token rather than something to panic over.
fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let header = headers.get(AUTHORIZATION)?.to_str().ok()?;
    let token = header.strip_prefix("Bearer ")?;

    if token.is_empty() {
        return None;
    }

    Some(token)
}

/// The `Authorization` header carrying `jwt`, or `None` when the token is not
/// something a header value can hold.
///
/// `HeaderValue` decides that rather than a list of our own, which would drift
/// from what the type actually accepts.
fn bearer_header(jwt: &str) -> Option<HeaderValue> {
    HeaderValue::from_str(&format!("Bearer {}", jwt)).ok()
}

/// Where the auth-server can be reached. `None`, with a line in the log, when
/// it is not configured: nothing can be verified without it, so every request
/// is then unauthenticated, which is the honest answer.
fn auth_backend_url() -> Option<String> {
    match env::var("AUTH_BACKEND_URL") {
        Ok(url) => Some(url),
        Err(_) => {
            eprintln!("AUTH_BACKEND_URL is not set, so nobody can be authenticated");
            None
        }
    }
}

/// Why a uid cannot be trusted, if it cannot.
///
/// It names a folder under `data`, so a uid that is not a plain name could
/// reach files that are not the caller's. The rule is an allowlist on purpose:
/// it leaves `.`, `..`, slashes, control characters and spaces unrepresentable
/// without having to think of each of them.
fn invalid_uid(uid: &str) -> Option<String> {
    if uid.is_empty() {
        return Some("the account has no identifier".to_string());
    }

    if uid.chars().count() > MAX_UID_LEN {
        return Some(format!(
            "the account identifier is longer than {} characters",
            MAX_UID_LEN
        ));
    }

    if !uid
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Some("the account identifier is not a plain name".to_string());
    }

    None
}

/// Verifies a Firebase ID token with the auth-server and returns the uid.
/// Returns `None` for any failure: unreachable backend, malformed response,
/// a token the backend rejected, or a uid this server will not use.
pub async fn verify_jwt(jwt: &str) -> Option<String> {
    // an auth-server that never answers would otherwise hold every request
    // behind it open indefinitely
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .ok()?;

    let url = auth_backend_url()?;
    let authorization = bearer_header(jwt)?;

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, authorization);

    let resp = match client.get(url).headers(headers).send().await {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("could not reach the auth-server: {:?}", e);
            return None;
        }
    };

    let resp = resp.json::<HashMap<String, String>>().await.ok()?;
    let uid = resp.get("uid")?;

    // the uid comes from another service and becomes a path here, so it is
    // checked once, at the only place one enters this process
    if let Some(reason) = invalid_uid(uid) {
        eprintln!("the auth-server returned a uid that cannot be used: {}", reason);
        return None;
    }

    Some(uid.clone())
}

pub async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let Some(jwt) = bearer_token(req.headers()) else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    // the borrow of the request ends here, so its extensions can be written
    let jwt = jwt.to_string();

    match verify_jwt(&jwt).await {
        Some(uid) => {
            req.extensions_mut().insert(uid);
            Ok(next.run(req).await)
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

#[cfg(test)]
mod tests;
