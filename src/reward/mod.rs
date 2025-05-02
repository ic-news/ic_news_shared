use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::time;

// Task types and structures
#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum TaskType {
    Daily,
    Weekly,
    OneTime,
    Special,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TaskDefinition {
    pub id: String,
    pub title: String,
    pub description: String,
    pub points: u64,
    pub task_type: TaskType,
    pub completion_criteria: String,
    pub expiration_time: Option<u64>,
    pub created_at: u64,
    pub updated_at: u64,
    pub is_active: bool,
    pub requirements: Option<TaskRequirements>,
    pub canister_id: Principal,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TaskRequirements {
    pub min_level: Option<u32>,
    pub min_points: Option<u64>,
    pub prerequisites: Option<Vec<String>>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CompleteTaskRequest {
    pub task_id: String,
    pub proof: Option<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TaskCompletionResponse {
    pub success: bool,
    pub points_earned: u64,
    pub message: String,
    pub completion_time: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TaskResponse {
    pub task: TaskDefinition,
    pub is_completed: bool,
    pub last_completion_time: Option<u64>,
}

// Reward types and structures
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UserRewardsResponse {
    pub total_points: u64,
    pub completed_tasks: Vec<CompletedTask>,
    pub available_tasks: Vec<TaskResponse>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CompletedTask {
    pub task_id: String,
    pub completion_time: u64,
    pub points_earned: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct AwardPointsRequest {
    pub user_id: Principal,
    pub points: u64,
    pub reason: String,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: String,
    pub points: u64,
    pub task_type: TaskType,
    pub completion_criteria: String,
    pub expiration_time: Option<u64>,
    pub requirements: Option<TaskRequirements>,
    pub is_active: bool,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UpdateTaskRequest {
    pub task_id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub points: Option<u64>,
    pub task_type: Option<TaskType>,
    pub completion_criteria: Option<String>,
    pub expiration_time: Option<u64>,
    pub requirements: Option<TaskRequirements>,
    pub is_active: Option<bool>,
}

// Helper functions
impl TaskDefinition {
    pub fn new(request: CreateTaskRequest, id: String) -> Self {
        TaskDefinition {
            id,
            title: request.title,
            description: request.description,
            points: request.points,
            task_type: request.task_type,
            completion_criteria: request.completion_criteria,
            expiration_time: request.expiration_time,
            created_at: time(),
            updated_at: time(),
            is_active: request.is_active,
            requirements: request.requirements,
            canister_id: ic_cdk::id(),
        }
    }
}
