use crate::models::HelloWorld;
use async_graphql::Object;

pub struct HelloQuery;

#[Object]
impl HelloQuery {
    async fn hello_world(&self) -> HelloWorld {
        HelloWorld {
            message: "Hello, World!".to_string(),
        }
    }

    async fn greet(&self, name: Option<String>) -> String {
        match name {
            Some(name) => format!("Hello, {}!", name),
            None => "Hello, anonymous!".to_string(),
        }
    }
}
