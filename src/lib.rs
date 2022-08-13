use lz_fnv::{Fnv1a, FnvHasher};
use tide::{Body, Request};

pub struct EtagMiddleware {}

impl Default for EtagMiddleware {
    fn default() -> Self {
        Self {}
    }
}

#[tide::utils::async_trait]
impl<State: Clone + Send + Sync + 'static> tide::Middleware<State> for EtagMiddleware {
    async fn handle(&self, req: Request<State>, next: tide::Next<'_, State>) -> tide::Result {
        let if_none_match = req.header(tide::http::headers::IF_NONE_MATCH).cloned();
        let mut response = next.run(req).await;
        if let Some(existing_etag) = response.header(tide::http::headers::ETAG) {
            if let Some(req_hash) = if_none_match {
                if req_hash.last().as_str() == existing_etag.last().as_str() {
                    response.set_status(304);
                    response.take_body();
                    return Ok(response);
                }
            }
            return Ok(response);
        }
        let body = response.take_body();
        let mut hasher = Fnv1a::<u32>::new();
        let bytes = &body.into_bytes().await?;
        hasher.write(&bytes);
        let hash = hasher.finish();
        let generated_etag = base64::encode(hash.to_be_bytes());
        response.append_header(tide::http::headers::ETAG, &generated_etag);
        if let Some(req_hash) = if_none_match {
            if req_hash.last().as_str() == generated_etag {
                response.set_status(304);
                return Ok(response);
            }
        }
        response.set_body(Body::from_bytes(bytes.to_vec()));
        Ok(response)
    }
}
