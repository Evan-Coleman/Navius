mod jwt;

use axum::http::StatusCode;
use jwt::Claims;
use serde::Deserialize;

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(SqlxPlugin)
        .add_plugin(WebPlugin)
        .run()
        .await;

    tracing::info!("Server Shutdown")
}

#[routes]
#[get("/")]
#[get("/hello_world")]
async fn hello_world() -> impl IntoResponse {
    "hello world"
}

#[route("/hello/{name}", method = "GET", method = "POST")]
async fn hello(Path(name): Path<String>) -> impl IntoResponse {
    format!("hello {name}")
}

#[derive(Deserialize)]
struct LoginCredentials {
    username: String,
    password: String,
}

#[post("/login")]
async fn login(Json(credentials): Json<LoginCredentials>) -> Result<impl IntoResponse> {
    let LoginCredentials { username, password } = credentials;
    if username == "root" && password == "correct_password" {
        let mock_user_id = 1000;
        let jwt_token = jwt::encode(Claims::new(mock_user_id))?;
        Ok((StatusCode::OK, jwt_token))
    } else {
        Ok((
            StatusCode::BAD_REQUEST,
            format!("{username} login failed: username or password are incorrect"),
        ))
    }
}

#[derive(Configurable, Deserialize)]
#[config_prefix = "custom"]
struct CustomConfig {
    user_info_detail: String,
}

#[get("/user-info")]
async fn protected_user_info(
    claims: Claims,
    Config(conf): Config<CustomConfig>,
) -> impl IntoResponse {
    let user_id = claims.uid;
    format!("get user info of id#{}: {}", user_id, conf.user_info_detail)
}

#[nest("/sql")]
mod sql {
    use anyhow::Context;
    use std::ops::Deref;

    #[get("/version")]
    pub async fn sqlx_request_handler(Component(pool): Component<ConnectPool>) -> Result<String> {
        let version = sqlx::query("select version() as version")
            .fetch_one(&pool)
            .await
            .context("sqlx query failed")?
            .get("version");
        Ok(version)
    }

    #[get("/now")]
    pub async fn sqlx_time_handler(pool: Component<ConnectPool>) -> Result<String> {
        let time = sqlx::query("select DATE_FORMAT(now(),'%Y-%m-%d %H:%i:%s') as time")
            .fetch_one(pool.deref())
            .await
            .context("sqlx query failed")?
            .get("time");
        Ok(time)
    }
}