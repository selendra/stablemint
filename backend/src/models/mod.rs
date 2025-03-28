pub mod user;

use async_graphql::SimpleObject;

#[derive(SimpleObject)]
pub struct HelloWorld {
    pub message: String,
}
