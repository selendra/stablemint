use crate::models::HelloWorld;
use async_graphql::Object;

pub struct Query;

#[Object]
impl Query {
    pub async fn hello_world(&self) -> HelloWorld {
        HelloWorld {
            message: "Hello, World!".to_string(),
        }
    }

    pub async fn greet(&self, name: Option<String>) -> String {
        match name {
            Some(name) => format!("Hello, {}!", name),
            None => "Hello, anonymous!".to_string(),
        }
    }
}
