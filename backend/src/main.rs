use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema, http::GraphiQLSource};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    Extension, Router,
    response::{Html, IntoResponse},
    routing::get,
};
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    // Create the Schema
    let schema = Schema::build(Query::default(), EmptyMutation, EmptySubscription).finish();

    let router = Router::new()
        .route("/graphql", get(graphql_playground).post(graphql_handler))
        // Add the schema as an extension
        .layer(Extension(schema))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_headers([
                    axum::http::header::CONTENT_TYPE,
                    axum::http::header::AUTHORIZATION,
                ])
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::OPTIONS,
                ]),
        );

    println!("GraphQL playground available at: http://localhost:8000/graphql");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

async fn graphql_playground() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

// Type alias for our GraphQL schema
type ApiSchema = Schema<Query, EmptyMutation, EmptySubscription>;

// Handler for GraphQL POST requests
async fn graphql_handler(
    schema: Extension<ApiSchema>,
    req: GraphQLRequest,
) -> Result<GraphQLResponse, ()> {
    let req_builder = req.into_inner();
    let response = schema.execute(req_builder).await;
    Ok(response.into())
}

#[derive(Default)]
struct Query;

#[Object]
impl Query {
    async fn hello(&self) -> String {
        "hello world".to_string()
    }
}
