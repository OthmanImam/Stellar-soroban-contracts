use actix_web::{post, web, App, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};
use chrono::{Utc};

#[derive(Serialize, Deserialize)]
struct TelemetryEvent {
    contract_id: String,
    operation: String,
    dashboard_id: Option<u64>,
    widget_id: Option<u64>,
    status: String,
    gas_used: u64,
    timestamp: i64,
}

#[post("/telemetry")]
async fn ingest_telemetry(event: web::Json<TelemetryEvent>) -> HttpResponse {
    // TODO: Store event in database for 1-year retention
    // TODO: Trigger alerting if needed
    HttpResponse::Ok().json("Telemetry event ingested")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(ingest_telemetry)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
