use std::future::ready;
use std::path::PathBuf;
use std::pin::Pin;

use headers::ContentType;
use headers::HeaderMapExt;
use net_traits::request::Request;
use net_traits::response::{Response, ResponseBody};
use net_traits::ResourceFetchTiming;

use crate::fetch::methods::{DoneChannel, FetchContext};
use crate::protocols::ProtocolHandler;

pub struct DirectoryProtocolHandler {
    root: PathBuf,
}

impl DirectoryProtocolHandler {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl ProtocolHandler for DirectoryProtocolHandler {
    fn load<'a>(
        &'a self,
        request: &'a mut Request,
        _done_chan: &mut DoneChannel,
        _context: &FetchContext,
    ) -> Pin<Box<dyn std::future::Future<Output = Response> + Send + 'a>> {
        let url = request.current_url();
        let rel = url.path().trim_start_matches('/');
        let path = self.root.join(rel);

        let content = std::fs::read(&path).unwrap_or_else(|_| {
            format!("<html><body>404: {}</body></html>", path.display()).into_bytes()
        });

        let mut response =
            Response::new(url.clone(), ResourceFetchTiming::new(request.timing_type()));

        let mime = mime_guess::from_path(&path).first_or_octet_stream();
        response.headers.typed_insert(ContentType::from(mime));
        *response.body.lock() = ResponseBody::Done(content);

        Box::pin(ready(response))
    }

    fn is_fetchable(&self) -> bool {
        true
    }

    fn is_secure(&self) -> bool {
        true
    }
}
