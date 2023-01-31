#![feature(is_some_and)]
#![feature(async_closure)]
#![feature(trait_alias)]

mod app;
mod router;

use std::{convert::Infallible, net::SocketAddr, collections::HashMap};

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server, body,
};
use regex::{Regex, RegexSet};
use serde::Serialize;

use app::{App, AppError, Role, Roles};

#[derive(Clone)]
enum ErrorType {
    ClientError,
    ServerError,
}

#[derive(Clone, Serialize)]
struct Error<'a> {
    #[serde(skip)]
    error_type: ErrorType,
    code: i32,
    reason: &'a str,
    /// The description of `Error`. If it is `None`, it will use the field
    /// `reason` as the itself when we try to turn it into response.
    description: Option<String>,
}

impl<'a> Error<'a> {
    const fn new_client_error(code: i32, reason: &'a str) -> Self {
        Error {
            error_type: ErrorType::ClientError,
            code,
            reason,
            description: None,
        }
    }

    const fn new_server_error(code: i32, reason: &'a str) -> Self {
        Error {
            error_type: ErrorType::ServerError,
            code,
            reason,
            description: None,
        }
    }

    fn with_description(&self, desc: &str) -> Self {
        let mut result = self.clone();
        result.description = Some(desc.to_string());
        result
    }
}

const INTERNAL: Error = Error::new_server_error(100001, "internal");

const ROLE_NOT_FOUND: Error = Error::new_client_error(200001, "role not found");
const ROLE_NAME_ALREADY_EXISTS: Error =
    Error::new_client_error(200002, "role name is already exists");

const INVALID_ARGUMENT: Error = Error::new_client_error(30001, "invalid argument");
const NOT_FOUND: Error = Error::new_client_error(30002, "not found");
const FAILED_TO_READ_REQUEST: Error = Error::new_client_error(30003, "failed to read request");
const FAILED_TO_DECODE: Error = Error::new_client_error(30004, "failed to decode");

// #[rocket::async_trait]
// impl<'r> Responder<'r, 'static> for AppError {
//     fn respond_to(self, request: &'r Request<'_>) -> response::Result<'static> {
//         let error_description = self.to_string();
//         let error_description = error_description.as_str();
//
//         let error = match self {
//             AppError::RoleNotFound => ROLE_NOT_FOUND,
//             AppError::RoleNameAlreadyExists => ROLE_NAME_ALREADY_EXISTS,
//             _ => INTERNAL,
//         }.with_description(error_description);
//         return error.respond_to(request)
//     }
// }
//
// #[rocket::async_trait]
// impl<'r, 'e> Responder<'r, 'static> for Error<'e> {
//     fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
//         let description = self.description;
//         let description = if description == None {
//             self.reason.to_string()
//         } else {
//             description.unwrap()
//         };
//         let body = json!({
//             "error": { "code": self.code, "reason": self.reason, "description": description },
//         })
//         .to_string();
//         Response::build()
//             .status(self.status)
//             .header(ContentType::JSON)
//             .sized_body(body.len(), Cursor::new(body))
//             .ok()
//     }
// }
//
// #[catch(422)]
// fn unprocessable_entity(req: &Request<'_>, outcome: &Outcome) -> Error<'static> {
//     println!("{:#?}", req);
//     INVALID_ARGUMENT.with_description("unprocessable entity")
// }
//
// #[catch(default)]
// fn all_catcher() -> Error<'static> {
//     INTERNAL.with_description("internal error")
// }
//
// type AppResult<T> = Result<Json<T>, AppError>;
//
#[derive(Serialize)]
struct Empty {}
//
// #[get("/roles/<id>")]
// async fn get_role(app: &State<App>, id: i32) -> AppResult<app::Role> {
// }
//
// #[get("/roles")]
// async fn list_roles(app: &State<App>) -> AppResult<app::Roles> {
//     Ok(Json(app.list_roles().await?))
// }
//
// #[post("/roles", format = "json", data = "<role>")]
// async fn create_role(app: &State<App>, role: Json<app::Role>) -> AppResult<app::Role> {
//     Ok(Json(app.create_role(role.into_inner()).await?))
// }
//
// #[delete("/roles/<id>")]
// async fn delete_role(app: &State<App>, id: i64) -> AppResult<Empty> {
//     app.delete_role(id).await?;
//     Ok(Json(Empty {}))
// }
//
// #[put("/roles/<id>", format = "json", data = "<role>")]
// async fn update_role(app: &State<App>, id: i32, role: Json<app::Role>) -> AppResult<app::Role> {
//     Ok(Json(
//         app.update_role(app::Role {
//             id,
//             ..role.into_inner()
//         })
//         .await?,
//     ))
// }
//
// #[post("/users", format = "json", data = "<user>")]
// async fn create_user(app: &State<App>, user: Json<app::User>) -> AppResult<app::User> {
//     Ok(Json(app.create_user(user.into_inner()).await?))
// }
//
// #[launch]
// async fn rocket() -> _ {
//     rocket::build()
//         .mount(
//             "/api/v1/",
//             routes![
//                 get_role,
//                 create_role,
//                 delete_role,
//                 update_role,
//                 list_roles,
//                 create_user
//             ],
//         )
//         .register("/api/v1", catchers![unprocessable_entity, all_catcher])
//         .manage(app)
// }

enum HTTPResult<'a> {
    Ok(Box<dyn ToJSON>),
    Error(Error<'a>),
}

trait ToJSON {
    fn to_json(&self) -> String;
}

impl ToJSON for Role {
    fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

impl ToJSON for Roles {
    fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

impl ToJSON for Empty {
    fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

impl AppError {
    fn to_http_result(&self) -> HTTPResult<'static> {
        match self {
            Self::Sqlx(..) | Self::Migrate(..) | Self::ConnectPostgres(..) => {
                HTTPResult::Error(INTERNAL.with_description(&self.to_string()))
            }
            Self::RoleNotFound => {
                HTTPResult::Error(ROLE_NOT_FOUND.with_description(&self.to_string()))
            }
            Self::RoleNameAlreadyExists => {
                HTTPResult::Error(ROLE_NAME_ALREADY_EXISTS.with_description(&self.to_string()))
            }
        }
    }
}

impl<'a> HTTPResult<'a> {
    fn to_response(&self) -> Response<Body> {
        let builder = Response::builder().header("Content-Type", "application/json");
        let builder = match self {
            HTTPResult::Ok(..) => builder.status(200),
            HTTPResult::Error(err) => match err.error_type {
                ErrorType::ClientError => builder.status(400),
                ErrorType::ServerError => builder.status(500),
            }
        };
        let body: Body = match self {
            HTTPResult::Ok(body) => body.to_json().into(),
            HTTPResult::Error(error) => serde_json::to_string(error).unwrap().into(),
        };
        builder.body(body).unwrap()
    }
}

async fn get_role(app: App, _req: Request<Body>, param_map: HashMap<String, String>) -> HTTPResult<'static> {
    let role_id = match param_map["id"].parse::<i32>() {
        Ok(role_id) => role_id,
        Err(err) => return HTTPResult::Error(INVALID_ARGUMENT.with_description(&err.to_string())),
    };
    let role = match app.get_role(role_id).await {
        Ok(role) => role,
        Err(err) => return err.to_http_result(),
    };
    HTTPResult::Ok(Box::new(role))
}

async fn list_roles(app: App, _req: Request<Body>, _param_map: HashMap<String, String>) -> HTTPResult<'static> {
    let roles = match app.list_roles().await {
        Ok(roles) => roles,
        Err(err) => return err.to_http_result(),
    };
    HTTPResult::Ok(Box::new(roles))
}

async fn create_role(app: App, mut req: Request<Body>, _param_map: HashMap<String, String>) -> HTTPResult<'static> {
    let body = match body::to_bytes(req.body_mut()).await {
        Ok(bytes) => bytes,
        Err(err) => return HTTPResult::Error(FAILED_TO_READ_REQUEST.with_description(
            &err.to_string(),
        )),
    };
    let role = match serde_json::from_slice(&body) {
        Ok(role) => role,
        Err(err) => return HTTPResult::Error(FAILED_TO_DECODE.with_description(
            &err.to_string(),
        )),
    };
    let role = match app.create_role(role).await {
        Ok(role) => role,
        Err(err) => return err.to_http_result(),
    };
    HTTPResult::Ok(Box::new(role))
}

async fn delete_role<'a>(app: App, _req: Request<Body>, param_map: HashMap<String, String>) -> HTTPResult<'a> {
    let role_id = match param_map["id"].parse::<i64>() {
        Ok(role_id) => role_id,
        Err(err) => return HTTPResult::Error(INVALID_ARGUMENT.with_description(&err.to_string())),
    };
    if let Err(err) = app.delete_role(role_id).await {
        return err.to_http_result()
    };
    HTTPResult::Ok(Box::new(Empty{}))
}

async fn update_role<'a>(app: App, mut req: Request<Body>, param_map: HashMap<String, String>) -> HTTPResult<'a> {
    let role_id = match param_map["id"].parse::<i32>() {
        Ok(role_id) => role_id,
        Err(err) => return HTTPResult::Error(INVALID_ARGUMENT.with_description(&err.to_string())),
    };
    let body = match body::to_bytes(req.body_mut()).await {
        Ok(bytes) => bytes,
        Err(err) => return HTTPResult::Error(FAILED_TO_READ_REQUEST.with_description(
            &err.to_string(),
        )),
    };
    let mut role: Role = match serde_json::from_slice(&body) {
        Ok(role) => role,
        Err(err) => return HTTPResult::Error(FAILED_TO_DECODE.with_description(
            &err.to_string(),
        )),
    };
    role.id = role_id;
    let role = match app.update_role(role).await {
        Ok(role) => role,
        Err(err) => return err.to_http_result(),
    };
    HTTPResult::Ok(Box::new(role))
}

async fn router(app: App, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    use router::RouterBuilder;

    let router = RouterBuilder::new(app)
        .register("GET", "/api/v1/roles", list_roles)
        .register("GET", "/api/v1/roles/<id>", get_role)
        .register("POST", "/api/v1/roles", create_role)
        .register("PUT", "/api/v1/roles/<id>", update_role)
        .register("DELETE", "/api/v1/roles/<id>", delete_role)
        .build();

    router.route(req.method(), &req.uri().path().to_owned());

    panic!("yes")
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    let app = match App::new().await {
        Ok(a) => a,
        Err(e) => panic!("create app: {}", e),
    };

    let make_svc = make_service_fn(move |_conn| {
        let app = app.clone();
        let service = service_fn(move |req: Request<Body>| {
            let app = app.clone();
            router(app.clone(), req)
        });
        async { Ok::<_, Infallible>(service) }
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        panic!("run server: {}", e);
    }
}
