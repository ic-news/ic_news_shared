use candid::{CandidType, Deserialize, Principal};
use crate::auth::permissions::{authenticated_call, CanisterRole};
use crate::user::UserProfile;

// User Center Interface
#[derive(CandidType, Deserialize)]
pub struct UserCenterInterface {
    pub canister_id: Principal,
}

impl UserCenterInterface {
    pub fn new(canister_id: Principal) -> Self {
        Self { canister_id }
    }

    // Get user profile with authentication
    pub async fn get_user_profile(&self, user_id: Principal) -> Result<Option<UserProfile>, String> {
        let result: Result<(Option<UserProfile>,), _> = authenticated_call(
            self.canister_id,
            "get_user_profile",
            (user_id,),
            CanisterRole::UserCenter,
        )
        .await;
        result.map(|(profile,)| profile)
            .map_err(|e| format!("Failed to get user profile: {}", e.1))
    }

    // Update user points with authentication
    pub async fn update_user_points(&self, user_id: Principal, points: i64) -> Result<(), String> {
        authenticated_call(
            self.canister_id,
            "update_user_points",
            (user_id, points),
            CanisterRole::UserCenter,
        )
        .await
        .map(|_: ()| ())
        .map_err(|e| format!("Failed to update user points: {}", e.1))
    }

    // Get user points with authentication
    pub async fn get_user_points(&self, user_id: Principal) -> Result<i64, String> {
        let result: Result<(i64,), _> = authenticated_call(
            self.canister_id,
            "get_user_points",
            (user_id,),
            CanisterRole::UserCenter,
        )
        .await;
        result.map(|(points,)| points)
            .map_err(|e| format!("Failed to get user points: {}", e.1))
    }
    
    // Get user social info with authentication
    // This returns a candid-encoded response that can be decoded by the caller
    pub async fn get_user_social_info(&self, user_id: String, caller: Option<Principal>) -> Result<Vec<u8>, String> {
        let result: Result<(Vec<u8>,), _> = authenticated_call(
            self.canister_id,
            "get_user_social_info",
            (user_id, caller),
            CanisterRole::UserCenter,
        )
        .await;
        result.map(|(info,)| info)
            .map_err(|e| format!("Failed to get user social info: {}", e.1))
    }
}
