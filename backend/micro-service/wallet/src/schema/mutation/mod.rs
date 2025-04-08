pub mod wallet;

use async_graphql::MergedObject;

#[derive(MergedObject)]
pub struct Mutation(wallet::WalletMutation);

pub fn create_mutation() -> Mutation {
    Mutation(wallet::WalletMutation)
}
