use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    extract::{Extension, Path, Query},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post},
    Json,
    extract::Request,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{info, warn};

use aio_api_gateway::{Route, RouteTable};
use aio_configuration_manager::ConfigurationStore;
use aio_core_engine::{CoreEngine, ServiceDescriptor};
use aio_permission_manager::PermissionStore;
use aio_plugin_manager::{PluginManifest, PluginRegistry};

// ─── Shared State ───────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct AppState {
    pub api_keys: Vec<String>,
    pub engine: Arc<tokio::sync::Mutex<CoreEngine>>,
    pub plugins: Arc<tokio::sync::Mutex<PluginRegistry>>,
    pub config: Arc<tokio::sync::Mutex<ConfigurationStore>>,
    pub permissions: Arc<tokio::sync::Mutex<PermissionStore>>,
    pub routes: Arc<tokio::sync::Mutex<RouteTable>>,
}

impl AppState {
    pub fn new(api_keys: Vec<String>) -> Self {
        let mut engine = CoreEngine::new();
        engine
            .register_service(ServiceDescriptor {
                id: "plugin-manager".into(),
                version: "0.1.0".into(),
            })
            .expect("built-in service should register");

        let mut route_table = RouteTable::new();
        for (method, path, svc) in ROUTES {
            route_table
                .register(Route {
                    method: method.to_string(),
                    path: path.to_string(),
                    target_service: svc.to_string(),
                })
                .expect("built-in route should register");
        }

        Self {
            api_keys,
            engine: Arc::new(tokio::sync::Mutex::new(engine)),
            plugins: Arc::new(tokio::sync::Mutex::new(PluginRegistry::new())),
            config: Arc::new(tokio::sync::Mutex::new(ConfigurationStore::new())),
            permissions: Arc::new(tokio::sync::Mutex::new(PermissionStore::new())),
            routes: Arc::new(tokio::sync::Mutex::new(route_table)),
        }
    }
}

const ROUTES: &[(&str, &str, &str)] = &[
    ("GET", "/v1/health", "health"),
    ("GET", "/v1/services", "core-engine"),
    ("GET", "/v1/plugins", "plugin-manager"),
    ("POST", "/v1/plugins", "plugin-manager"),
    ("GET", "/v1/config/:scope/:key", "configuration-manager"),
    ("PUT", "/v1/config/:scope/:key", "configuration-manager"),
    ("POST", "/v1/permissions/request", "permission-manager"),
    ("POST", "/v1/permissions/grant", "permission-manager"),
    ("GET", "/v1/permissions/check", "permission-manager"),
    ("GET", "/v1/routes", "api-gateway"),
];

// ─── API Key Middleware ──────────────────────────────────────────────────────

async fn require_api_key(
    headers: HeaderMap,
    Extension(state): Extension<AppState>,
    req: Request,
    next: Next,
) -> impl IntoResponse {
    // Accept "Authorization: Bearer <key>" or "X-Api-Key: <key>"
    let provided = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .or_else(|| {
            headers
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
        });

    match provided {
        Some(key) if state.api_keys.iter().any(|k| k == key) => {
            Ok(next.run(req).await)
        }
        _ => {
            warn!("rejected request: invalid or missing API key");
            Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "unauthorized",
                    "message": "A valid API key is required. Provide it via X-Api-Key header or Authorization: Bearer <key>.",
                    "docs": "https://github.com/aio-mcp/aio-mcp#authentication"
                })),
            ))
        }
    }
}

// ─── Route Handlers ─────────────────────────────────────────────────────────

async fn health() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "service": "aio-mcp-server",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn list_services(
    Extension(state): Extension<AppState>,
) -> impl IntoResponse {
    let engine = state.engine.lock().await;
    Json(json!({
        "service_count": engine.service_count()
    }))
}

// ── Plugins ─────────────────────────────────────────────────────────────────

async fn list_plugins(
    Extension(state): Extension<AppState>,
) -> impl IntoResponse {
    let registry = state.plugins.lock().await;
    let plugins: Vec<_> = registry
        .list()
        .iter()
        .map(|p| json!({ "id": p.id, "version": p.version }))
        .collect();
    Json(json!({ "plugins": plugins, "count": plugins.len() }))
}

#[derive(Deserialize)]
struct RegisterPluginBody {
    id: String,
    version: String,
}

async fn register_plugin(
    Extension(state): Extension<AppState>,
    Json(body): Json<RegisterPluginBody>,
) -> impl IntoResponse {
    let mut registry = state.plugins.lock().await;
    match registry.add(PluginManifest {
        id: body.id.clone(),
        version: body.version.clone(),
    }) {
        Ok(()) => (
            StatusCode::CREATED,
            Json(json!({
                "id": body.id,
                "version": body.version,
                "status": "registered"
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

// ── Config ───────────────────────────────────────────────────────────────────

async fn get_config(
    Path((scope, key)): Path<(String, String)>,
    Extension(state): Extension<AppState>,
) -> impl IntoResponse {
    let store = state.config.lock().await;
    match store.get(&scope, &key) {
        Some(entry) => (
            StatusCode::OK,
            Json(json!({
                "scope": entry.key.scope,
                "key": entry.key.key,
                "value": entry.value,
                "revision": entry.revision
            })),
        ),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "not_found", "scope": scope, "key": key })),
        ),
    }
}

#[derive(Deserialize)]
struct SetConfigBody {
    value: Value,
    expected_revision: Option<u64>,
}

async fn set_config(
    Path((scope, key)): Path<(String, String)>,
    Extension(state): Extension<AppState>,
    Json(body): Json<SetConfigBody>,
) -> impl IntoResponse {
    let mut store = state.config.lock().await;
    match store.upsert(&scope, &key, body.value, body.expected_revision) {
        Ok(entry) => (
            StatusCode::OK,
            Json(json!({
                "scope": entry.key.scope,
                "key": entry.key.key,
                "value": entry.value,
                "revision": entry.revision
            })),
        ),
        Err(e) => (
            StatusCode::CONFLICT,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

// ── Permissions ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct RequestPermBody {
    principal_id: String,
    scope: String,
}

async fn request_permission(
    Extension(state): Extension<AppState>,
    Json(body): Json<RequestPermBody>,
) -> impl IntoResponse {
    let mut store = state.permissions.lock().await;
    match store.request(&body.principal_id, &body.scope) {
        Ok(grant) => (
            StatusCode::CREATED,
            Json(json!({
                "principal_id": grant.principal_id,
                "scope": grant.scope,
                "state": format!("{:?}", grant.state)
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

#[derive(Deserialize)]
struct GrantPermBody {
    principal_id: String,
    scope: String,
}

async fn grant_permission(
    Extension(state): Extension<AppState>,
    Json(body): Json<GrantPermBody>,
) -> impl IntoResponse {
    let mut store = state.permissions.lock().await;
    match store.set_state(
        &body.principal_id,
        &body.scope,
        aio_permission_manager::GrantState::Granted,
    ) {
        Ok(grant) => (
            StatusCode::OK,
            Json(json!({
                "principal_id": grant.principal_id,
                "scope": grant.scope,
                "state": format!("{:?}", grant.state)
            })),
        ),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

#[derive(Deserialize)]
struct CheckPermQuery {
    principal_id: String,
    scope: String,
}

async fn check_permission(
    Query(q): Query<CheckPermQuery>,
    Extension(state): Extension<AppState>,
) -> impl IntoResponse {
    let store = state.permissions.lock().await;
    let allowed = store.can_access(&q.principal_id, &q.scope);
    Json(json!({
        "principal_id": q.principal_id,
        "scope": q.scope,
        "allowed": allowed
    }))
}

// ── Gateway Routes ───────────────────────────────────────────────────────────

async fn list_routes(
    Extension(state): Extension<AppState>,
) -> impl IntoResponse {
    let table = state.routes.lock().await;
    // Route table inspection (read the registered routes from its internal state via a resolve loop)
    let known: Vec<Value> = ROUTES
        .iter()
        .filter_map(|(method, path, _)| {
            table
                .resolve(method, path)
                .ok()
                .map(|r| json!({ "method": r.method, "path": r.path, "target": r.target_service }))
        })
        .collect();
    Json(json!({ "routes": known, "count": known.len() }))
}

// ─── Main ────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aio_server=info,tower_http=info".parse().unwrap()),
        )
        .init();

    // API keys from environment: AIO_API_KEYS="key1,key2,key3"
    let api_keys: Vec<String> = std::env::var("AIO_API_KEYS")
        .unwrap_or_else(|_| "changeme".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if api_keys.iter().any(|k| k == "changeme") {
        tracing::warn!(
            "Using default API key 'changeme'. Set AIO_API_KEYS env var before deploying to production."
        );
    }

    let state = AppState::new(api_keys);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    // Public routes — no auth
    let public_routes = Router::new()
        .route("/health", get(health))
        .route("/", get(|| async {
            Json(json!({
                "service": "aio-mcp-server",
                "version": env!("CARGO_PKG_VERSION"),
                "docs": "/v1/routes"
            }))
        }));

    // Protected routes — require API key
    let protected_routes = Router::new()
        .route("/v1/services", get(list_services))
        .route("/v1/plugins", get(list_plugins).post(register_plugin))
        .route("/v1/config/:scope/:key", get(get_config).put(set_config))
        .route("/v1/permissions/request", post(request_permission))
        .route("/v1/permissions/grant", post(grant_permission))
        .route("/v1/permissions/check", get(check_permission))
        .route("/v1/routes", get(list_routes))
        .layer(middleware::from_fn_with_state(state.clone(), require_api_key));

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(Extension(state))
        .layer(
            tower_http::cors::CorsLayer::permissive(),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("aio-mcp-server listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
