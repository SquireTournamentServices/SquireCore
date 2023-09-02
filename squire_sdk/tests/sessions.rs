// Tests that pertain to the different session types

use http::{HeaderValue, HeaderMap};
use squire_sdk::api::SessionToken;

#[test]
fn token_parsing() {
    let token = SessionToken([0; 32]);
    let (name, header) = token.as_header();
    let expected = "00".repeat(32);
    assert_eq!(header, HeaderValue::from_str(&expected).unwrap());
    let mut headers = HeaderMap::new();
    headers.insert(name, header);
    assert_eq!(token, SessionToken::try_from(&headers).unwrap());
}

