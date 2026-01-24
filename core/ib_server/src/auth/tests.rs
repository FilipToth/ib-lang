use axum::http::{HeaderMap, HeaderValue};
use reqwest::header::AUTHORIZATION;

use super::*;

/// A header map holding one `Authorization` value, built from raw bytes so a
/// test can send what a well behaved client never would.
fn authorization(value: &[u8]) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_bytes(value).expect("the test value must be a legal header"),
    );

    headers
}

#[test]
fn reads_the_token_after_bearer() {
    let headers = authorization(b"Bearer abc.def.ghi");
    assert_eq!(bearer_token(&headers), Some("abc.def.ghi"));
}

/// The one that used to abort the request: HTTP allows these bytes, `to_str`
/// refuses them, and the unwrap behind it dropped the connection.
#[test]
fn refuses_an_authorization_header_that_is_not_text() {
    let headers = authorization(b"Bearer \xc3\xa9");
    assert_eq!(bearer_token(&headers), None);
}

#[test]
fn refuses_a_scheme_that_is_not_bearer() {
    assert_eq!(bearer_token(&authorization(b"Basic abc")), None);
    assert_eq!(bearer_token(&authorization(b"Bearer")), None);
    assert_eq!(bearer_token(&authorization(b"Bearer ")), None);

    // the scheme is matched as written, which is what the one client sends
    assert_eq!(bearer_token(&authorization(b"bearer abc")), None);

    assert_eq!(bearer_token(&HeaderMap::new()), None);
}

#[test]
fn builds_a_header_for_a_normal_token() {
    assert!(bearer_header("abc.def.ghi").is_some());
}

#[test]
fn refuses_a_token_a_header_cannot_carry() {
    assert_eq!(bearer_header("a\nb"), None);
    assert_eq!(bearer_header("a\u{7f}b"), None);
}

/// A header value may hold bytes over 0x7f even though `to_str` will not read
/// them back. That asymmetry is the whole reason the reading side has to be
/// careful, so it is worth stating rather than assuming it cuts both ways.
#[test]
fn carries_a_token_with_bytes_a_string_cannot_read_back() {
    assert!(bearer_header("é").is_some());
}

#[test]
fn accepts_a_firebase_uid() {
    assert_eq!(invalid_uid("kJ3mPq7XyZ2aB9cD4eF6gH8iJ0kL"), None);
    assert_eq!(invalid_uid("a-b_c123"), None);
}

/// uids become folder names under `data`, so nothing may lead out of it.
#[test]
fn refuses_a_uid_that_could_escape_the_data_folder() {
    assert!(invalid_uid(".").is_some());
    assert!(invalid_uid("..").is_some());
    assert!(invalid_uid("../other").is_some());
    assert!(invalid_uid("a/b").is_some());
    assert!(invalid_uid("a\\b").is_some());
    assert!(invalid_uid("/etc").is_some());
}

#[test]
fn refuses_a_uid_with_a_character_a_folder_name_should_not_have() {
    assert!(invalid_uid("a b").is_some());
    assert!(invalid_uid("a\nb").is_some());
    assert!(invalid_uid("a\0b").is_some());
    assert!(invalid_uid("a:b").is_some());
}

#[test]
fn refuses_an_empty_uid() {
    assert!(invalid_uid("").is_some());
}

#[test]
fn holds_the_uid_to_the_length_firebase_allows() {
    assert_eq!(invalid_uid(&"a".repeat(MAX_UID_LEN)), None);
    assert!(invalid_uid(&"a".repeat(MAX_UID_LEN + 1)).is_some());
}
