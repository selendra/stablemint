pub mod wallet;

use async_graphql::MergedObject;

#[derive(MergedObject)]
pub struct Query(wallet::WalletQuery);

pub fn create_query() -> Query {
    Query(wallet::WalletQuery)
}
