use http::{header, StatusCode};

/*
#[tokio::test]
async fn create_tournament_requires_login() {
    let request = create_tournament_request();
    let resp = send_request(request).await;

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn create_tournament() {
    let request = register_account_request();
    let resp = send_request(request).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let account: CreateAccountResponse = extract_json_body(resp).await;

    let request = login_request(account.0.id);
    let resp = send_request(request).await;

    assert_eq!(resp.status(), StatusCode::OK);
    let cookies = get_cookies(&resp);

    let mut request = create_tournament_request();
    request
        .headers_mut()
        .insert(header::COOKIE, cookies[0].clone());
    let resp = send_request(request).await;

    assert_eq!(resp.status(), StatusCode::OK);
    let tourn: CreateTournamentResponse = extract_json_body(resp).await;
}
*/
