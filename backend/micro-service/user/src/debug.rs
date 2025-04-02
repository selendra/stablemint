use axum::{body::Body, http::Request, middleware::Next, response::Response};
use std::sync::Arc;
use tracing::info;

use app_authentication::{AuthService, JwtService}; // Import from auth crate

// Middleware to debug available extensions
pub async fn debug_extensions(req: Request<Body>, next: Next) -> Response {
    // Log the path
    let path = req.uri().path().to_owned();

    // Check if we're hitting the GraphQL endpoint
    if path == "/graphql" {
        // Check for specific extension types
        info!("Checking extensions for /graphql endpoint");

        if req.extensions().get::<Arc<AuthService>>().is_some() {
            info!("✅ AuthService is available in extensions");
        } else {
            info!("❌ AuthService is NOT available in extensions");
        }

        if req.extensions().get::<Arc<JwtService>>().is_some() {
            info!("✅ JwtService is available in extensions");
        } else {
            info!("❌ JwtService is NOT available in extensions");
        }

        if req.extensions().get::<crate::schema::ApiSchema>().is_some() {
            info!("✅ ApiSchema is available in extensions");
        } else {
            info!("❌ ApiSchema is NOT available in extensions");
        }
    }

    // Process the request
    next.run(req).await
}
