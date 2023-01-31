use std::env;

use serde::{Deserialize, Serialize};
use sqlx::{migrate::MigrateError, postgres::{PgPoolOptions}, Pool, Postgres};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Role {
    /// (Output only) The role's ID. Default value is `unknown_id()`.
    #[serde(default = "unknown_id")]
    pub id: i32,

    /// The role's name.
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Roles {
    /// The list of roles.
    pub roles: Vec<Role>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct User {
    /// [Output only] The user's ID. Default value is `unknown_id()`.
    #[serde(default = "unknown_id")]
    pub id: i32,

    /// The user's name.
    pub name: String,

    /// (Input only) The user's password.
    pub password: String,

    /// (Hidden by default) The user's hashed password.
    #[serde(skip)]
    pub hashed_password: String,
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

    #[error("role name is already exists")]
    RoleNameAlreadyExists,
}

#[derive(Clone)]
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

        Ok(App { pool: pool })
    }

    /// Get a role by ID. If the role does not exist, raise the error
    /// `RoleNotFound`.
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

    /// List all roles.
    pub async fn list_roles(&self) -> AppResult<Roles> {
        let rows: Vec<Role> = sqlx::query_as("SELECT * FROM roles")
            .fetch_all(&self.pool)
            .await?;
        Ok(Roles { roles: rows })
    }

    /// Create a new role. If there is already a role with the same name, raise
    /// error `RoleNameAlreadyExists`.
    pub async fn create_role(&self, role: Role) -> AppResult<Role> {
        let role_result: Result<Role, _> = sqlx::query_as("INSERT INTO roles (name) VALUES ($1) RETURNING *")
            .bind(&role.name)
            .fetch_one(&self.pool)
            .await;

        let is_role_name_already_exists = async || -> AppResult<bool> {
            let (count, ): (i64, ) = sqlx::query_as("SELECT COUNT(*) FROM roles WHERE name = $1")
                .bind(role.name)
                .fetch_one(&self.pool)
                .await?;
            Ok(count > 0)
        };

        if let Err(err) = role_result {
            // Because the role table should have a unique index on the field
            // `name`, if the err is a duplicate key error, then it may already
            // have a role with the same name in the database.
            if is_duplicate_key_error(&err) {
                if is_role_name_already_exists().await? {
                    return Err(AppError::RoleNameAlreadyExists)
                }
            }
            return Err(err.into())
        }
        Ok(role_result.unwrap())
    }

    /// Delete a role by ID. If the roles is not found by ID, raise no error
    /// also.
    pub async fn delete_role(&self, id: i64) -> AppResult<()> {
        sqlx::query("DELETE FROM roles WHERE id = $1")
            .bind(&id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Update a role by ID.
    pub async fn update_role(&self, role: Role) -> AppResult<Role> {
        let row: Role = sqlx::query_as("UPDATE roles SET name = $1 WHERE id = $2 RETURNING *")
            .bind(&role.name)
            .bind(&role.id)
            .fetch_one(&self.pool)
            .await?;
        Ok(row)
    }

    /// Create a user.
    pub async fn create_user(&self, user: User) -> AppResult<User> {
        return Ok(user)
    }
}

fn is_duplicate_key_error(err: &sqlx::Error) -> bool {
    if let Some(err) = err.as_database_error() {
        if err.code().is_some_and(|c| c == "23505") {
            return true;
        }
    }
    false
}
