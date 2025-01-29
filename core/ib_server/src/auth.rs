use std::{collections::HashMap, env};

use axum::{
    extract::Request,
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use reqwest::header::AUTHORIZATION;

async fn ping_auth_backend(jwt: &str) -> Option<String> {
    let client = reqwest::Client::new();
    let authorization = format!("Bearer {}", jwt);
    let url = env::var("AUTH_BACKEND_URL").unwrap();

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&authorization).unwrap(),
    );

    let resp = match client.get(url).headers(headers).send().await {
        Ok(resp) => resp,
        Err(e) => {
            println!("{:?}", e);
            return None;
        }
    };

    let resp = match resp.json::<HashMap<String, String>>().await {
        Ok(resp) => resp,
        Err(_) => return None,
    };

    let uid = match resp.get("uid") {
        Some(uid) => uid,
        None => return None,
    };

    Some(uid.clone())
}

pub async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let Some(auth_header) = req.headers().get("Authorization") else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let auth_header = auth_header.to_str().unwrap();

    let Some(jwt) = auth_header.strip_prefix("Bearer ") else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    match ping_auth_backend(jwt).await {
        Some(uid) => {
            req.extensions_mut().insert(uid);
            Ok(next.run(req).await)
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}
