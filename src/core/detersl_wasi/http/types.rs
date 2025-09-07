use std::any::Any;

use wasmtime_wasi::{ResourceTable, p2::{IoView, IoImpl}};
use hyper::header::HeaderName;

use super::{bindings::detersl::http_api::types::{Method, Scheme, ErrorCode}, body::HostIncomingBody};

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
        let income = HostFutureIncomingResponse::Consumed; 
        Ok(income)
        //Ok(default_send_request(request))
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

pub type FutureIncomingResponseHandle = anyhow::Result<Result<IncomingResponse, ErrorCode>>;

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
