use std::future::{ready, Ready};
use std::rc::Rc;

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    http::header,
    Error,
};
use futures_util::future::LocalBoxFuture;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub struct AdminAuth;

impl AdminAuth {
    pub fn new() -> Self {
        AdminAuth
    }
}

impl<S, B> Transform<S, ServiceRequest> for AdminAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AdminAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AdminAuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct AdminAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AdminAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        Box::pin(async move {
            let pool = req.app_data::<actix_web::web::Data<Pool<SqliteConnectionManager>>>()
                .ok_or_else(|| ErrorUnauthorized("Database connection not available"))?;

            // Get the PIN from the Authorization header
            let auth_header = req.headers()
                .get(header::AUTHORIZATION)
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.strip_prefix("Bearer "));

            if let Some(pin) = auth_header {
                let conn = pool.get().map_err(|e| ErrorUnauthorized(e.to_string()))?;
                
                // Check if the PIN belongs to an admin
                let is_admin: bool = conn
                    .query_row(
                        "SELECT is_admin FROM staff WHERE pin = ?1",
                        [pin],
                        |row| row.get::<_, i32>(0),
                    )
                    .map(|val| val != 0)
                    .unwrap_or(false);

                if is_admin {
                    return service.call(req).await;
                }
            }

            Err(ErrorUnauthorized("Admin access required"))
        })
    }
}

// Validation middleware for request payloads
#[derive(Debug, Clone)]
pub struct ValidatedRequest<T>(pub T);

impl<T> ValidatedRequest<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}
