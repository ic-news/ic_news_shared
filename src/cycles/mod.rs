use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::canister_balance;
use ic_cdk::api::time;
use std::cell::RefCell;
use std::collections::VecDeque;

use crate::error::{SquareError, SquareResult};
use crate::auth;

// Helper functions for error handling
fn unauthorized_error(msg: &str, module: &str, function: &str) -> SquareError {
    SquareError::Unauthorized(format!("[{}::{}] {}", module, function, msg))
}

fn log_and_return<T>(error: SquareError) -> SquareResult<T> {
    // Log the error (in a real implementation)
    Err(error)
}

// Type definitions
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CyclesBalanceResponse {
    pub balance: u64,
    pub timestamp: u64,
    pub balance_in_trillion: f64,
    pub estimated_days_remaining: u64,
    pub threshold_warning: bool,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CyclesConsumptionResponse {
    pub daily_consumption: Vec<(u64, u64)>, // (timestamp, consumption)
    pub weekly_consumption: Vec<(u64, u64)>,
    pub monthly_consumption: Vec<(u64, u64)>,
    pub average_daily_consumption: u64,
    pub total_consumed_last_week: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UpdateCyclesThresholdRequest {
    pub warning_threshold: Option<u64>,
    pub critical_threshold: Option<u64>,
    pub notification_enabled: Option<bool>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CyclesThresholdConfig {
    pub warning_threshold: u64,
    pub critical_threshold: u64,
    pub notification_enabled: bool,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum CyclesWarningSeverity {
    Warning,
    Critical,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CyclesWarningNotification {
    pub timestamp: u64,
    pub message: String,
    pub severity: CyclesWarningSeverity,
    pub is_acknowledged: bool,
    pub balance: u64,
    pub threshold: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CyclesNotificationsResponse {
    pub notifications: Vec<CyclesWarningNotification>,
    pub unacknowledged_count: usize,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct NotificationSettings {
    pub enabled: bool,
    pub email: Option<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct DailyConsumption {
    pub date: u64,
    pub consumption: u64,
    pub operations: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UpdateHeartbeatIntervalRequest {
    pub interval_hours: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct HeartbeatIntervalResponse {
    pub interval_hours: u64,
}

// Constants
const WARNING_THRESHOLD: u64 = 100_000_000_000;  // 100 billion cycles (0.1 ICP)
const CRITICAL_THRESHOLD: u64 = 50_000_000_000;  // 50 billion cycles (0.05 ICP)
const TRILLION: f64 = 1_000_000_000_000.0;
const MAX_HISTORY_DAYS: usize = 30;  // Keep 30 days of history
const ESTIMATED_DAILY_CONSUMPTION: u64 = 5_000_000_000;  // 5 billion cycles per day by default
const SECONDS_IN_DAY: u64 = 24 * 60 * 60; // 24 hours in seconds
const CYCLES_HISTORY_MAX_DAYS: usize = 30; // Keep 30 days of history

// Thread-local storage for cycles consumption history
thread_local! {
    static CYCLES_HISTORY: RefCell<VecDeque<DailyConsumption>> = RefCell::new(VecDeque::with_capacity(MAX_HISTORY_DAYS));
    static LAST_RECORDED_BALANCE: RefCell<u64> = RefCell::new(0);
    static CYCLES_THRESHOLD_CONFIG: RefCell<CyclesThresholdConfig> = RefCell::new(CyclesThresholdConfig {
        warning_threshold: WARNING_THRESHOLD,
        critical_threshold: CRITICAL_THRESHOLD,
        notification_enabled: true,
    });
    static CYCLES_NOTIFICATIONS: RefCell<Vec<CyclesWarningNotification>> = RefCell::new(Vec::new());
    static NOTIFICATION_SETTINGS: RefCell<NotificationSettings> = RefCell::new(NotificationSettings {
        enabled: false,
        email: None,
    });
    static LAST_NOTIFICATION_TIME: RefCell<u64> = RefCell::new(0);
}

// Initialize cycles monitoring
pub fn init_cycles_monitoring() {
    let current_balance = canister_balance();
    LAST_RECORDED_BALANCE.with(|balance| {
        *balance.borrow_mut() = current_balance;
    });
    
    // Initialize with empty history
    CYCLES_HISTORY.with(|history| {
        history.borrow_mut().clear();
    });
}

// Record daily cycles consumption
pub fn record_cycles_consumption() {
    let current_balance = canister_balance();
    let current_time = time();
    
    LAST_RECORDED_BALANCE.with(|last_balance| {
        let last = *last_balance.borrow();
        
        // Only record if balance decreased (consumption occurred)
        if last > current_balance {
            let consumption = last - current_balance;
            
            CYCLES_HISTORY.with(|history| {
                let mut history_mut = history.borrow_mut();
                
                // Add new consumption record
                let daily_consumption = DailyConsumption {
                    date: current_time,
                    consumption,
                    operations: 1, // Increment for each recording period
                };
                
                // Add to history and maintain max size
                history_mut.push_back(daily_consumption);
                if history_mut.len() > MAX_HISTORY_DAYS {
                    history_mut.pop_front();
                }
            });
        }
        
        // Update last recorded balance
        *last_balance.borrow_mut() = current_balance;
    });
    
    // Check if balance is below threshold and notify if needed
    check_balance_threshold();
}

// Get current cycles balance
pub fn get_cycles_balance() -> SquareResult<CyclesBalanceResponse> {
    const MODULE: &str = "services::cycles";
    const FUNCTION: &str = "get_cycles_balance";
    
    
    let current_balance = canister_balance();
    let balance_in_trillion = current_balance as f64 / TRILLION;
    
    // Calculate estimated days remaining based on average consumption
    let average_daily_consumption = get_average_daily_consumption();
    let estimated_days = if average_daily_consumption > 0 {
        current_balance / average_daily_consumption
    } else {
        current_balance / ESTIMATED_DAILY_CONSUMPTION
    };
    
    // Check if balance is below warning threshold
    let threshold_warning = CYCLES_THRESHOLD_CONFIG.with(|config| {
        let config_ref = config.borrow();
        current_balance < config_ref.warning_threshold
    });
    
    // Log low balance warning if necessary
    
    Ok(CyclesBalanceResponse {
        balance: current_balance,
        timestamp: time(),
        balance_in_trillion,
        estimated_days_remaining: estimated_days,
        threshold_warning,
    })
}

// Get cycles consumption history
pub fn get_cycles_consumption_history() -> SquareResult<CyclesConsumptionResponse> {
    const MODULE: &str = "services::cycles";
    const FUNCTION: &str = "get_cycles_consumption_history";
    
    
    let mut daily_consumption = Vec::new();
    let mut total_consumed = 0u64;
    let mut total_days = 0usize;
    
    CYCLES_HISTORY.with(|history| {
        let history_ref = history.borrow();
        
        // Get last 7 days for weekly consumption
        let week_history: Vec<&DailyConsumption> = history_ref.iter()
            .rev()
            .take(7)
            .collect();
            
        // Calculate total consumed in the last week
        for day in &week_history {
            total_consumed += day.consumption;
        }
        
        // Convert all history to vector
        daily_consumption = history_ref.iter().cloned().collect();
        total_days = if history_ref.len() > 0 { history_ref.len() } else { 1 };
    });
    
    // Calculate average daily consumption
    let average_daily_consumption = if total_days > 0 {
        total_consumed / total_days as u64
    } else {
        ESTIMATED_DAILY_CONSUMPTION
    };
    
    // 将 DailyConsumption 转换为 (u64, u64) 元组
    let daily_consumption_tuples: Vec<(u64, u64)> = daily_consumption
        .iter()
        .map(|dc| (dc.date, dc.consumption))
        .collect();
    
    // 创建空的周和月消耗数据
    let weekly_consumption: Vec<(u64, u64)> = Vec::new();
    let monthly_consumption: Vec<(u64, u64)> = Vec::new();
    
    Ok(CyclesConsumptionResponse {
        daily_consumption: daily_consumption_tuples,
        weekly_consumption,
        monthly_consumption,
        average_daily_consumption,
        total_consumed_last_week: total_consumed,
    })
}

// Update cycles threshold configuration
pub fn update_cycles_threshold(request: UpdateCyclesThresholdRequest, _caller: Principal) -> SquareResult<()> {
    const MODULE: &str = "services::cycles";
    const FUNCTION: &str = "update_cycles_threshold";
    
    
    // Only admin can update threshold configuration
    match auth::is_admin() {
        Err(msg) => return log_and_return(unauthorized_error(
            &msg,
            MODULE,
            FUNCTION
        )),
        Ok(_) => {
            
        }
    }
    
    CYCLES_THRESHOLD_CONFIG.with(|config| {
        let mut config_mut = config.borrow_mut();
        
        if let Some(warning) = request.warning_threshold {
            config_mut.warning_threshold = warning;
        }
        
        if let Some(critical) = request.critical_threshold {
            config_mut.critical_threshold = critical;
        }
        
        if let Some(enabled) = request.notification_enabled {
            config_mut.notification_enabled = enabled;
        }
    });
    
    Ok(())
}

// Get current cycles threshold configuration
pub fn get_cycles_threshold() -> SquareResult<CyclesThresholdConfig> {
    const MODULE: &str = "services::cycles";
    const FUNCTION: &str = "get_cycles_threshold";
    
    CYCLES_THRESHOLD_CONFIG.with(|config| {
        Ok(config.borrow().clone())
    })
}

// Get all cycles notifications
pub fn get_cycles_notifications() -> SquareResult<CyclesNotificationsResponse> {
    const MODULE: &str = "services::cycles";
    const FUNCTION: &str = "get_cycles_notifications";
    
    let mut notifications = Vec::new();
    let mut unacknowledged_count = 0;
    
    CYCLES_NOTIFICATIONS.with(|notifs| {
        let notifs_ref = notifs.borrow();
        notifications = notifs_ref.clone();
        
        // Count unacknowledged notifications
        unacknowledged_count = notifs_ref.iter()
            .filter(|n| !n.is_acknowledged)
            .count();
    });
    
    Ok(CyclesNotificationsResponse {
        notifications,
        unacknowledged_count,
    })
}

// Acknowledge a notification
pub fn acknowledge_notification(timestamp: u64, _caller: Principal) -> SquareResult<()> {
    const MODULE: &str = "services::cycles";
    const FUNCTION: &str = "acknowledge_notification";
    
    // Only admin can acknowledge notifications
    match auth::is_admin() {
        Err(msg) => return log_and_return(unauthorized_error(
            &msg,
            MODULE,
            FUNCTION
        )),
        Ok(_) => {}
    }
    
    CYCLES_NOTIFICATIONS.with(|notifs| {
        let mut notifs_mut = notifs.borrow_mut();
        
        // Find the notification with the given timestamp
        for notification in notifs_mut.iter_mut() {
            if notification.timestamp == timestamp {
                notification.is_acknowledged = true;
                break;
            }
        }
    });
    
    Ok(())
}

// Update notification settings
pub fn update_notification_settings(email_enabled: Option<bool>, email_address: Option<String>, 
                                   _notification_frequency_hours: Option<u64>, _caller: Principal) -> SquareResult<()> {
    const MODULE: &str = "services::cycles";
    const FUNCTION: &str = "update_notification_settings";
    
    // Only admin can update notification settings
    match auth::is_admin() {
        Err(msg) => return log_and_return(unauthorized_error(
            &msg,
            MODULE,
            FUNCTION
        )),
        Ok(_) => {}
    }
    
    NOTIFICATION_SETTINGS.with(|settings| {
        let mut settings_mut = settings.borrow_mut();
        
        if let Some(enabled) = email_enabled {
            settings_mut.enabled = enabled;
        }
        
        if let Some(address) = email_address {
            settings_mut.email = Some(address);
        }
    });
    
    Ok(())
}

// Get current notification settings
pub fn get_notification_settings(_caller: Principal) -> SquareResult<NotificationSettings> {
    const MODULE: &str = "services::cycles";
    const FUNCTION: &str = "get_notification_settings";
    
    // Only admin can view notification settings
    match auth::is_admin() {
        Err(msg) => return log_and_return(unauthorized_error(
            &msg,
            MODULE,
            FUNCTION
        )),
        Ok(_) => {}
    }
    
    NOTIFICATION_SETTINGS.with(|settings| {
        Ok(settings.borrow().clone())
    })
}

// Private helper functions

// Check if balance is below threshold and handle accordingly
fn check_balance_threshold() {
    let current_balance = canister_balance();
    let current_time = time();
    
    CYCLES_THRESHOLD_CONFIG.with(|config| {
        let config_ref = config.borrow();
        
        if !config_ref.notification_enabled {
            return;
        }
        
        // Check if we should create a notification based on thresholds
        if current_balance < config_ref.critical_threshold {
            // Create a critical notification
            create_warning_notification(
                current_balance,
                config_ref.critical_threshold,
                CyclesWarningSeverity::Critical,
                format!("CRITICAL: Cycles balance is critically low ({}). Immediate action required!", current_balance)
            );
            
            // Implement additional emergency actions
            // For example, disable non-essential operations to save cycles
            emergency_cycles_conservation();
            
        } else if current_balance < config_ref.warning_threshold {
            
            // Create a warning notification
            create_warning_notification(
                current_balance,
                config_ref.warning_threshold,
                CyclesWarningSeverity::Warning,
                format!("WARNING: Cycles balance is running low ({}). Consider topping up soon.", current_balance)
            );
        }
        
        // Check if we should send an email notification
        should_send_email_notification(current_time);
    });
}

// Create a warning notification
fn create_warning_notification(balance: u64, threshold: u64, severity: CyclesWarningSeverity, message: String) {
    let notification = CyclesWarningNotification {
        timestamp: time(),
        balance,
        threshold,
        severity: severity.clone(),
        message,
        is_acknowledged: false,
    };
    
    CYCLES_NOTIFICATIONS.with(|notifications| {
        let mut notifications_mut = notifications.borrow_mut();
        
        // Check if we already have a similar unacknowledged notification
        let has_similar = notifications_mut.iter().any(|n| {
            !n.is_acknowledged && n.severity == severity
        });
        
        // Only add if we don't have a similar unacknowledged notification
        if !has_similar {
            notifications_mut.push(notification);
            
            // Keep only the last 50 notifications
            if notifications_mut.len() > 50 {
                notifications_mut.remove(0);
            }
        }
    });
}

// Check if we should send an email notification
fn should_send_email_notification(current_time: u64) {
    NOTIFICATION_SETTINGS.with(|settings| {
        let settings_ref = settings.borrow();
        
        // If email notifications are not enabled, return
        if !settings_ref.enabled || settings_ref.email.is_none() {
            return;
        }
        
        LAST_NOTIFICATION_TIME.with(|last_time| {
            let last = *last_time.borrow();
            // Default to 24 hours if not specified
            let frequency_nanos = 24 * 3600_000_000_000;
            
            // Check if enough time has passed since the last notification
            if current_time - last > frequency_nanos {
                // Get unacknowledged critical notifications
                let has_critical = CYCLES_NOTIFICATIONS.with(|notifications| {
                    notifications.borrow().iter().any(|n| {
                        !n.is_acknowledged && n.severity == CyclesWarningSeverity::Critical
                    })
                });
                
                if has_critical {
                    // Send email notification (this is a placeholder - actual implementation would depend on your email service)
                    send_email_notification(settings_ref.email.as_ref().unwrap());
                    
                    // Update last notification time
                    *last_time.borrow_mut() = current_time;
                }
            }
        });
    });
}

use ic_cdk::api::management_canister::http_request::{HttpMethod, TransformArgs, TransformContext, http_request, CanisterHttpRequestArgument, HttpResponse};

// Send notification using Bark service
fn send_email_notification(message: &str) {
    // Log the notification message
    ic_cdk::println!("Sending notification: {}", message);
    
    // Create a custom notification with the provided message
    let custom_message = message.to_string();
    
    // Use spawn to run the async function
    ic_cdk::spawn(async move {
        // Call the Bark notification service with the custom message
        match send_custom_bark_notification(&custom_message).await {
            Ok(_) => {
                ic_cdk::println!("Successfully sent notification via Bark");
            }
            Err(e) => {
                ic_cdk::println!("Failed to send notification via Bark: {}", e);
            }
        }
    });
}

async fn send_bark_notification() -> Result<(), String> {
    // Bark API configuration
    let api_key = "K8DaSYAVyVZMToSsL2DbFn";
    let url = format!("https://api.day.app/{}", api_key);
    
    // Get current balance for notification content
    let current_balance = canister_balance();
    let balance_in_trillion = current_balance as f64 / TRILLION;
    
    // Build the request parameters for Bark API
    // Documentation: https://github.com/Finb/Bark/blob/master/README.md
    let title = "Critical: Cycles Balance Warning";
    let body = format!("Cycles balance is critically low: {} ({:.6} T). Please top up soon to avoid service interruption.", 
                      current_balance, balance_in_trillion);
    
    // URL encode the parameters
    let encoded_title = url_encode(title);
    let encoded_body = url_encode(&body);
    
    // Construct the full URL with parameters
    let full_url = format!("{}/{}?body={}&group=IC-News-Square&isArchive=1&sound=alarm", 
                          url, encoded_title, encoded_body);
    
    // Build HTTP request
    let request = CanisterHttpRequestArgument {
        url: full_url,
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(1024),
        transform: Some(TransformContext::from_name(
            "transform_bark_response".to_string(),
            vec![]
        )),
        headers: vec![],
    };
    
    // Send HTTP request
    match http_request(request, 0).await {
        Ok((response,)) => {
            // Check if status code is in 200-299 range
            // Since candid::Nat cannot be directly compared with integers, we convert it to string
            let status_str = response.status.to_string();
            let status_code = status_str.parse::<u16>().unwrap_or(0);
            if status_code >= 200 && status_code < 300 {
                Ok(())
            } else {
                Err(format!(
                    "Bark API returned error status: {}. Body: {}",
                    response.status,
                    String::from_utf8_lossy(&response.body)
                ))
            }
        }
        Err((code, message)) => {
            Err(format!("HTTP request failed with code {:?} and message: {}", code, message))
        }
    }
}

// Transform function for the Bark HTTP response
fn transform_bark_response(args: TransformArgs) -> HttpResponse {
    HttpResponse {
        status: args.response.status,
        headers: vec![],
        body: args.response.body,
    }
}

// Send a custom notification message using Bark service
async fn send_custom_bark_notification(message: &str) -> Result<(), String> {
    // Bark API configuration
    let api_key = "K8DaSYAVyVZMToSsL2DbFn";
    let url = format!("https://api.day.app/{}", api_key);
    
    // Build the request parameters for Bark API
    let title = "IC News Square Notification";
    let body = message;
    
    // URL encode the parameters
    let encoded_title = url_encode(title);
    let encoded_body = url_encode(body);
    
    // Construct the full URL with parameters
    let full_url = format!("{}/{}?body={}&group=IC-News-Square&isArchive=1", 
                          url, encoded_title, encoded_body);
    
    // Build HTTP request
    let request = CanisterHttpRequestArgument {
        url: full_url,
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(1024),
        transform: Some(TransformContext::from_name(
            "transform_bark_response".to_string(),
            vec![]
        )),
        headers: vec![],
    };
    
    // Send HTTP request
    match http_request(request, 0).await {
        Ok((response,)) => {
            // Check if status code is in 200-299 range
            let status_str = response.status.to_string();
            let status_code = status_str.parse::<u16>().unwrap_or(0);
            if status_code >= 200 && status_code < 300 {
                Ok(())
            } else {
                Err(format!(
                    "Bark API returned error status: {}. Body: {}",
                    response.status,
                    String::from_utf8_lossy(&response.body)
                ))
            }
        }
        Err((code, message)) => {
            Err(format!("HTTP request failed with code {:?} and message: {}", code, message))
        }
    }
}

// Simple URL encoding function
fn url_encode(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
            ' ' => result.push('+'),
            _ => {
                let bytes = c.to_string().into_bytes();
                for b in bytes {
                    result.push_str(&format!("%{:02X}", b));
                }
            }
        }
    }
    result
}

// Emergency cycles conservation measures
fn emergency_cycles_conservation() {
    // This function implements emergency measures to conserve cycles
    // For example:
    // 1. Disable non-essential features
    // 2. Reduce update frequency of background tasks
    // 3. Limit certain API calls to admins only
    
    // Log the emergency measures
    let _current_balance = ic_cdk::api::canister_balance();
    let _timestamp = ic_cdk::api::time() / 1_000_000; // Convert to seconds
    
    // Simple implementation that focuses on logging the emergency
    // In a real implementation, we would modify configuration settings
    // to reduce cycles consumption
    
    // Notify administrators via email (if supported)
    send_email_notification("Emergency cycles conservation measures activated due to critically low cycles balance");
}

// Get average daily consumption
fn get_average_daily_consumption() -> u64 {
    let mut total_consumption = 0u64;
    let mut days = 0usize;
    
    CYCLES_HISTORY.with(|history| {
        let history_ref = history.borrow();
        for day in history_ref.iter() {
            total_consumption += day.consumption;
            days += 1;
        }
    });
    
    if days > 0 {
        total_consumption / days as u64
    } else {
        ESTIMATED_DAILY_CONSUMPTION
    }
}
