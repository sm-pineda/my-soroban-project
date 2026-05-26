#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    PolicyHolder,      // Address of the registered rural beneficiary/cooperative
    WeatherOracle,     // Address of the verified meteorological data node provider
    UsdcToken,         // Address of the USDC stablecoin asset contract
    WindSpeedTrigger,  // Critical parametric trigger threshold measured in km/h (u32)
    PayoutAmount,      // Pre-negotiated insurance relief payout sum in USDC (i128)
    IsPolicyActive,    // Active status track indicator flag (bool)
    IsClaimSettled,    // State parameter tracking payout execution (bool)
}

#[contract]
pub struct LigtasLinkContract;

#[contractimpl]
impl LigtasLinkContract {
    /// Initializes an on-chain parametric disaster micro-insurance contract policy mapping.
    pub fn initialize(env: Env, holder: Address, oracle: Address, usdc_token: Address, trigger_speed: u32, payout: i128) {
        if env.storage().instance().has(&DataKey::PolicyHolder) {
            panic!("Parametric climate insurance registry profile already active");
        }
        if trigger_speed == 0 {
            panic!("Parametric wind speed triggers must be configured above zero values");
        }
        if payout <= 0 {
            panic!("Policy insurance relief payouts must stand above zero values");
        }
        env.storage().instance().set(&DataKey::PolicyHolder, &holder);
        env.storage().instance().set(&DataKey::WeatherOracle, &oracle);
        env.storage().instance().set(&DataKey::UsdcToken, &usdc_token);
        env.storage().instance().set(&DataKey::WindSpeedTrigger, &trigger_speed);
        env.storage().instance().set(&DataKey::PayoutAmount, &payout);
        env.storage().instance().set(&DataKey::IsPolicyActive, &true);
        env.storage().instance().set(&DataKey::IsClaimSettled, &false);
    }

    /// Secures liquidity parameters, allowing liquidity providers or underwriters to fund the potential liability pool.
    pub fn fund_policy_pool(env: Env, funder: Address) {
        funder.require_auth();

        let token_addr: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();
        let payout: i128 = env.storage().instance().get(&DataKey::PayoutAmount).unwrap();
        let contract_address = env.current_contract_address();

        let usdc_client = token::Client::new(&env, &token_addr);

        // Pull full claim reserve requirements directly into the contract instance custody
        usdc_client.transfer(&funder, &contract_address, &payout);
    }

    /// Processes objective meteorological reports; triggers instant payouts if storm limits are breached.
    pub fn report_weather_metrics(env: Env, reported_wind_speed: u32) {
        let oracle: Address = env.storage().instance().get(&DataKey::WeatherOracle).unwrap();
        // Metric data log updates are strictly restricted to the authorized weather oracle node signature
        oracle.require_auth();

        let active: bool = env.storage().instance().get(&DataKey::IsPolicyActive).unwrap_or(false);
        if !active {
            panic!("The coverage window for this parametric insurance framework is closed");
        }

        let settled: bool = env.storage().instance().get(&DataKey::IsClaimSettled).unwrap_or(false);
        if settled {
            panic!("Disaster relief allocations have already been fully disbursed to this profile");
        }

        let trigger_speed: u32 = env.storage().instance().get(&DataKey::WindSpeedTrigger).unwrap();

        // Evaluate objective climate logs against target parameter configurations
        if reported_wind_speed >= trigger_speed {
            let token_addr: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();
            let holder: Address = env.storage().instance().get(&DataKey::PolicyHolder).unwrap();
            let payout: i128 = env.storage().instance().get(&DataKey::PayoutAmount).unwrap();
            let contract_address = env.current_contract_address();

            let usdc_client = token::Client::new(&env, &token_addr);

            // Execute instantaneous programmatic relief payout direct to the cooperative's wallet
            usdc_client.transfer(&contract_address, &holder, &payout);

            env.storage().instance().set(&DataKey::IsClaimSettled, &true);
            env.storage().instance().set(&DataKey::IsPolicyActive, &false);

            env.events().publish(
                (Symbol::new(&env, "disaster_relief_disbursed"), holder),
                payout,
            );
        } else {
            // Log update if metrics do not breach the threshold
env.events().publish(
    (Symbol::new(&env, "weather_log_recorded_below_trigger"),), // Notice the enclosing tuple and trailing comma
    reported_wind_speed,
);
        }
    }

    /// Read-only optimization helper to evaluate internal policy tracking checkpoints.
    pub fn get_policy_state(env: Env) -> (bool, bool) {
        let active: bool = env.storage().instance().get(&DataKey::IsPolicyActive).unwrap_or(false);
        let settled: bool = env.storage().instance().get(&DataKey::IsClaimSettled).unwrap_or(false);
        (active, settled)
    }
}