use std::env;

use rocket::serde::{Deserialize, Serialize};
use sqlx::{migrate::MigrateError, postgres::PgPoolOptions, Pool, Postgres};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Role {
    /// The role's ID. Output only. Default value is `unknown_id()`.
    #[serde(default = "unknown_id")]
    pub id: i32,

    /// The role's name.
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Roles {
    /// The list of roles.
    pub roles: Vec<Role>,
}

fn unknown_id() -> i32 {
    -1
}

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("database: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("migrate: {0}")]
    Migrate(#[from] MigrateError),

    #[error("connect to database {0:?}: {1}")]
    ConnectPostgres(String, sqlx::Error),

    #[error("role not found")]
    RoleNotFound,
}

pub struct App {
    pool: Pool<Postgres>,
}

type AppResult<T> = Result<T, AppError>;

impl App {
    /// Create a new `App` instance. Which use PostgreSQL as its database by
    /// enviroment variable `POSTGRES_URL`.
    pub async fn new() -> AppResult<App> {
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
            }
        };

        // Run migrations.
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(App { pool })
    }

    pub async fn get_role(&self, id: i32) -> AppResult<Role> {
        let rows: Vec<Role> = sqlx::query_as("SELECT * FROM roles WHERE id = $1")
            .bind(&id)
            .fetch_all(&self.pool)
            .await?;

        if rows.len() == 0 {
            return Err(AppError::RoleNotFound)
        }

        Ok(rows[0].clone())
    }

    pub async fn list_roles(&self) -> AppResult<Roles> {
        let rows: Vec<Role> = sqlx::query_as("SELECT * FROM roles")
            .fetch_all(&self.pool)
            .await?;
        Ok(Roles { roles: rows })
    }

    pub async fn create_role(&self, role: Role) -> AppResult<Role> {
        let row: Role = sqlx::query_as("INSERT INTO roles (name) VALUES ($1) RETURNING *")
            .bind(&role.name)
            .fetch_one(&self.pool)
            .await?;
        Ok(row)
    }

    pub async fn delete_role(&self, id: i64) -> AppResult<()> {
        sqlx::query("DELETE FROM roles WHERE id = $1")
            .bind(&id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_role(&self, role: Role) -> AppResult<Role> {
        let row: Role = sqlx::query_as("UPDATE roles SET name = $1 WHERE id = $2 RETURNING *")
            .bind(&role.name)
            .bind(&role.id)
            .fetch_one(&self.pool)
            .await?;
        Ok(row)
    }
}
