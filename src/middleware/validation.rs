use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::{ready, Ready, LocalBoxFuture};

pub struct ValidateRequest<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> Default for ValidateRequest<T> {
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for ValidateRequest<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static + Clone,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ValidateRequestMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ValidateRequestMiddleware {
            service,
            _marker: std::marker::PhantomData,
        }))
    }
}

pub struct ValidateRequestMiddleware<S> {
    service: S,
    _marker: std::marker::PhantomData<S>,
}

impl<S, B> Service<ServiceRequest> for ValidateRequestMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static + Clone,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        Box::pin(async move {
            service.call(req).await
        })
    }
}
