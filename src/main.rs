#[macro_use]
extern crate rocket;
use std::{env, io::Cursor};

use rocket::{
    serde::{json::{Json, serde_json::json}, Deserialize, Serialize},
    State, response::{status, Responder, self}, Response, http::{Status, ContentType},
    Request,
};
use sqlx::{
    postgres::PgPoolOptions,
    Pool, Postgres, migrate::MigrateError,
};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
#[serde(crate = "rocket::serde")]
struct Role {
    /// The role's ID. Output only. Default value is `unknown_id()`.
    #[serde(default = "unknown_id")]
    id: i32,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Roles {
    /// The list of roles.
    roles: Vec<Role>,
}

fn unknown_id() -> i32 { -1 }

struct App {
    pool: Pool<Postgres>,
}

#[derive(thiserror::Error, Debug)]
enum AppError {
    #[error("database: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("migrate: {0}")]
    Migrate(#[from] MigrateError),

    #[error("connect to database {0:?}: {1}")]
    ConnectPostgres(String, sqlx::Error),
}

impl App {
    async fn new() -> Result<App, AppError> {
        // Connect to the database.
        let postgres_url = env::var("POSTGRES_URL").unwrap();
        let pool = PgPoolOptions::new()
            .max_connections(16)
            .connect(&postgres_url)
            .await;
        let pool = match pool {
            Ok(p) => p,
            Err(e) => {
                return Err(AppError::ConnectPostgres(postgres_url, e));
            },
        };

        // Run migrations.
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(App { pool })
    }

    async fn get_role(&self, id: i32) -> Result<Role, AppError> {
        let rows: Vec<Role> = sqlx::query_as("SELECT * FROM roles WHERE id = $1")
            .bind(&id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows[0].clone())
    }

    async fn list_roles(&self) -> Result<Roles, AppError> {
        let rows: Vec<Role> = sqlx::query_as("SELECT * FROM roles")
            .fetch_all(&self.pool)
            .await?;
        Ok(Roles { roles: rows })
    }

    async fn create_role(&self, role: Role) -> Result<Role, AppError> {
        let row: Role = sqlx::query_as("INSERT INTO roles (name) VALUES ($1) RETURNING *")
            .bind(&role.name)
            .fetch_one(&self.pool)
            .await?;
        Ok(row)
    }

    fn delete_role(&self, id: i64) {}

    fn update_role(&self, role: Role) -> Role {
        role
    }
}

const CODE_INTERNAL: u32 = 1;

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for AppError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let body = json!({
            "code": CODE_INTERNAL,
            "message": self.to_string(),
        }).to_string();
        Response::build()
            .status(Status::InternalServerError)
            .header(ContentType::JSON)
            .sized_body(body.len(), Cursor::new(body))
            .ok()
    }
}

#[get("/roles/<id>")]
async fn get_role(app: &State<App>, id: i32) -> Result<Json<Role>, AppError> {
    Ok(Json(app.get_role(id).await?))
}

#[get("/roles")]
async fn list_roles(app: &State<App>) -> Result<Json<Roles>, AppError> {
    match app.list_roles().await {
        Ok(roles) => Ok(Json(roles)),
        Err(err) => Err(err),
    }
}

#[post("/roles", format = "json", data = "<role>")]
async fn create_role(app: &State<App>, role: Json<Role>) -> Result<Json<Role>, AppError> {
    match app.create_role(role.into_inner()).await {
        Ok(role) => Ok(Json(role)),
        Err(err) => Err(err),
    }
}

#[delete("/roles/<id>")]
async fn delete_role(app: &State<App>, id: i64) {
    app.delete_role(id);
}

#[put("/roles/<id>", format = "json", data = "<role>")]
async fn update_role(app: &State<App>, id: i32, role: Json<Role>) -> Json<Role> {
    Json(app.update_role(Role {
        id,
        ..role.into_inner()
    }))
}

#[launch]
async fn rocket() -> _ {
    let app = match App::new().await {
        Ok(a) => a,
        Err(e) => panic!("create app: {}", e),
    };

    rocket::build()
        .mount(
            "/api/v1/",
            routes![get_role, create_role, delete_role, update_role, list_roles],
        )
        .manage(app)
}
