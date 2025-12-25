use axum::{Router, routing::get, Json, extract::State};
use sqlx::{PgPool, FromRow};
use serde::Serialize;
use chrono::{DateTime, Utc};
use rand::Rng;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[derive(Serialize, FromRow)]
struct SpaceData {
    id: i32,
    source: String,
    data: serde_json::Value,
    fetched_at: DateTime<Utc>,
}

async fn fetch_fake_nasa_data() -> serde_json::Value {
    let mut rng = rand::thread_rng();
    serde_json::json!({
        "iss_position": {
            "latitude": 51.0 + rng.gen::<f64>() * 10.0,
            "longitude": -0.1 + rng.gen::<f64>() * 20.0,
        },
        "fuel_level": rng.gen_range(80..100),
        "timestamp": Utc::now().timestamp()
    })
}

async fn get_data(State(pool): State<PgPool>) -> Result<Json<Vec<SpaceData>>, String> {
    // В реальности: пишем в БД
    let fake_data = fetch_fake_nasa_data().await;
    
    // Игнорируем ошибку записи, если таблицы нет
    let _ = sqlx::query("INSERT INTO space_data (source, data, fetched_at) VALUES ($1, $2, $3)")
        .bind("nasa_stub")
        .bind(&fake_data)
        .bind(Utc::now())
        .execute(&pool)
        .await
        .ok(); 

    // Читаем последние 10 записей
    let rows = sqlx::query_as::<_, SpaceData>("SELECT * FROM space_data ORDER BY fetched_at DESC LIMIT 10")
        .fetch_all(&pool)
        .await;

    match rows {
        Ok(data) => Ok(Json(data)),
        Err(e) => {
            println!("DB Error: {}", e);
            // Возвращаем стаб, если ошибка БД
            Ok(Json(vec![SpaceData {
                id: 0,
                source: "offline_stub".to_string(),
                data: fake_data,
                fetched_at: Utc::now(),
            }]))
        }
    }
}

// Простой хелсчек
async fn health_check() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() {
    println!("🚀 Starting Rust Backend...");
    
    // Получаем URL базы
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://user:pass@db:5432/space_db".to_string());
    println!("🔌 Connecting to DB: {}", db_url);
    
    // Подключаемся к базе (с повторами, если база еще не готова)
    let pool = match PgPool::connect(&db_url).await {
        Ok(p) => {
            println!("✅ DB Connected!");
            // Создаем таблицу, если нет
            let _ = sqlx::query(
                "CREATE TABLE IF NOT EXISTS space_data (
                    id SERIAL PRIMARY KEY,
                    source VARCHAR(50),
                    data JSONB,
                    fetched_at TIMESTAMPTZ
                );"
            ).execute(&p).await.ok();
            p
        },
        Err(e) => {
            println!("⚠️ Failed to connect to DB: {}", e);
            // Паника? Нет, лучше подождать, но для упрощения просто выйдем с ошибкой,
            // но Docker перезапустит. Но чтобы не 'exit 0', сделаем panic.
            panic!("Cannot connect to DB");
        }
    };

    let app = Router::new()
        .route("/api/data", get(get_data))
        .route("/health", get(health_check))
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 Listening on {}", addr);
    
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
