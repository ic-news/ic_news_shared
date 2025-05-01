use candid::{CandidType, Deserialize};

// Response for cycles balance query
#[derive(CandidType, Deserialize, Clone)]
pub struct CyclesBalanceResponse {
    pub balance: u64,
    pub balance_in_trillion: f64,  // Balance in trillion cycles (approximately ICP)
    pub estimated_days_remaining: u64,  // Estimated days remaining based on current consumption
    pub threshold_warning: bool,  // True if balance is below warning threshold
}

// Response for cycles consumption history
#[derive(CandidType, Deserialize, Clone)]
pub struct CyclesConsumptionResponse {
    pub daily_consumption: Vec<DailyConsumption>,
    pub average_daily_consumption: u64,
    pub total_consumed_last_week: u64,
}

// Daily consumption record
#[derive(CandidType, Deserialize, Clone)]
pub struct DailyConsumption {
    pub date: u64,  // Timestamp
    pub consumption: u64,  // Cycles consumed
    pub operations: u64,  // Number of operations
}

// Cycles threshold configuration
#[derive(CandidType, Deserialize, Clone)]
pub struct CyclesThresholdConfig {
    pub warning_threshold: u64,  // Warning threshold in cycles
    pub critical_threshold: u64,  // Critical threshold in cycles
    pub notification_enabled: bool,  // Whether to enable notifications
}

// Request to update cycles threshold configuration
#[derive(CandidType, Deserialize, Clone)]
pub struct UpdateCyclesThresholdRequest {
    pub warning_threshold: Option<u64>,
    pub critical_threshold: Option<u64>,
    pub notification_enabled: Option<bool>,
}

// Cycles warning notification
#[derive(CandidType, Deserialize, Clone)]
pub struct CyclesWarningNotification {
    pub timestamp: u64,
    pub balance: u64,
    pub threshold: u64,
    pub severity: CyclesWarningSeverity,
    pub message: String,
    pub is_acknowledged: bool,
}

// Warning severity level
#[derive(CandidType, Deserialize, Clone, PartialEq)]
pub enum CyclesWarningSeverity {
    Warning,
    Critical,
}

// Notification settings
#[derive(CandidType, Deserialize, Clone)]
pub struct NotificationSettings {
    pub enabled: bool,
    pub email: Option<String>,
}

// Heartbeat interval configuration request
#[derive(CandidType, Deserialize, Clone)]
pub struct UpdateHeartbeatIntervalRequest {
    pub interval_hours: u64,  // Heartbeat interval in hours
}

// Heartbeat interval configuration response
#[derive(CandidType, Deserialize, Clone)]
pub struct HeartbeatIntervalResponse {
    pub interval_hours: u64,  // Current heartbeat interval in hours
}

// Notification response
#[derive(CandidType, Deserialize, Clone)]
pub struct CyclesNotificationsResponse {
    pub notifications: Vec<CyclesWarningNotification>,
    pub unacknowledged_count: usize,
}
