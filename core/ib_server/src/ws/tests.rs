use super::*;

#[test]
fn accepts_the_marker_and_a_token() {
    assert_eq!(offered_token("ib-auth-v1,abc.def.ghi"), Some("abc.def.ghi"));
    assert_eq!(
        offered_token("ib-auth-v1, abc.def.ghi"),
        Some("abc.def.ghi")
    );
}

/// axum compares `req_protocol.trim()` when it decides whether to echo the
/// marker. Reading the token any other way could accept a handshake axum then
/// refuses to complete.
#[test]
fn trims_like_axum_does() {
    assert_eq!(
        offered_token("  ib-auth-v1 ,  abc.def.ghi  "),
        Some("abc.def.ghi")
    );
}

#[test]
fn refuses_a_token_without_the_marker() {
    assert_eq!(offered_token("abc.def.ghi"), None);
    assert_eq!(offered_token("chat, abc.def.ghi"), None);
    assert_eq!(offered_token("ib-auth-v2, abc.def.ghi"), None);
}

/// An empty token must never reach the auth-server as if it were one.
#[test]
fn refuses_the_marker_alone() {
    assert_eq!(offered_token("ib-auth-v1"), None);
    assert_eq!(offered_token("ib-auth-v1,"), None);
    assert_eq!(offered_token("ib-auth-v1,   "), None);
}

#[test]
fn refuses_more_than_a_marker_and_a_token() {
    assert_eq!(offered_token("ib-auth-v1, abc, def"), None);
}

#[test]
fn refuses_an_empty_header() {
    assert_eq!(offered_token(""), None);
    assert_eq!(offered_token(","), None);
}
