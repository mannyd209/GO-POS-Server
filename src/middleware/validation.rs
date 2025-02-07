use std::future::{ready, Ready};
use actix_web::{dev::Payload, FromRequest, HttpRequest};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::models::ApiError;

#[derive(Debug)]
pub struct ValidatedJson<T>(pub T);

impl<T> ValidatedJson<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> FromRequest for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
{
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let json_result = match actix_web::web::Json::<T>::from_request(req, payload).into_inner() {
            Ok(json) => json,
            Err(e) => return ready(Err(ApiError::BadRequest(format!("JSON parsing error: {}", e)))),
        };

        let data = json_result.0;
        ready(match data.validate() {
            Ok(_) => Ok(ValidatedJson(data)),
            Err(e) => Err(ApiError::BadRequest(format!("Validation error: {}", e))),
        })
    }
}
