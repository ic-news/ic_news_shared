use candid::Principal;
use ic_cdk::api::call::CallResult;

// Canister roles for inter-canister calls
#[derive(Clone, Debug, PartialEq)]
pub enum CanisterRole {
    UserCenter,
    RewardCenter,
    Square,
    DailyCheckin,
}

// Get the canister role based on its ID
pub fn get_canister_role(_canister_id: Principal) -> Option<CanisterRole> {
    // TODO: Implement canister ID mapping based on your deployment configuration
    None
}

// Check if the caller has permission to make inter-canister calls
pub async fn verify_canister_call_permission(
    caller: Principal,
    target_role: CanisterRole,
) -> Result<(), String> {
    match get_canister_role(caller) {
        Some(caller_role) => {
            match (caller_role, target_role) {
                // UserCenter permissions
                (CanisterRole::RewardCenter, CanisterRole::UserCenter) |
                (CanisterRole::Square, CanisterRole::UserCenter) |
                (CanisterRole::DailyCheckin, CanisterRole::UserCenter) => Ok(()),

                // RewardCenter permissions
                (CanisterRole::UserCenter, CanisterRole::RewardCenter) |
                (CanisterRole::Square, CanisterRole::RewardCenter) |
                (CanisterRole::DailyCheckin, CanisterRole::RewardCenter) => Ok(()),

                // Square permissions
                (CanisterRole::UserCenter, CanisterRole::Square) |
                (CanisterRole::RewardCenter, CanisterRole::Square) => Ok(()),

                // DailyCheckin permissions
                (CanisterRole::UserCenter, CanisterRole::DailyCheckin) |
                (CanisterRole::RewardCenter, CanisterRole::DailyCheckin) => Ok(()),

                _ => Err("Unauthorized canister call".to_string()),
            }
        }
        None => Err("Unknown canister caller".to_string()),
    }
}

// Helper function to make authenticated canister calls
pub async fn authenticated_call<T, A>(
    canister_id: Principal,
    method: &str,
    args: A,
    target_role: CanisterRole,
) -> CallResult<T> 
where
    A: candid::utils::ArgumentEncoder,
    T: for<'a> candid::utils::ArgumentDecoder<'a>,
{
    let caller = ic_cdk::caller();
    match verify_canister_call_permission(caller, target_role).await {
        Ok(_) => ic_cdk::api::call::call(canister_id, method, args).await,
        Err(e) => Err((ic_cdk::api::call::RejectionCode::CanisterError, e)),
    }
}
