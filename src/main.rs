#[macro_use]
extern crate rocket;

mod app;

use std::io::Cursor;

use rocket::{
    http::{ContentType, Status},
    response::{self, Responder},
    serde::{
        json::{serde_json::json, Json},
        Deserialize, Serialize,
    },
    Request, Response, State,
};

use app::{App, AppError};

const CODE_INTERNAL: u32 = 10001;

const CODE_ROLE_NOT_FOUND: u32 = 20001;

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for AppError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let (code, status) = match self {
            AppError::RoleNotFound => (CODE_ROLE_NOT_FOUND, Status::BadRequest),
            _ => (CODE_INTERNAL, Status::InternalServerError),
        };
        let body = json!({
            "error": { "code": code, "message": self.to_string() }
        })
        .to_string();
        Response::build()
            .status(status)
            .header(ContentType::JSON)
            .sized_body(body.len(), Cursor::new(body))
            .ok()
    }
}

type AppResult<T> = Result<Json<T>, AppError>;

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Empty {}

#[get("/roles/<id>")]
async fn get_role(app: &State<App>, id: i32) -> AppResult<app::Role> {
    Ok(Json(app.get_role(id).await?))
}

#[get("/roles")]
async fn list_roles(app: &State<App>) -> AppResult<app::Roles> {
    Ok(Json(app.list_roles().await?))
}

#[post("/roles", format = "json", data = "<role>")]
async fn create_role(app: &State<App>, role: Json<app::Role>) -> AppResult<app::Role> {
    Ok(Json(app.create_role(role.into_inner()).await?))
}

#[delete("/roles/<id>")]
async fn delete_role(app: &State<App>, id: i64) -> AppResult<Empty> {
    app.delete_role(id).await?;
    Ok(Json(Empty {}))
}

#[put("/roles/<id>", format = "json", data = "<role>")]
async fn update_role(
    app: &State<App>,
    id: i32,
    role: Json<app::Role>,
) -> AppResult<app::Role> {
    Ok(Json(
        app.update_role(app::Role {
            id,
            ..role.into_inner()
        })
        .await?,
    ))
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
