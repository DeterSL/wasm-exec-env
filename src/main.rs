mod core;
mod config;

use core::{*, worker::FuncJob};
use std::{net::SocketAddr, convert::Infallible};

use bytes::Bytes;
use hyper::server::conn::http1;
use hyper::{body::Incoming, Method, Request, Response, StatusCode};
use hyper::service::service_fn;
use hyper_util::{rt::TokioIo};
use tokio::sync::{mpsc, oneshot};
use http_body_util::BodyExt;

use wasmtime::{Config, CacheConfig, Cache};

use crate::core::worker::Worker;

type RespBody = http_body_util::combinators::BoxBody<Bytes, Infallible>;

async fn handle_request(
    req: Request<Incoming>,
    tx: mpsc::Sender<FuncJob>,
) -> Result<Response<RespBody>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/run") => {
            let collected = match req.into_body().collect().await {
                Ok(c) => c,
                Err(e) => return Ok(resp(StatusCode::BAD_REQUEST, format!("read body error: {e}"))),
            };
            let body_bytes = collected.to_bytes();

            let cfg: config::func_config::FuncBinaryConfig = match serde_json::from_slice(&body_bytes) {
                Ok(c) => c,
                Err(e) => {
                    return Ok(resp(
                        StatusCode::BAD_REQUEST,
                        format!("invalid JSON for FuncBinaryConfig: {e}"),
                    ))
                }
            };

            let (reply_tx, reply_rx) = oneshot::channel();

            if let Err(_e) = tx.send(FuncJob { config: cfg, reply: reply_tx }).await {
                return Ok(resp(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "worker unavailable".to_string(),
                ));
            }

            match reply_rx.await {
                Ok(Ok(out)) => {
                    match serde_json::to_vec(&out) {
                        Ok(json) => Ok(json_resp(StatusCode::OK, json)),
                        Err(e) => Ok(resp(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("failed to serialize output: {e}"),
                        )),
                    }
                }
                Ok(Err(err)) => Ok(resp(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
                Err(_canceled) => Ok(resp(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "worker dropped".to_string(),
                )),
            }
        }
        _ => Ok(resp(StatusCode::NOT_FOUND, "not found".to_string())),
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let (tx, rx) = mpsc::channel::<FuncJob>(1024);

    // 2) Spawn the synchronous worker on its own OS thread and run forever
    std::thread::spawn(move || {
        let mut cache_config = Config::new();
        let cache = Cache::new(CacheConfig::new()).unwrap();
        cache_config.cache(Some(cache));
        let mut worker = Worker::new(cache_config);
        worker.run_forever(rx);
    });

    // 3) Minimal Hyper 1.x server: accept loop + per-conn http1 server
    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Listening on http://{addr}");

    loop {
        let (stream, _peer) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let tx_conn = tx.clone();

        tokio::spawn(async move {
            let svc = service_fn(move |req| {
                let tx_req = tx_conn.clone();
                handle_request(req, tx_req)
            });

            if let Err(err) = http1::Builder::new().serve_connection(io, svc).await {
                eprintln!("connection error: {err}");
            }
        });
    }
}

fn resp(status: StatusCode, body: String) -> Response<RespBody> {
    use http_body_util::{BodyExt, Full};
    let full = Full::new(Bytes::from(body));
    Response::builder()
        .status(status)
        .header("content-type", "text/plain; charset=utf-8")
        .body(full.boxed())
        .unwrap()
}

fn json_resp(status: StatusCode, body: Vec<u8>) -> Response<RespBody> {
    use http_body_util::{BodyExt, Full};
    let full = Full::new(Bytes::from(body));
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(full.boxed())
        .unwrap()
}
