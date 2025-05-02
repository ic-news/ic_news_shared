use candid::{CandidType, Deserialize, Principal};

// Custom Value enum for flexible data representation
// This replaces serde_json::Value with a Candid-compatible version
#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum Value {
    Int(i64),
    Nat(u64),
    Float(f64),
    Text(String),
    Bool(bool),
    Blob(Vec<u8>),
    Array(Vec<Value>),
    Map(Vec<(String, Value)>),
    Principal(Principal),
    Null,
}
