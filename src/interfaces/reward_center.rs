use candid::{CandidType, Deserialize, Principal};
use crate::auth::permissions::{authenticated_call, CanisterRole};

#[derive(CandidType, Deserialize, Clone)]
pub struct PointsTransaction {
    pub user_id: Principal,
    pub points: i64,
    pub reason: String,
    pub timestamp: u64,
}

// Reward Center Interface
#[derive(CandidType, Deserialize)]
pub struct RewardCenterInterface {
    pub canister_id: Principal,
}

impl RewardCenterInterface {
    pub fn new(canister_id: Principal) -> Self {
        Self { canister_id }
    }

    // Award points to a user with authentication
    pub async fn award_points(
        &self,
        user_id: Principal,
        points: i64,
        reason: String,
    ) -> Result<(), String> {
        authenticated_call(
            self.canister_id,
            "award_points",
            (user_id, points, reason),
            CanisterRole::RewardCenter,
        )
        .await
        .map(|_: ()| ())
        .map_err(|e| format!("Failed to award points: {}", e.1))
    }

    // Get user points with authentication
    pub async fn get_user_points(&self, user_id: Principal) -> Result<i64, String> {
        let result: Result<(i64,), _> = authenticated_call(
            self.canister_id,
            "get_user_points",
            (user_id,),
            CanisterRole::RewardCenter,
        )
        .await;
        result.map(|(points,)| points)
            .map_err(|e| format!("Failed to get user points: {}", e.1))
    }

    // Get user points history with authentication
    pub async fn get_points_history(
        &self,
        user_id: Principal,
    ) -> Result<Vec<PointsTransaction>, String> {
        let result: Result<(Vec<PointsTransaction>,), _> = authenticated_call(
            self.canister_id,
            "get_points_history",
            (user_id,),
            CanisterRole::RewardCenter,
        )
        .await;
        result.map(|(history,)| history)
            .map_err(|e| format!("Failed to get points history: {}", e.1))
    }
}
