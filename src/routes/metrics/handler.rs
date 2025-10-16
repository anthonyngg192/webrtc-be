// use crate::services::metrics_service::REGISTRY;
use actix_web::{get, HttpResponse};
use prometheus::{Encoder, TextEncoder};

#[get("metrics")]
pub async fn metrics() -> HttpResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    HttpResponse::Ok()
        .content_type(encoder.format_type())
        .body(buffer)
}
