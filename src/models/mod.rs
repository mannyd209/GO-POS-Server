use actix_web::{http::StatusCode, HttpResponse, ResponseError, web::Json};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationErrors};

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct Staff {
    #[validate(length(min = 1, max = 50))]
    pub staff_id: String,
    #[validate(length(min = 4))]
    pub pin: String,
    #[validate(length(min = 1, max = 50))]
    pub first_name: String,
    #[validate(length(min = 1, max = 50))]
    pub last_name: String,
    #[validate(range(min = 0.0))]
    pub hourly_wage: f64,
    pub is_admin: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct StaffCredentials {
    #[validate(length(min = 1, max = 50))]
    pub staff_id: String,
    #[validate(length(min = 4))]
    pub pin: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct Category {
    pub category_id: String,
    #[validate(length(min = 1))]
    pub name: String,
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct Item {
    pub item_id: String,
    pub category_id: String,
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(range(min = 0.0))]
    pub regular_price: f64,
    #[validate(range(min = 0.0))]
    pub event_price: f64,
    pub sort_order: i32,
    pub available: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct Modifier {
    pub modifier_id: String,
    pub item_id: String,
    #[validate(length(min = 1))]
    pub name: String,
    pub single_selection: bool,
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct Option {
    pub option_id: String,
    pub modifier_id: String,
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(range(min = 0.0))]
    pub price: f64,
    pub available: bool,
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct Discount {
    pub discount_id: String,
    #[validate(length(min = 1))]
    pub name: String,
    pub is_percentage: bool,
    #[validate(range(min = 0.0))]
    pub amount: f64,
    pub available: bool,
    pub sort_order: i32,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl From<rusqlite::Error> for ApiError {
    fn from(err: rusqlite::Error) -> Self {
        ApiError::DatabaseError(err.to_string())
    }
}

impl From<r2d2::Error> for ApiError {
    fn from(err: r2d2::Error) -> Self {
        ApiError::DatabaseError(err.to_string())
    }
}

impl From<ValidationErrors> for ApiError {
    fn from(errors: ValidationErrors) -> Self {
        ApiError::ValidationError(errors.to_string())
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let (status, message) = match self {
            ApiError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.to_string()),
            ApiError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg.to_string()),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.to_string()),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.to_string()),
        };

        HttpResponse::build(status).json(serde_json::json!({
            "error": message
        }))
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::ValidationError(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WsMessage {
    StaffCreated(Staff),
    StaffUpdated(Staff),
    StaffDeleted(String),
    CategoryCreated(Category),
    CategoryUpdated(Category),
    CategoryDeleted(String),
    ItemCreated(Item),
    ItemUpdated(Item),
    ItemDeleted(String),
    ModifierCreated(Modifier),
    ModifierUpdated(Modifier),
    ModifierDeleted(String),
    OptionCreated(Option),
    OptionUpdated(Option),
    OptionDeleted(String),
    DiscountCreated(Discount),
    DiscountUpdated(Discount),
    DiscountDeleted(String),
}

pub trait ValidatedJson<T> {
    fn validate(self) -> Result<T, ApiError>;
}

impl<T: Validate> ValidatedJson<T> for Json<T> {
    fn validate(self) -> Result<T, ApiError> {
        let data = self.into_inner();
        data.validate()?;
        Ok(data)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct AuthRequest {
    #[validate(length(min = 4))]
    pub pin: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthResponse {
    pub staff_id: String,
    pub is_admin: bool,
}
