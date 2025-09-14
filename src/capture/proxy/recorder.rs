use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

use super::records::RequestRecord;

pub struct RequestRecorder {
    records: Arc<Mutex<Vec<RequestRecord>>>,
}

impl RequestRecorder {
    pub fn new() -> Self {
        Self {
            records: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn record_request(&self, record: RequestRecord) {
        debug!("Recording request: {} {}", record.method, record.url);
        let mut records = self.records.lock().await;
        records.push(record);
    }

    pub async fn get_records(&self) -> Vec<RequestRecord> {
        let records = self.records.lock().await;
        records.clone()
    }

    pub async fn clear_records(&self) {
        let mut records = self.records.lock().await;
        records.clear();
        debug!("Cleared all recorded requests");
    }
}

impl Default for RequestRecorder {
    fn default() -> Self {
        Self::new()
    }
}
