use cosmwasm_schema::write_api;
use lido_staking_base::msg::provider_proposals::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg
    }
}
