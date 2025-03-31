use async_graphql::{http::GraphiQLSource, EmptyMutation, EmptySubscription, Object, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{response::{Html, IntoResponse}, routing::get, Extension, Router};

#[tokio::main]
async fn main() {
    
    let router = Router::new()
    .route("/graphql", get(graphql_playground).post(graphql_handler));
    
    let listener = tokio::net::TcpListener::bind("localhost:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

async fn graphql_playground() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

// Type alias for our GraphQL schema
type ApiSchema = Schema<Query, EmptyMutation, EmptySubscription>;

// Handler for GraphQL POST requests with authentication
async fn graphql_handler(
    schema: Extension<ApiSchema>,
    req: GraphQLRequest,
) -> Result<GraphQLResponse, ()> {
    let req_builder = req.into_inner();

    // Execute the GraphQL request
    let response = schema.execute(req_builder).await;

    Ok(response.into())
}


struct Query;

#[Object]
impl Query {
    async fn hello(&self) -> String {
        "hello world".to_string()
    }
}