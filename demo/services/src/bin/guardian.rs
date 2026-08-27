//! Battery Thermal Guardian
//!
//! Subscribes to BatteryTempEvent over the uProtocol/Zenoh bus,
//! logs received values, and exposes /health + /state HTTP endpoints.

use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use dr_whodunit_services::{
    decode_json_payload, evaluate_thermal_state, make_uri_provider, now_ms,
    open_up_transport, vss_battery_temp_uri, vss_battery_high_temp_uri,
    BatteryTempEvent, GuardianSnapshot, GuardianState, HighTempAlert,
};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tracing::{info, warn};
use up_rust::communication::{InMemoryRpcClient};
use up_rust::{UListener, UMessage, UTransport};

// =============================================================================
// App state
// =============================================================================

#[derive(Clone)]
struct AppState {
    data: Arc<Mutex<GuardianRuntime>>,
}

#[derive(Debug)]
struct GuardianRuntime {
    temp_max: f32,
    temp_avg: f32,
    soc: f32,
    current_state: GuardianState,
    prev_temp: Option<f32>,
    prev_time: Option<Instant>,
}

impl GuardianRuntime {
    fn new() -> Self {
        Self {
            temp_max: 0.0,
            temp_avg: 0.0,
            soc: 0.0,
            current_state: GuardianState::Clear,
            prev_temp: None,
            prev_time: None,
        }
    }

    fn apply_event(&mut self, event: BatteryTempEvent) {
        self.temp_max = event.temp_max;
        self.temp_avg = event.temp_avg;
        self.soc = event.soc;

        let rate = match (self.prev_temp, self.prev_time) {
            (Some(prev), Some(t)) => {
                let mins = t.elapsed().as_secs_f32() / 60.0;
                if mins > 0.0 { (self.temp_max - prev) / mins } else { 0.0 }
            }
            _ => 0.0,
        };

        let new_state = if self.current_state == GuardianState::Clear {
            GuardianState::Monitoring
        } else {
            evaluate_thermal_state(self.temp_max)
        };

        info!(
            "TempMax: {:.1}C | TempAvg: {:.1}C | SoC: {:.0}% | Rate: {:.2}C/min -> {:?}",
            self.temp_max, self.temp_avg, self.soc, rate, new_state
        );

        self.current_state = new_state;
        self.prev_temp = Some(self.temp_max);
        self.prev_time = Some(Instant::now());
    }

    fn snapshot(&self) -> GuardianSnapshot {
        GuardianSnapshot {
            state: self.current_state,
            temp_max: self.temp_max,
            temp_avg: self.temp_avg,
            soc: self.soc,
            timestamp_ms: now_ms(),
        }
    }
}

// =============================================================================
// Listeners
// =============================================================================

struct BatteryTempListener {
    app: AppState,
    #[allow(dead_code)] // retained for future state-publish use
    transport: Arc<dyn UTransport>,
}

#[async_trait]
impl UListener for BatteryTempListener {
    async fn on_receive(&self, message: UMessage) {
        match decode_json_payload::<BatteryTempEvent>(&message) {
            Ok(event) => {
                info!("[Guardian] Battery Temperature Received = {:.1}C", event.temp_max);
                self.app.data.lock().await.apply_event(event);
            }
            Err(e) => warn!("Invalid battery event payload: {}", e),
        }
    }
}

struct HighTempListener;

#[async_trait]
impl UListener for HighTempListener {
    async fn on_receive(&self, message: UMessage) {
        match decode_json_payload::<HighTempAlert>(&message) {
            Ok(alert) => warn!(
                "[Guardian] HIGH TEMPERATURE ALERT: {:.1}C (severity: {}, source: {})",
                alert.value, alert.severity, alert.source
            ),
            Err(e) => warn!("Invalid high-temp alert payload: {}", e),
        }
    }
}

// =============================================================================
// HTTP handlers
// =============================================================================

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn get_state(State(app): State<AppState>) -> Json<GuardianSnapshot> {
    let guard = app.data.lock().await;
    Json(guard.snapshot())
}

// =============================================================================
// Main
// =============================================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "guardian=info,info".to_string()),
        )
        .init();

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("{}:{}", host, port);

    let uri_provider = make_uri_provider("guardian", 0x1001, 0x01);
    let transport = open_up_transport(uri_provider.clone()).await?;
    // rpc_client is initialised here; wire it into a listener when mitigation RPC is needed
    let _rpc_client = Arc::new(InMemoryRpcClient::new(transport.clone(), uri_provider).await?);

    let app_state = AppState {
        data: Arc::new(Mutex::new(GuardianRuntime::new())),
    };

    transport
        .register_listener(
            &vss_battery_temp_uri(),
            None,
            Arc::new(BatteryTempListener {
                app: app_state.clone(),
                transport: transport.clone(),
            }),
        )
        .await?;

    transport
        .register_listener(
            &vss_battery_high_temp_uri(),
            None,
            Arc::new(HighTempListener),
        )
        .await?;

    info!("[Guardian] Subscribed to {} -- waiting for VSS events", vss_battery_temp_uri().to_uri(false));

    let router = Router::new()
        .route("/health", get(health))
        .route("/state", get(get_state))
        .with_state(app_state);

    let listener = TcpListener::bind(&addr).await?;
    info!("[Guardian] HTTP server listening on {}", addr);

    axum::serve(listener, router)
        .with_graceful_shutdown(async { let _ = tokio::signal::ctrl_c().await; })
        .await?;

    Ok(())
}
