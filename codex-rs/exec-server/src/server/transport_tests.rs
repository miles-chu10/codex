use std::net::SocketAddr;

use pretty_assertions::assert_eq;

use super::DEFAULT_LISTEN_URL;
use super::parse_listen_url;
use super::reject_origin_header;

#[test]
fn parse_listen_url_accepts_default_websocket_url() {
    let bind_address =
        parse_listen_url(DEFAULT_LISTEN_URL).expect("default listen URL should parse");
    assert_eq!(
        bind_address,
        "127.0.0.1:0"
            .parse::<SocketAddr>()
            .expect("valid socket address")
    );
}

#[test]
fn parse_listen_url_accepts_websocket_url() {
    let bind_address =
        parse_listen_url("ws://127.0.0.1:1234").expect("websocket listen URL should parse");
    assert_eq!(
        bind_address,
        "127.0.0.1:1234"
            .parse::<SocketAddr>()
            .expect("valid socket address")
    );
}

#[test]
fn parse_listen_url_rejects_non_loopback_websocket_url() {
    let err = parse_listen_url("ws://0.0.0.0:1234")
        .expect_err("non-loopback bind address should be rejected");
    assert_eq!(
        err.to_string(),
        "unsafe websocket --listen URL `ws://0.0.0.0:1234`; exec-server only supports loopback addresses"
    );
}

#[test]
fn parse_listen_url_rejects_invalid_websocket_url() {
    let err = parse_listen_url("ws://localhost:1234")
        .expect_err("hostname bind address should be rejected");
    assert_eq!(
        err.to_string(),
        "invalid websocket --listen URL `ws://localhost:1234`; expected `ws://IP:PORT`"
    );
}

#[test]
fn parse_listen_url_rejects_unsupported_url() {
    let err =
        parse_listen_url("http://127.0.0.1:1234").expect_err("unsupported scheme should fail");
    assert_eq!(
        err.to_string(),
        "unsupported --listen URL `http://127.0.0.1:1234`; expected `ws://IP:PORT`"
    );
}

#[test]
fn reject_origin_header_rejects_browser_origin() {
    let request = tokio_tungstenite::tungstenite::handshake::server::Request::builder()
        .header("Origin", "http://evil.example")
        .body(())
        .expect("valid request");
    let response = tokio_tungstenite::tungstenite::handshake::server::Response::builder()
        .status(101)
        .body(())
        .expect("valid response");

    let err = reject_origin_header(&request, response).expect_err("origin should be rejected");

    assert_eq!(err.status(), 403);
}

#[test]
fn reject_origin_header_allows_non_browser_request() {
    let request = tokio_tungstenite::tungstenite::handshake::server::Request::builder()
        .body(())
        .expect("valid request");
    let response = tokio_tungstenite::tungstenite::handshake::server::Response::builder()
        .status(101)
        .body(())
        .expect("valid response");

    let response = reject_origin_header(&request, response).expect("request should be allowed");

    assert_eq!(response.status(), 101);
}
