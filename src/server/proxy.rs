use std::sync::Arc;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::response::Response;
use base64::Engine;
use hyper::Uri;
use hyper::header;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioIo;
use tokio::io::copy_bidirectional;
use tokio::net::TcpStream;
use crate::server::server_bootstrap::ServerState;

pub type ProxyClient = Client<HttpConnector, Body>;

pub async fn proxy_handler(
    State(state): State<Arc<ServerState>>,
    mut req: Request,
) -> Response {
    let proxy_port = state.dev_proxy_port.unwrap();

    let path = req.uri().path_and_query().map(|p| p.as_str()).unwrap_or("/");
    let new_uri = format!("http://127.0.0.1:{}{}", proxy_port, path)
        .parse::<Uri>()
        .unwrap();

    *req.uri_mut() = new_uri;
    if req.headers()
        .get(header::UPGRADE)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_lowercase().contains("websocket"))
        .unwrap_or(false)
    {
        let addr = format!("127.0.0.1:{}", proxy_port);
        let path = req.uri().path_and_query()
            .map(|p| p.as_str())
            .unwrap_or("/")
            .to_string();
        let req_headers = req.headers().clone();
        let on_upgrade = hyper::upgrade::on(&mut req);

        tokio::spawn(async move {
            let mut server = match TcpStream::connect(&addr).await {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!("WS proxy: failed to connect: {}", e);
                    return;
                }
            };

            // Write the handshake to Next.js
            use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};

            let mut handshake = format!("GET {} HTTP/1.1\r\n", path);
            for (key, value) in &req_headers {
                if let Ok(v) = value.to_str() {
                    handshake.push_str(&format!("{}: {}\r\n", key, v));
                }
            }
            handshake.push_str("\r\n");

            if server.write_all(handshake.as_bytes()).await.is_err() {
                return;
            }
            
            let mut reader = BufReader::new(&mut server);
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(_) if line == "\r\n" => break, // end of headers
                    Ok(0) => return,                  // connection closed
                    Err(_) => return,
                    _ => {}
                }
            }

            // Now pipe the raw frames bidirectionally
            match on_upgrade.await {
                Ok(client) => {
                    let mut client_io = TokioIo::new(client);
                    let server = reader.into_inner();
                    if let Err(e) = copy_bidirectional(&mut client_io, server).await {
                        tracing::debug!("WS pipe closed: {}", e);
                    }
                }
                Err(e) => tracing::warn!("WS upgrade error: {}", e),
            }
        });

        return Response::builder()
            .status(101)
            .header(header::UPGRADE, "websocket")
            .header(header::CONNECTION, "upgrade")
            .header(
                "Sec-WebSocket-Accept",
                derive_accept_key(
                    req.headers()
                        .get("Sec-WebSocket-Key")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("")
                ),
            )
            .body(Body::empty())
            .unwrap();
    }

    // Regular HTTP
    state.proxy_client
        .clone()
        .unwrap()
        .request(req)
        .await
        .map(|r| r.map(Body::new))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(502)
                .body(Body::from("Dev server unavailable - is it running?"))
                .unwrap()
        })
}

fn derive_accept_key(key: &str) -> String {
    use sha1::{Digest, Sha1};
    let mut hasher = Sha1::new();
    hasher.update(key.as_bytes());
    hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    base64::engine::general_purpose::STANDARD.encode(hasher.finalize())
}