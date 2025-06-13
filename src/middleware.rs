use crate::claims::Claims;
use crate::error::AuthError;
use crate::jwt::JwtService;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
};

/// Middleware for JWT authentication
pub struct JwtAuth;

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct JwtAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            // Extract authorization header
            let auth_header = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok());

            match auth_header {
                Some(header) => {
                    // Extract and validate token
                    match JwtService::extract_token_from_header(header) {
                        Ok(token) => {
                            match JwtService::validate_token(token) {
                                Ok(claims) => {
                                    // Check if token is expired
                                    if claims.is_expired() {
                                        return Err(AuthError::TokenExpired.into());
                                    }
                                    
                                    // Insert claims into request extensions
                                    req.extensions_mut().insert(claims);
                                    
                                    // Continue with the request
                                    service.call(req).await
                                }
                                Err(e) => Err(e.into()),
                            }
                        }
                        Err(e) => Err(e.into()),
                    }
                }
                None => Err(AuthError::MissingToken.into()),
            }
        })
    }
}

/// Extract JWT claims from request extensions
/// This is used in route handlers to get the authenticated user information
pub fn extract_claims(req: &ServiceRequest) -> Result<Claims, AuthError> {
    req.extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AuthError::Unauthorized)
}

/// Actix-web extractor for JWT claims
/// Usage in handlers: fn handler(claims: JwtClaims) -> impl Responder
use actix_web::{FromRequest, HttpRequest};
use std::future::Future;
use std::pin::Pin;

pub struct JwtClaims(pub Claims);

impl FromRequest for JwtClaims {
    type Error = AuthError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let claims = req.extensions().get::<Claims>().cloned();
        
        Box::pin(async move {
            match claims {
                Some(claims) => Ok(JwtClaims(claims)),
                None => Err(AuthError::Unauthorized),
            }
        })
    }
}

impl std::ops::Deref for JwtClaims {
    type Target = Claims;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
} 