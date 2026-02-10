use bytes::Bytes;
use http_body_util::{Empty, BodyExt};
use wasmtime::component::Resource;
use wasmtime_wasi::p2::IoView;

use super::{types::{DeterSLHttpImpl, DeterSLHttpView, HostOutgoingRequest, HostFutureIncomingResponse}, errors::{HttpResult, http_request_error, internal_error}};

use super::bindings::detersl::http_api::types::{Method, Scheme, ErrorCode};


impl<T> super::bindings::detersl::http_api::outgoing_handler::Host for DeterSLHttpImpl<T>
where
    T: DeterSLHttpView,
{
    fn handle(
        &mut self,
        request_id: Resource<HostOutgoingRequest>
    ) -> HttpResult<Resource<HostFutureIncomingResponse>> {

        let req = self.table().delete(request_id)?;
        let mut builder = hyper::Request::builder();

        builder = builder.method(match req.method {
            Method::Get => hyper::Method::GET,
            Method::Head => hyper::Method::HEAD,
            Method::Post => hyper::Method::POST,
            Method::Put => hyper::Method::PUT,
            Method::Delete => hyper::Method::DELETE,
            Method::Connect => hyper::Method::CONNECT,
            Method::Options => hyper::Method::OPTIONS,
            Method::Trace => hyper::Method::TRACE,
            Method::Patch => hyper::Method::PATCH,
            Method::Other(m) => match hyper::Method::from_bytes(m.as_bytes()) {
                Ok(method) => method,
                Err(_) => return Err(ErrorCode::HttpRequestMethodInvalid.into()),
            },
        });

        let (_use_tls, scheme) = match req.scheme.unwrap_or(Scheme::Https) {
            Scheme::Http => (false, http::uri::Scheme::HTTP),
            Scheme::Https => (true, http::uri::Scheme::HTTPS),

            // We can only support http/https
            Scheme::Other(_) => return Err(ErrorCode::HttpProtocolError.into()),
        };

        // Currently, we dont use tls
        
        let authority = req.authority.unwrap_or_else(String::new);

        builder = builder.header(hyper::header::HOST, &authority);

        let mut uri = http::Uri::builder()
            .scheme(scheme)
            .authority(authority.clone());

        if let Some(path) = req.path_with_query {
            uri = uri.path_and_query(path);
        }

        builder = builder.uri(uri.build().map_err(http_request_error)?);

        for (k, v) in req.headers.iter() {
            builder = builder.header(k, v);
        }

        let body = req.body.unwrap_or_else(|| {
            Empty::<Bytes>::new()
                .map_err(|_| unreachable!("Infallible error"))
                .boxed()
        });

        let request = builder
            .body(body)
            .map_err(|err| internal_error(err.to_string()))?;

        let future = self.send_request(
            request
        )?;

        Ok(self.table().push(future)?)
    }
}
