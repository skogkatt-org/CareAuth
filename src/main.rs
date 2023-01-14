#[macro_use]
extern crate rocket;
use std::{env, format};

use rocket::{
    serde::{json::Json, Deserialize, Serialize},
    State, response::status,
};
use sqlx::{
    postgres::PgPoolOptions,
    Pool, Postgres,
};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
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

impl App {
    async fn new() -> Result<App, sqlx::Error> {
        // Connect to the database.
        let postgres_url = &env::var("POSTGRES_URL").unwrap();
        let pool = PgPoolOptions::new()
            .max_connections(16)
            .connect(&postgres_url)
            .await
            .expect(format!("connect to database {}", postgres_url).as_str());

        // Run migrations.
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(App { pool })
    }

    fn get_role(&self, id: i32) -> Role {
        Role {
            id: id,
            name: "admin".to_string(),
        }
    }

    async fn list_roles(&self) -> Result<Roles, sqlx::Error> {
        let rows: Vec<Role> = sqlx::query_as("SELECT * FROM roles")
            .fetch_all(&self.pool)
            .await?;
        return Ok(Roles {
            roles: rows,
        })
    }

    async fn create_role(&self, role: Role) -> Result<Role, sqlx::Error> {
        let row: (i32, String) = sqlx::query_as("INSERT INTO roles (name) VALUES ($1) RETURNING *")
            .bind(&role.name)
            .fetch_one(&self.pool)
            .await?;
        return Ok(Role {
            id: row.0,
            name: row.1,
        });
    }

    fn delete_role(&self, id: i64) {}

    fn update_role(&self, role: Role) -> Role {
        role
    }
}

#[get("/roles/<id>")]
async fn get_role(app: &State<App>, id: i32) -> Json<Role> {
    Json(app.get_role(id))
}

#[get("/roles")]
async fn list_roles(app: &State<App>) -> Result<Json<Roles>, status::Unauthorized<String>> {
    match app.list_roles().await {
        Ok(roles) => Ok(Json(roles)),
        Err(err) => Err(status::Unauthorized(Some(err.to_string()))),
    }
}

#[post("/roles", format = "json", data = "<role>")]
async fn create_role(app: &State<App>, role: Json<Role>) -> Result<Json<Role>, status::Unauthorized<String>> {
    match app.create_role(role.into_inner()).await {
        Ok(role) => Ok(Json(role)),
        Err(err) => Err(status::Unauthorized(Some(err.to_string()))),
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
    let app = App::new().await.expect("create app");

    rocket::build()
        .mount(
            "/api/v1/",
            routes![get_role, create_role, delete_role, update_role, list_roles],
        )
        .manage(app)
}
