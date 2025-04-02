use anyhow::Context;
use async_graphql::{EmptySubscription, Schema, http::GraphiQLSource};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use graphql::{mutation::MutationRoot, query::QueryRoot};
use stablemint_authentication::{AuthUser, JwtAuth, JwtConfig};
use stablemint_error::AppError;
use stablemint_surrealdb::{
    conn::{credentials::{ConnectionManager, DatabaseCredentials, SecureDatabaseConfig}, initialize_db, DatabaseConfig},
    types::{Database, DB_ARC},
};
use std::{sync::Arc, time::Duration};
use tokio::net::TcpListener;

use axum::{
    Extension, Router,
    response::{Html, IntoResponse},
    routing::{get, post},
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

pub mod graphql;

#[derive(Clone)]
struct AppState {
    db: Arc<Database>,
    jwt_auth: JwtAuth,
}

type UserSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;


#[axum::debug_handler]
async fn graphql_handler(
    schema: Extension<UserSchema>,
    auth_user: Option<Extension<AuthUser>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut req = req.into_inner();
    // If user is authenticated, add user info to the GraphQL context
    if let Some(Extension(user)) = auth_user {
        req = req.data(user.clone());
    }
    
    // You can now use state if needed
    // req = req.data(state.db.clone());
    
    schema.execute(req).await.into()
}

// Handler for GraphQL playground UI
pub async fn graphql_playground() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}


#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Set up JWT
    let jwt_config = JwtConfig::from_env()?;
    let jwt_auth = JwtAuth::new(jwt_config);

    let creds = DatabaseCredentials::new_direct("manager_user", "manager_pass")
            .with_rotation_interval(Duration::from_secs(1));

    let db_config = SecureDatabaseConfig::new("memory", creds, "test", "test").with_tls(false);

    let mut conn_manager = ConnectionManager::new(db_config);
    let db = conn_manager.get_connection().await?;

    // Set up GraphQL schema
    let schema = Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        EmptySubscription,
    )
    .data(db.clone())
    .finish();

    // Set up application state
    let app_state = AppState {
        db: db.clone(),
        jwt_auth: jwt_auth.clone(),
    };

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/graphql", post(graphql_handler))
        .route("/graphql", get(graphql_playground))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(Extension(schema))
        .with_state(app_state);

    let address = format!("{}:{}", "0.0.0.0", "3000");
    let listener = TcpListener::bind(&address)
        .await
        .map_err(anyhow::Error::new)?;

    axum::serve(listener, app).await.context("Server error")?;

    Ok(())
}
