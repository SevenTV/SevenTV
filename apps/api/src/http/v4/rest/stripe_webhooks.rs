use axum::{Json, http::StatusCode};

pub async fn handle(Json(event): Json<stripe::Event>) -> StatusCode {
    StatusCode::OK
}
