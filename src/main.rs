mod core;
mod config;

use core::{worker::FuncJob};
use std::{convert::Infallible, net::SocketAddr};

use anyhow::anyhow;
use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{body::Incoming, Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::sync::{mpsc, oneshot};

use crate::core::engine::DeterSLEngine;
use crate::core::worker::Worker;

// NEW:
use crate::config::engine::engine_config::EngineConfig;
use crate::config::engine::global_engine_config::{
    init_global_engine_config,
    require_global_engine_config,
};

type RespBody = http_body_util::combinators::BoxBody<Bytes, Infallible>;

async fn handle_request(
    req: Request<Incoming>,
    tx_router: mpsc::Sender<FuncJob>,
) -> Result<Response<RespBody>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/run") => {
            let collected = match req.into_body().collect().await {
                Ok(c) => c,
                Err(e) => return Ok(resp(StatusCode::BAD_REQUEST, format!("read body error: {e}"))),
            };
            let body_bytes = collected.to_bytes();

            let cfg: config::FuncBinaryConfig = match serde_json::from_slice(&body_bytes) {
                Ok(c) => c,
                Err(e) => {
                    return Ok(resp(
                        StatusCode::BAD_REQUEST,
                        format!("invalid JSON for FuncBinaryConfig: {e}"),
                    ))
                }
            };

            let (reply_tx, reply_rx) = oneshot::channel();

            if let Err(_e) = tx_router.send(FuncJob { config: cfg, reply: reply_tx }).await {
                return Ok(resp(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "router unavailable".to_string(),
                ));
            }

            match reply_rx.await {
                Ok(Ok(out)) => match serde_json::to_vec(&out) {
                    Ok(json) => Ok(json_resp(StatusCode::OK, json)),
                    Err(e) => Ok(resp(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("failed to serialize output: {e}"),
                    )),
                },
                Ok(Err(err)) => Ok(resp(StatusCode::INTERNAL_SERVER_ERROR, format!("{:#}", err))),
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
    env_logger::init();

    // Initialize global engine config with defaults
    init_global_engine_config(EngineConfig::default())?;

    let (tx_router, rx_router) = mpsc::channel::<FuncJob>(1024);

    let num_workers = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    log::info!("spawning {} workers", num_workers);

    let mut worker_senders: Vec<mpsc::Sender<FuncJob>> = Vec::with_capacity(num_workers);

    // Build engine from the global config
    let detersl_engine: DeterSLEngine = require_global_engine_config().build_detersl_engine()?;

    for i in 0..num_workers {
        let (tx_w, rx_w) = mpsc::channel::<FuncJob>(1);

        let detersl_engine_clone = detersl_engine.clone();
        std::thread::Builder::new()
            .name(format!("worker-{i}"))
            .spawn(move || {
                let mut worker = Worker::from_parts(detersl_engine_clone);
                worker.run_forever(rx_w);
            })
            .expect("spawn worker thread");

        worker_senders.push(tx_w);
    }

    tokio::spawn(async move {
        route_jobs(rx_router, worker_senders).await;
    });

    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Listening on http://{addr}");

    loop {
        let (stream, _peer) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let tx_conn = tx_router.clone();

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

async fn route_jobs(
    mut rx_router: mpsc::Receiver<FuncJob>,
    mut workers: Vec<mpsc::Sender<FuncJob>>,
) {
    use tokio::sync::mpsc::error::TrySendError;

    if workers.is_empty() {
        log::error!("no workers available; dropping all incoming jobs");
        while let Some(job) = rx_router.recv().await {
            let _ = job
                .reply
                .send(Err(anyhow!("no workers available")))
                .map_err(|_| ());
        }
        return;
    }

    let mut next_idx: usize = 0;

    'route: while let Some(mut job) = rx_router.recv().await {
        let n = workers.len();
        let mut i = 0;
        while i < n {
            let idx = (next_idx + i) % n;
            match workers[idx].try_send(job) {
                Ok(()) => {
                    next_idx = (idx + 1) % n;
                    continue 'route;
                }
                Err(TrySendError::Full(j)) => {
                    job = j;
                    i += 1;
                    continue;
                }
                Err(TrySendError::Closed(j)) => {
                    job = j;
                    let _ = workers.remove(idx);
                    if workers.is_empty() {
                        let _ = job
                            .reply
                            .send(Err(anyhow!("all workers closed")))
                            .map_err(|_| ());
                        continue 'route;
                    }
                    next_idx %= workers.len();
                    i = 0;
                    continue;
                }
            }
        }

        if workers.is_empty() {
            continue;
        }

        let idx = next_idx % workers.len();
        match workers[idx].send(job).await {
            Ok(()) => {
                next_idx = (idx + 1) % workers.len();
            }
            Err(_e) => {
                let _ = workers.remove(idx);
                if workers.is_empty() {
                    log::error!("all workers closed while routing");
                }
                next_idx %= workers.len().max(1);
            }
        }
    }

    log::info!("router channel closed; exiting router task");
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
