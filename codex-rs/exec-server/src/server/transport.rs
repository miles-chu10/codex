use std::io::Write as _;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_hdr_async;
use tokio_tungstenite::tungstenite::handshake::server::ErrorResponse;
use tokio_tungstenite::tungstenite::handshake::server::Request;
use tokio_tungstenite::tungstenite::handshake::server::Response;
use tokio_tungstenite::tungstenite::http::StatusCode;
use tracing::warn;

use crate::ExecServerRuntimePaths;
use crate::connection::JsonRpcConnection;
use crate::server::processor::ConnectionProcessor;

pub const DEFAULT_LISTEN_URL: &str = "ws://127.0.0.1:0";

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ExecServerListenUrlParseError {
    UnsupportedListenUrl(String),
    InvalidWebSocketListenUrl(String),
    NonLoopbackWebSocketListenUrl(String),
}

impl std::fmt::Display for ExecServerListenUrlParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecServerListenUrlParseError::UnsupportedListenUrl(listen_url) => write!(
                f,
                "unsupported --listen URL `{listen_url}`; expected `ws://IP:PORT`"
            ),
            ExecServerListenUrlParseError::InvalidWebSocketListenUrl(listen_url) => write!(
                f,
                "invalid websocket --listen URL `{listen_url}`; expected `ws://IP:PORT`"
            ),
            ExecServerListenUrlParseError::NonLoopbackWebSocketListenUrl(listen_url) => write!(
                f,
                "unsafe websocket --listen URL `{listen_url}`; exec-server only supports loopback addresses"
            ),
        }
    }
}

impl std::error::Error for ExecServerListenUrlParseError {}

pub(crate) fn parse_listen_url(
    listen_url: &str,
) -> Result<SocketAddr, ExecServerListenUrlParseError> {
    if let Some(socket_addr) = listen_url.strip_prefix("ws://") {
        let socket_addr = socket_addr.parse::<SocketAddr>().map_err(|_| {
            ExecServerListenUrlParseError::InvalidWebSocketListenUrl(listen_url.to_string())
        })?;
        if !socket_addr.ip().is_loopback() {
            return Err(
                ExecServerListenUrlParseError::NonLoopbackWebSocketListenUrl(
                    listen_url.to_string(),
                ),
            );
        }
        return Ok(socket_addr);
    }

    Err(ExecServerListenUrlParseError::UnsupportedListenUrl(
        listen_url.to_string(),
    ))
}

pub(crate) async fn run_transport(
    listen_url: &str,
    runtime_paths: ExecServerRuntimePaths,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bind_address = parse_listen_url(listen_url)?;
    run_websocket_listener(bind_address, runtime_paths).await
}

async fn run_websocket_listener(
    bind_address: SocketAddr,
    runtime_paths: ExecServerRuntimePaths,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind(bind_address).await?;
    let local_addr = listener.local_addr()?;
    let processor = ConnectionProcessor::new(runtime_paths);
    tracing::info!("codex-exec-server listening on ws://{local_addr}");
    println!("ws://{local_addr}");
    std::io::stdout().flush()?;

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let processor = processor.clone();
        tokio::spawn(async move {
            match accept_hdr_async(stream, reject_origin_header).await {
                Ok(websocket) => {
                    processor
                        .run_connection(JsonRpcConnection::from_websocket(
                            websocket,
                            format!("exec-server websocket {peer_addr}"),
                        ))
                        .await;
                }
                Err(err) => {
                    warn!(
                        "failed to accept exec-server websocket connection from {peer_addr}: {err}"
                    );
                }
            }
        });
    }
}

fn reject_origin_header(request: &Request, response: Response) -> Result<Response, ErrorResponse> {
    if request.headers().contains_key("origin") {
        let mut response = ErrorResponse::new(Some(
            "browser Origin websocket requests are not allowed".to_string(),
        ));
        *response.status_mut() = StatusCode::FORBIDDEN;
        return Err(response);
    }

    Ok(response)
}

#[cfg(test)]
#[path = "transport_tests.rs"]
mod transport_tests;
