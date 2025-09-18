use devasign_task_escrow::{TaskEscrowContract, TaskEscrowContractClient};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{token, Address, Env};
use token::{StellarAssetClient, TokenClient};

/// Test environment setup utility
pub fn create_test_env() -> (
    Env,
    Address,
    Address,
    StellarAssetClient<'static>,
    TokenClient<'static>,
    Address,
    TaskEscrowContractClient<'static>,
) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    let usdc_address = sac.address();
    let usdc_token = StellarAssetClient::new(&env, &usdc_address);
    let usdc_token_client = TokenClient::new(&env, &usdc_address);
    let contract_id = env.register(TaskEscrowContract, ());
    let client = TaskEscrowContractClient::new(&env, &contract_id);

    (
        env,
        admin,
        usdc_address,
        usdc_token,
        usdc_token_client,
        contract_id,
        client,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_works() {
        let (
            _env, 
            _admin, 
            _usdc_address,
            _usdc_token, 
            _usdc_token_client, 
            _contract_id, 
            _client
        ) = create_test_env();
        // Basic test to ensure setup works
        assert!(true);
    }
}
