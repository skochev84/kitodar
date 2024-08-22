use crate::repository::kub::KubeRepository;
use actix_web::{
    delete,
    error::ResponseError,
    get,
    http::{header::ContentType, StatusCode},
    patch, post,
    web::Data,
    web::Json,
    web::Path,
    HttpResponse,
};
use common::model::user::User;
use common::model::user::VmsVersion;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Deserialize)]
pub struct CreateUserRequest {
    user_name: String,
    vms_version: String,
}

#[derive(Debug, Display)]
pub enum UserError {
    UserNotFound,
    UserUpgradeFailure,
    UserCreationFailure,
    BadUserRequest,
}

impl ResponseError for UserError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            UserError::UserNotFound => StatusCode::NOT_FOUND,
            UserError::UserUpgradeFailure => StatusCode::FAILED_DEPENDENCY,
            UserError::UserCreationFailure => StatusCode::FAILED_DEPENDENCY,
            UserError::BadUserRequest => StatusCode::BAD_REQUEST,
        }
    }
}

#[get("/user")]
pub async fn get_users(kube_repo: Data<KubeRepository>) -> Result<Json<Vec<User>>, UserError> {
    let user = kube_repo.get_users().await;

    match user {
        Some(user) => Ok(Json(user)),
        None => Err(UserError::UserNotFound),
    }
}

#[get("/user/{user_global_id}")]
pub async fn get_user(
    kube_repo: Data<KubeRepository>,
    user_global_id: Path<String>,
) -> Result<Json<User>, UserError> {
    let user = kube_repo.get_user(user_global_id.to_string()).await;

    match user {
        Some(user) => Ok(Json(user)),
        None => Err(UserError::UserNotFound),
    }
}

#[post("/user")]
pub async fn create_user(
    kube_repo: Data<KubeRepository>,
    request: Json<CreateUserRequest>,
) -> Result<Json<User>, UserError> {
    let vms_version = match VmsVersion::from_str(&request.vms_version) {
        Ok(vms_version) => vms_version,
        Err(_) => return Err(UserError::UserCreationFailure),
    };
    let user = User::new(request.user_name.clone(), vms_version);

    match kube_repo.create_user(user).await {
        Ok(user) => Ok(Json(user)),
        Err(_) => Err(UserError::UserCreationFailure),
    }
}

#[patch("/user/{user_global_id}")]
pub async fn upgrade_user(
    kube_repo: Data<KubeRepository>,
    user_global_id: Path<String>,
) -> Result<Json<User>, UserError> {
    let user = kube_repo.upgrade_user(user_global_id.to_string()).await;

    match user {
        Some(user) => Ok(Json(user)),
        None => Err(UserError::UserNotFound),
    }
}

#[delete("/user/{user_global_id}")]
pub async fn delete_user(
    kube_repo: Data<KubeRepository>,
    user_global_id: Path<String>,
) -> Result<Json<User>, UserError> {
    let user = kube_repo.delete_user(user_global_id.to_string()).await;

    match user {
        Some(user) => Ok(Json(user)),
        None => Err(UserError::UserNotFound),
    }
}
