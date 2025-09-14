use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::capture::proxy::RequestRecord;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub name: String,
    pub url: String,
    pub created_at: DateTime<Utc>,
    pub requests: Vec<RequestRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub name: String,
    pub url: String,
    pub created_at: DateTime<Utc>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotData {
    pub metadata: SnapshotMetadata,
    pub requests: Vec<RequestRecord>,
}

#[derive(Debug, Clone)]
pub struct SnapshotInfo {
    pub name: String,
    pub url: String,
    pub created_at: DateTime<Utc>,
}
