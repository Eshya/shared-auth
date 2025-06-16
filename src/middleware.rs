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
pub struct JwtAuth {
    jwt_service: JwtService,
}

impl JwtAuth {
    pub fn new(jwt_service: JwtService) -> Self {
        Self { jwt_service }
    }
}

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = JwtAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthMiddleware {
            service: Rc::new(service),
            jwt_service: self.jwt_service.clone(),
        }))
    }
}

pub struct JwtAuthMiddleware<S> {
    service: Rc<S>,
    jwt_service: JwtService,
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
        let service = self.service.clone();
        let jwt_service = self.jwt_service.clone();

        Box::pin(async move {
            // Extract Authorization header
            let auth_header = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok());

            if let Some(auth_header) = auth_header {
                if let Some(token) = JwtService::extract_token_from_header(auth_header) {
                    // Validate token (includes blacklist check)
                    match jwt_service.validate_token(token).await {
                        Ok(claims) => {
                            // Insert claims into request extensions
                            req.extensions_mut().insert(claims);
                            service.call(req).await
                        }
                        Err(_auth_error) => {
                            // Return authentication error
                            Err(actix_web::error::ErrorUnauthorized("Invalid token"))
                        }
                    }
                } else {
                    // Invalid Authorization header format
                    Err(actix_web::error::ErrorUnauthorized("Invalid authorization header"))
                }
            } else {
                // Missing Authorization header
                Err(actix_web::error::ErrorUnauthorized("Missing authorization header"))
            }
        })
    }
}

/// Extract JWT claims from request extensions
/// This is used in route handlers to get the authenticated user information
pub fn extract_claims(req: &ServiceRequest) -> Option<Claims> {
    req.extensions().get::<Claims>().cloned()
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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, HttpResponse};

    async fn test_handler(claims: crate::claims::JwtClaims) -> HttpResponse {
        HttpResponse::Ok().json(serde_json::json!({
            "user_id": claims.sub,
            "email": claims.email
        }))
    }

    #[actix_web::test]
    async fn test_jwt_middleware_missing_token() {
        let jwt_service = JwtService::new("test-secret");
        
        let app = test::init_service(
            App::new()
                .service(
                    web::scope("/protected")
                        .wrap(JwtAuth::new(jwt_service))
                        .route("/test", web::get().to(test_handler))
                )
        ).await;

        let req = test::TestRequest::get()
            .uri("/protected/test")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);
    }

    #[actix_web::test]
    async fn test_jwt_middleware_valid_token() {
        let jwt_service = JwtService::new("test-secret");
        
        // Generate a test token
        let token = jwt_service.generate_token(
            1,
            "test@example.com".to_string(),
            Some(1),
            vec!["user".to_string()],
            Some(1),
        ).unwrap();

        let app = test::init_service(
            App::new()
                .service(
                    web::scope("/protected")
                        .wrap(JwtAuth::new(jwt_service))
                        .route("/test", web::get().to(test_handler))
                )
        ).await;

        let req = test::TestRequest::get()
            .uri("/protected/test")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }
} 