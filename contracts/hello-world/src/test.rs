#![cfg(test)]
use super::{LigtasLinkContract, LigtasLinkContractClient, DataKey};
use soroban_sdk::{testutils::Address as _, token, Address, Env, Symbol};

fn setup_test_env<'a>() -> (Env, LigtasLinkContractClient, Address, Address, Address, token::Client<'a>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, LigtasLinkContract);
    let client_contract = LigtasLinkContractClient::new(&env, &contract_id);

    let farm_coop = Address::generate(&env);
    let weather_oracle_node = Address::generate(&env);
    let underwriter_pool = Address::generate(&env);
    
    let token_id = env.register_stellar_asset_contract(Address::generate(&env));
    let usdc_admin = token::StellarAssetClient::new(&env, &token_id);
    let usdc_token = token::Client::new(&env, &token_id);

    // Seed the underwriting pool wallet with immediate payout coverage capital reserves
    usdc_admin.mint(&underwriter_pool, &10_000_000);

    (env, client_contract, farm_coop, weather_oracle_node, underwriter_pool, usdc_token)
}

#[test]
fn test_1_happy_path_parametric_insurance_payout_lifecycle() {
    let (env, contract, coop, oracle, underwriter, usdc_token) = setup_test_env();
    let critical_wind_speed_limit = 150_u32; // Typhoon trigger parameter set at 150 km/h
    let insurance_payout_pool = 1_500_000_i128; // $1,500 USDC relief payout limit (6 decimals)

    contract.initialize(&coop, &oracle, &usdc_token.address, &critical_wind_speed_limit, &insurance_payout_pool);

    // Underwriter deposits liability reserves into contract custody structures
    contract.fund_policy_pool(&underwriter);
    assert_eq!(usdc_token.balance(&contract.address), 1_500_000);

    // Weather event breaks parameter target profile -> Oracle signs off on a 165 km/h wind log
    contract.report_weather_metrics(&165_u32);

    // Verify affected community cooperative instantly received the relief payout asset tranche
    assert_eq!(usdc_token.balance(&coop), 1_500_000);
    assert_eq!(usdc_token.balance(&contract.address), 0);
    
    let (active_state, settled_state) = contract.get_policy_state();
    assert_eq!(active_state, false);
    assert_eq!(settled_state, true);
}

#[test]
fn test_2_weather_below_trigger_does_not_fire_claims() {
    let (env, contract, coop, oracle, underwriter, usdc_token) = setup_test_env();
    let critical_wind_speed_limit = 150_u32;
    let insurance_payout_pool = 1_500_000_i128;

    contract.initialize(&coop, &oracle, &usdc_token.address, &critical_wind_speed_limit, &insurance_payout_pool);
    contract.fund_policy_pool(&underwriter);

    // Moderate storm recorded -> Oracle pushes a 110 km/h log entry
    contract.report_weather_metrics(&110_u32);

    // Verify contract states stand unaffected and balance remains securely locked in escrow
    assert_eq!(usdc_token.balance(&coop), 0);
    assert_eq!(usdc_token.balance(&contract.address), 1_500_000);
    
    let (active_state, settled_state) = contract.get_policy_state();
    assert_eq!(active_state, true);
    assert_eq!(settled_state, false);
}

#[test]
#[should_panic(expected = "Disaster relief allocations have already been fully disbursed to this profile")]
fn test_3_edge_case_duplicate_disaster_claims_are_blocked() {
    let (env, contract, coop, oracle, underwriter, usdc_token) = setup_test_env();
    let critical_wind_speed_limit = 120_u32;
    let insurance_payout_pool = 500_000_i128;

    contract.initialize(&coop, &oracle, &usdc_token.address, &critical_wind_speed_limit, &insurance_payout_pool);
    contract.fund_policy_pool(&underwriter);
    
    // Initial storm trigger processes correctly
    contract.report_weather_metrics(&130_u32);
    
    // Secondary oracle reports on the same policy run must fail instantly
    contract.report_weather_metrics(&140_u32);
}

#[test]
#[should_panic(expected = "Parametric climate insurance registry profile already active")]
fn test_4_prevent_double_initialization() {
    let (env, contract, coop, oracle, underwriter, usdc_token) = setup_test_env();
    
    contract.initialize(&coop, &oracle, &usdc_token.address, &150_u32, &100_000_i128);
    
    // Duplicate initializations are blocked by security boundaries
    contract.initialize(&coop, &oracle, &usdc_token.address, &150_u32, &100_000_i128);
}

#[test]
#[should_panic(expected = "Parametric wind speed triggers must be configured above zero values")]
fn test_5_cannot_initialize_with_zero_trigger() {
    let (env, contract, coop, oracle, underwriter, usdc_token) = setup_test_env();
    
    // Initializing configuration boundaries at zero values returns a structural error
    contract.initialize(&coop, &oracle, &usdc_token.address, &0_u32, &100_000_i128);
}