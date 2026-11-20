#[tokio::main]
async fn main() -> Result<(), vercel_runtime::Error> {
    frizzi::init_tracing();
    let app = tower::ServiceBuilder::new()
        .map_request(
            |(state, mut request): (vercel_runtime::AppState, vercel_runtime::Request)| {
                request.extensions_mut().insert(state.clone());
                (state, request)
            },
        )
        .layer(vercel_runtime::axum::VercelLayer::new())
        .service(frizzi::app().await);
    vercel_runtime::run(app).await
}
