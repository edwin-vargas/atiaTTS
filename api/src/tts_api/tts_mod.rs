use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SignupRequest {
    pub user_name: String,
    pub user_email: String,
    pub user_pass: String,
}

#[derive(Deserialize)]
pub struct SigninRequest {
    pub user_email: String,
    pub user_pass: String,
}

#[derive(Deserialize)]
pub struct PlanRequest {
    pub user_id: String,
    pub plan: i32,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub user_id: String,
    pub user_name: String,
    pub user_email: String,
}

#[derive(Serialize)]
pub struct SignupResponse {
    pub user_id: String,
    pub success: bool,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Serialize)]
pub struct SuccessResponse {
    pub success: bool,
}

#[derive(Deserialize)]
pub struct TtsRequest {
    pub user_id: String,
    pub text: String,
}

#[derive(Serialize)]
pub struct TtsResponse {
    pub success: bool,
    pub message: String,
}
