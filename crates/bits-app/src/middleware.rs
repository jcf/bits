use crate::tenant::{resolve_realm, Realm};
use crate::AppState;
use dioxus::server::axum::{extract::Request, response::Response};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct RealmLayer;

impl<S> Layer<S> for RealmLayer {
    type Service = RealmMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RealmMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct RealmMiddleware<S> {
    inner: S,
}

impl<S> Service<Request> for RealmMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + Clone + 'static,
    S::Future: Send,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response, S::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request) -> Self::Future {
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let realm = if let Some(app_state) = req.extensions().get::<AppState>() {
                if let Some(host) = crate::http::extract_host(&req) {
                    let scheme = crate::http::extract_scheme(&req);
                    resolve_realm(app_state, scheme, &host).await
                } else {
                    Realm::Platform
                }
            } else {
                tracing::warn!("AppState not found in request extensions");
                Realm::Platform
            };

            req.extensions_mut().insert(realm);
            inner.call(req).await
        })
    }
}
