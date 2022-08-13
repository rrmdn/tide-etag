#[async_std::test]
async fn no_etag() {
    let mut app = tide::new();
    app.with(tide_etag::EtagMiddleware::default());
    app.at("/").get(|_| async {
        let mut resp = tide::Response::new(200);
        resp.set_body(tide::Body::from_string("HELLO".to_string()));
        Ok(resp)
    });
    app.at("/has_etag").get(|_| async {
        let mut resp = tide::Response::new(200);
        resp.insert_header(tide::http::headers::ETAG, "I_AM_BATMAN");
        resp.set_body(tide::Body::from_string("HELLO".to_string()));
        Ok(resp)
    });

    let req = tide::http::Request::new(
        tide::http::Method::Get,
        tide::http::Url::parse("http://_/").unwrap(),
    );

    let ok_resp: tide::http::Response = app.respond(req).await.unwrap();

    assert_eq!(ok_resp.status(), 200);
    assert_ne!(ok_resp.len().unwrap(), 0);

    let fresh_resp_etag = ok_resp
        .header(tide::http::headers::ETAG)
        .unwrap()
        .last()
        .as_str();
    println!("etag {}", &fresh_resp_etag);

    let mut second_req = tide::http::Request::new(
        tide::http::Method::Get,
        tide::http::Url::parse("http://_/").unwrap(),
    );
    second_req.append_header(&tide::http::headers::IF_NONE_MATCH, fresh_resp_etag);

    let not_modified_resp: tide::http::Response = app.respond(second_req).await.unwrap();
    assert_eq!(not_modified_resp.status(), 304);
    assert_eq!(not_modified_resp.len().unwrap(), 0);

    let respect_etag_req = tide::http::Request::new(
        tide::http::Method::Get,
        tide::http::Url::parse("http://_/has_etag").unwrap(),
    );

    let respect_etag_resp: tide::http::Response = app.respond(respect_etag_req).await.unwrap();
    assert_eq!(
        respect_etag_resp
            .header(tide::http::headers::ETAG)
            .unwrap()
            .last()
            .as_str(),
        "I_AM_BATMAN"
    );
}
