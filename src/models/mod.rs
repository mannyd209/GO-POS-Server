use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};
use std::fmt;
use validator::Validate;
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Staff {
    pub staff_id: String,
    #[validate(length(min = 4, max = 4, message = "PIN must be exactly 4 characters"))]
    #[validate(regex(path = "regex::Regex::new(r\"^[0-9]{4}$\").unwrap()", message = "PIN must be 4 digits"))]
    pub pin: String,
    #[validate(length(min = 1, message = "First name cannot be empty"))]
    pub first_name: String,
    #[validate(length(min = 1, message = "Last name cannot be empty"))]
    pub last_name: String,
    #[validate(range(min = 0.0, message = "Hourly wage must be non-negative"))]
    pub hourly_wage: f64,
    pub is_admin: bool,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Category {
    pub category_id: String,
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: String,
    #[validate(range(min = 0, message = "Sort order must be non-negative"))]
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Item {
    pub item_id: String,
    pub category_id: String,
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: String,
    #[validate(range(min = 0.0, message = "Regular price must be non-negative"))]
    pub regular_price: f64,
    #[validate(range(min = 0.0, message = "Event price must be non-negative"))]
    pub event_price: f64,
    #[validate(range(min = 0, message = "Sort order must be non-negative"))]
    pub sort_order: i32,
    pub available: bool,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Modifier {
    pub modifier_id: String,
    pub item_id: String,
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: String,
    pub single_selection: bool,
    #[validate(range(min = 0, message = "Sort order must be non-negative"))]
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Option {
    pub option_id: String,
    pub modifier_id: String,
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: String,
    #[validate(range(min = 0.0, message = "Price must be non-negative"))]
    pub price: f64,
    pub available: bool,
    #[validate(range(min = 0, message = "Sort order must be non-negative"))]
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Discount {
    pub discount_id: String,
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: String,
    pub is_percentage: bool,
    #[validate(custom = "validate_discount_amount")]
    pub amount: f64,
    pub available: bool,
    #[validate(range(min = 0, message = "Sort order must be non-negative"))]
    pub sort_order: i32,
}

fn validate_discount_amount(amount: f64) -> Result<(), validator::ValidationError> {
    if amount < 0.0 {
        return Err(validator::ValidationError::new("Amount must be non-negative"));
    }
    
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Validation error: {0}")]
    ValidationError(#[from] validator::ValidationErrors),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            ApiError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ApiError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            ApiError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            ApiError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
        }
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let (status, message) = match self {
            ApiError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
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

impl From<r2d2::Error> for ApiError {
    fn from(err: r2d2::Error) -> Self {
        ApiError::DatabaseError(err.to_string())
    }
}

impl From<rusqlite::Error> for ApiError {
    fn from(err: rusqlite::Error) -> Self {
        ApiError::DatabaseError(err.to_string())
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

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AuthRequest {
    #[validate(length(min = 4))]
    pub pin: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub staff_id: String,
    pub is_admin: bool,
}

pub trait ValidatedJson<T>: Sized {
    fn validate(self) -> Result<T, ApiError>;
}

impl<T: Validate> ValidatedJson<T> for actix_web::web::Json<T> {
    fn validate(self) -> Result<T, ApiError> {
        let inner = self.into_inner();
        inner.validate()?;
        Ok(inner)
    }
}
