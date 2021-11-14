use std::net::SocketAddr;

use axum::{extract::Extension, extract::Json, handler::get, handler::post, response, Router};
use redis::{Client, Commands};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tower_http::add_extension::AddExtensionLayer;

use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let redis_conn_url = format!("redis://:{}@{}", "password", "127.0.0.1:6379");
    let conn = Arc::new(Mutex::new(
        Client::open(redis_conn_url).expect("invalid connection URL"),
    ));
    let state = Arc::clone(&conn);

    let app = Router::new()
        .route("/", get(handler))
        .route("/metric", post(post_metric))
        .layer(AddExtensionLayer::new(state));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler() -> response::Html<&'static str> {
    response::Html("<h1>Hello, World!</h1>")
}

#[derive(Serialize, Deserialize, Debug)]
struct Metric {
    id: String,
    timestamp: i64,
    value: String,
}

type RedisState = Arc<Mutex<Client>>;

async fn post_metric(
    client: Extension<RedisState>,
    Json(metric): Json<Metric>,
) -> response::Json<Value> {
    let mut conn = client
        .lock()
        .await
        .get_connection()
        .expect("failed to connect to Redis");

    let zvalue = format!(
        "{{\"timestamp\":{}, \"value\": \"{}\"}}",
        metric.timestamp.to_string(),
        metric.value
    );
    conn.zadd::<&str, i64, String, i64>(&*metric.id.clone(), zvalue, metric.timestamp)
        .expect("failed to execute ZADD");

    let values = conn
        .zrange::<&str, Vec<String>>(&*metric.id, 0, -1)
        .expect("failed to execute ZRANGE");

    println!("zvalues for {} = {:#?}", &*metric.id, values);
    response::Json(json!({ "result": "ok" }))
}
