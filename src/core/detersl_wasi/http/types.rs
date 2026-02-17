use std::{any::Any, time::Duration};

use http_body_util::BodyExt;
use tokio::{time::timeout, net::TcpStream};
use wasmtime_wasi::{ResourceTable, p2::{IoView, IoImpl, Pollable}, runtime::AbortOnDropJoinHandle};
use hyper::header::HeaderName;

use super::{bindings::detersl::http_api::types::{Method, Scheme, ErrorCode}, body::HostIncomingBody, errors::{dns_error, hyper_request_error}, io::TokioIo};

use super::{body::{HyperOutgoingBody, HyperIncomingBody}, errors::HttpResult};

pub struct DeterSLHttpCtx {
    _priv: ()
}

impl DeterSLHttpCtx {
    pub fn new() -> Self {
        DeterSLHttpCtx {_priv: () }
    }
}

pub trait DeterSLHttpView: IoView {
    fn ctx(&mut self) -> &mut DeterSLHttpCtx;

    /// Send an outgoing request.
    fn send_request(
        &mut self,
        request: hyper::Request<HyperOutgoingBody>
    ) -> HttpResult<HostFutureIncomingResponse> {
        //let income = HostFutureIncomingResponse::Consumed; 
        // TODO
        //
        //println!("heloooooo");
        //Ok(income)
        Ok(default_send_request(request))
    }

    /// Whether a given header should be considered forbidden and not allowed.
    fn is_forbidden_header(&mut self, _name: &hyper::header::HeaderName) -> bool {
        false
    }

    /// Number of distinct write calls to the outgoing body's output-stream
    /// that the implementation will buffer.
    /// Default: 1.
    fn outgoing_body_buffer_chunks(&mut self) -> usize {
        DEFAULT_OUTGOING_BODY_BUFFER_CHUNKS
    }

    /// Maximum size allowed in a write call to the outgoing body's output-stream.
    /// Default: 1024 * 1024.
    fn outgoing_body_chunk_size(&mut self) -> usize {
        DEFAULT_OUTGOING_BODY_CHUNK_SIZE
    }
}

impl<T: ?Sized + DeterSLHttpView> DeterSLHttpView for &mut T {
    fn ctx(&mut self) -> &mut DeterSLHttpCtx {
        T::ctx(self)
    }

    /// Send an outgoing request.
    fn send_request(
        &mut self,
        request: hyper::Request<HyperOutgoingBody>
    ) -> HttpResult<HostFutureIncomingResponse> {
        T::send_request(self, request)
    }

    /// Whether a given header should be considered forbidden and not allowed.
    fn is_forbidden_header(&mut self, _name: &hyper::header::HeaderName) -> bool {
        T::is_forbidden_header(self, _name)
    }

    /// Number of distinct write calls to the outgoing body's output-stream
    /// that the implementation will buffer.
    /// Default: 1.
    fn outgoing_body_buffer_chunks(&mut self) -> usize {
        T::outgoing_body_buffer_chunks(self)
    }

    /// Maximum size allowed in a write call to the outgoing body's output-stream.
    /// Default: 1024 * 1024.
    fn outgoing_body_chunk_size(&mut self) -> usize {
        T::outgoing_body_chunk_size(self)
    }
}

impl<T: ?Sized + DeterSLHttpView> DeterSLHttpView for Box<T> {
    fn ctx(&mut self) -> &mut DeterSLHttpCtx {
        T::ctx(self)
    }

    /// Send an outgoing request.
    fn send_request(
        &mut self,
        request: hyper::Request<HyperOutgoingBody>
    ) -> HttpResult<HostFutureIncomingResponse> {
        T::send_request(self, request)
    }

    /// Whether a given header should be considered forbidden and not allowed.
    fn is_forbidden_header(&mut self, _name: &hyper::header::HeaderName) -> bool {
        T::is_forbidden_header(self, _name)
    }

    /// Number of distinct write calls to the outgoing body's output-stream
    /// that the implementation will buffer.
    /// Default: 1.
    fn outgoing_body_buffer_chunks(&mut self) -> usize {
        T::outgoing_body_buffer_chunks(self)
    }

    /// Maximum size allowed in a write call to the outgoing body's output-stream.
    /// Default: 1024 * 1024.
    fn outgoing_body_chunk_size(&mut self) -> usize {
        T::outgoing_body_chunk_size(self)
    }
}

/// The default value configured for [`WasiHttpView::outgoing_body_buffer_chunks`] in [`WasiHttpView`].
pub const DEFAULT_OUTGOING_BODY_BUFFER_CHUNKS: usize = 1;
/// The default value configured for [`WasiHttpView::outgoing_body_chunk_size`] in [`WasiHttpView`].
pub const DEFAULT_OUTGOING_BODY_CHUNK_SIZE: usize = 1024 * 1024;

pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(60);

pub const FIRST_BYTE_CONNECT_TIMEOUT: Duration = Duration::from_secs(60);

pub const CHUNKS_TIMEOUT: Duration = Duration::from_secs(60);

pub(crate) fn is_forbidden_header(view: &mut dyn DeterSLHttpView, name: &HeaderName) -> bool {
    static FORBIDDEN_HEADERS: [HeaderName; 9] = [
        hyper::header::CONNECTION,
        HeaderName::from_static("keep-alive"),
        hyper::header::PROXY_AUTHENTICATE,
        hyper::header::PROXY_AUTHORIZATION,
        HeaderName::from_static("proxy-connection"),
        hyper::header::TRANSFER_ENCODING,
        hyper::header::UPGRADE,
        hyper::header::HOST,
        HeaderName::from_static("http2-settings"),
    ];

    FORBIDDEN_HEADERS.contains(name) || view.is_forbidden_header(name)
}

/// The default implementation of how an outgoing request is sent.
///
/// This implementation is used by the `wasi:http/outgoing-handler` interface
/// default implementation.
pub fn default_send_request(
    request: hyper::Request<HyperOutgoingBody>
) -> HostFutureIncomingResponse {
    let handle = wasmtime_wasi::runtime::spawn(async move {
        Ok(default_send_request_handler(request).await)
    });
    HostFutureIncomingResponse::pending(handle)
}

/// The underlying implementation of how an outgoing request is sent. This should likely be spawned
/// in a task.
///
/// This is called from [default_send_request] to actually send the request.
pub async fn default_send_request_handler(
    mut request: hyper::Request<HyperOutgoingBody>
) -> Result<IncomingResponse, ErrorCode> {
    let authority = if let Some(authority) = request.uri().authority() {
        if authority.port().is_some() {
            authority.to_string()
        } else {
            let port = 80;
            format!("{}:{port}", authority.to_string())
        }
    } else {
        return Err(ErrorCode::HttpRequestUriInvalid);
    };
    let tcp_stream = timeout(CONNECT_TIMEOUT, TcpStream::connect(&authority))
        .await
        .map_err(|_| ErrorCode::ConnectionTimeout)?
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::AddrNotAvailable => {
                dns_error("address not available".to_string(), 0)
            }

            _ => {
                if e.to_string()
                    .starts_with("failed to lookup address information")
                {
                    dns_error("address not available".to_string(), 0)
                } else {
                    ErrorCode::ConnectionRefused
                }
            }
        })?;

    let (mut sender, _worker) = {
        let tcp_stream = TokioIo::new(tcp_stream);
        let (sender, conn) = timeout(
            CONNECT_TIMEOUT,
            // TODO: we should plumb the builder through the http context, and use it here
            hyper::client::conn::http1::handshake(tcp_stream),
        )
        .await
        .map_err(|_| ErrorCode::ConnectionTimeout)?
        .map_err(hyper_request_error)?;

        let worker = wasmtime_wasi::runtime::spawn(async move {
            match conn.await {
                Ok(()) => {}
                // TODO: same as above, shouldn't throw this error away.
                Err(_e) => println!("Oh!"),
            }
        });

        (sender, worker)
    };

    // at this point, the request contains the scheme and the authority, but
    // the http packet should only include those if addressing a proxy, so
    // remove them here, since SendRequest::send_request does not do it for us
    *request.uri_mut() = http::Uri::builder()
        .path_and_query(
            request
                .uri()
                .path_and_query()
                .map(|p| p.as_str())
                .unwrap_or("/"),
        )
        .build()
        .expect("comes from valid request");

    let resp = timeout(FIRST_BYTE_CONNECT_TIMEOUT, sender.send_request(request))
        .await
        .map_err(|_| ErrorCode::ConnectionReadTimeout)?
        .map_err(hyper_request_error)?
        .map(|body| body.map_err(hyper_request_error).boxed());

    Ok(IncomingResponse {
        resp,
        between_bytes_timeout: CHUNKS_TIMEOUT,
    })
}

/// Removes forbidden headers from a [`hyper::HeaderMap`].
pub(crate) fn remove_forbidden_headers(
    view: &mut dyn DeterSLHttpView,
    headers: &mut hyper::HeaderMap,
) {
    let forbidden_keys = Vec::from_iter(headers.keys().filter_map(|name| {
        if is_forbidden_header(view, name) {
            Some(name.clone())
        } else {
            None
        }
    }));

    for name in forbidden_keys {
        headers.remove(name);
    }
}


#[repr(transparent)]
pub struct DeterSLHttpImpl<T>(pub IoImpl<T>);

impl<T: IoView> IoView for DeterSLHttpImpl<T> {
    fn table(&mut self) -> &mut ResourceTable {
        T::table(&mut self.0.0)
    }
}

impl<T: DeterSLHttpView> DeterSLHttpView for DeterSLHttpImpl<T> {

    fn ctx(&mut self) -> &mut DeterSLHttpCtx {
        self.0.0.ctx()
    }

    fn send_request(
            &mut self,
            request: hyper::Request<HyperOutgoingBody>
        ) -> HttpResult<HostFutureIncomingResponse> {
        self.0.0.send_request(request)
    }
    
    /// Whether a given header should be considered forbidden and not allowed.
    fn is_forbidden_header(&mut self, _name: &hyper::header::HeaderName) -> bool {
        self.0.0.is_forbidden_header(_name)
    }

    /// Number of distinct write calls to the outgoing body's output-stream
    /// that the implementation will buffer.
    /// Default: 1.
    fn outgoing_body_buffer_chunks(&mut self) -> usize {
        self.0.0.outgoing_body_buffer_chunks()
    }

    /// Maximum size allowed in a write call to the outgoing body's output-stream.
    /// Default: 1024 * 1024.
    fn outgoing_body_chunk_size(&mut self) -> usize {
        self.0.0.outgoing_body_chunk_size()
    }
}

/// The concrete type behind a `wasi:http/types/fields` resource.
#[derive(Debug)]
pub enum HostFields {
    /// A reference to the fields of a parent entry.
    Ref {
        /// The parent resource rep.
        parent: u32,

        /// The function to get the fields from the parent.
        // NOTE: there's not failure in the result here because we assume that HostFields will
        // always be registered as a child of the entry with the `parent` id. This ensures that the
        // entry will always exist while this `HostFields::Ref` entry exists in the table, thus we
        // don't need to account for failure when fetching the fields ref from the parent.
        get_fields: for<'a> fn(elem: &'a mut (dyn Any + 'static)) -> &'a mut FieldMap,
    },
    /// An owned version of the fields.
    Owned {
        /// The fields themselves.
        fields: FieldMap,
    },
}

pub type FieldMap = hyper::HeaderMap;

pub type FutureIncomingResponseHandle = AbortOnDropJoinHandle<anyhow::Result<Result<IncomingResponse, ErrorCode>>>;

/// The concrete type behind a `wasi:http/types/outgoing-request` resource.
#[derive(Debug)]
pub struct HostOutgoingRequest {
    /// The method of the request.
    pub method: Method,
    /// The scheme of the request.
    pub scheme: Option<Scheme>,
    /// The authority of the request.
    pub authority: Option<String>,
    /// The path and query of the request.
    pub path_with_query: Option<String>,
    /// The request headers.
    pub headers: FieldMap,
    /// The request body.
    pub body: Option<HyperOutgoingBody>,
}

/// A response that is in the process of being received.
#[derive(Debug)]
pub struct IncomingResponse {
    /// The response itself.
    pub resp: hyper::Response<HyperIncomingBody>,
    /// The timeout between chunks of the response.
    pub between_bytes_timeout: std::time::Duration,
}

/// The concrete type behind a `wasi:http/types/future-incoming-response` resource.
#[derive(Debug)]
pub enum HostFutureIncomingResponse {
    /// A pending response
    Pending(FutureIncomingResponseHandle),
    /// The response is ready.
    ///
    /// An outer error will trap while the inner error gets returned to the guest.
    Ready(anyhow::Result<Result<IncomingResponse, ErrorCode>>),
    /// The response has been consumed.
    Consumed,
}

impl HostFutureIncomingResponse {
    /// Create a new `HostFutureIncomingResponse` that is pending on the provided task handle.
    pub fn pending(handle: FutureIncomingResponseHandle) -> Self {
        Self::Pending(handle)
    }

    /// Create a new `HostFutureIncomingResponse` that is ready.
    pub fn ready(result: anyhow::Result<Result<IncomingResponse, ErrorCode>>) -> Self {
        Self::Ready(result)
    }

    /// Returns `true` if the response is ready.
    #[allow(dead_code)] // TODO: fix later
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready(_))
    }

    /// Unwrap the response, panicking if it is not ready.
    pub fn unwrap_ready(self) -> anyhow::Result<Result<IncomingResponse, ErrorCode>> {
        match self {
            Self::Ready(res) => res,
            Self::Pending(_) | Self::Consumed => {
                panic!("unwrap_ready called on a pending HostFutureIncomingResponse")
            }
        }
    }
}

#[async_trait::async_trait]
impl Pollable for HostFutureIncomingResponse {
    async fn ready(&mut self) {
        if let Self::Pending(handle) = self {
            *self = Self::Ready(handle.await);
        }
    }
}

/// The concrete type behind a `wasi:http/types/incoming-response` resource.
#[derive(Debug)]
pub struct HostIncomingResponse {
    /// The response status
    pub status: u16,
    /// The response headers
    pub headers: FieldMap,
    /// The response body
    pub body: Option<HostIncomingBody>,
}
