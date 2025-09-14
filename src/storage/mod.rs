pub mod serialization;
pub mod types;

#[cfg(test)]
mod tests;

pub use serialization::SnapshotSerializer;
pub use types::{Snapshot, SnapshotData, SnapshotInfo, SnapshotMetadata};

use std::fs;
use std::path::PathBuf;
use tracing::{debug, info};

use crate::error::Result;

pub struct Storage {
    base_path: PathBuf,
}

impl Storage {
    pub fn new(base_path: PathBuf) -> Self {
        info!("Creating storage with base path: {:?}", base_path);
        Self { base_path }
    }

    /// Ensure the snapshots directory exists
    pub fn ensure_snapshots_dir(&self) -> Result<PathBuf> {
        let snapshots_dir = self.base_path.join("snapshots");
        if !snapshots_dir.exists() {
            debug!("Creating snapshots directory: {:?}", snapshots_dir);
            fs::create_dir_all(&snapshots_dir)?;
        }
        Ok(snapshots_dir)
    }

    /// Get the file path for a snapshot
    pub fn get_snapshot_path(&self, name: &str) -> PathBuf {
        self.base_path
            .join("snapshots")
            .join(format!("{}.msgpack", name))
    }

    /// Check if a snapshot exists
    pub fn snapshot_exists(&self, name: &str) -> bool {
        self.get_snapshot_path(name).exists()
    }

    /// Load only the metadata of a snapshot (for listing purposes)
    async fn load_snapshot_metadata(&self, name: &str) -> Result<SnapshotInfo> {
        let snapshot_path = self.get_snapshot_path(name);

        // Read file contents
        let file_data = tokio::fs::read(&snapshot_path).await?;

        // Deserialize only to get metadata
        let metadata = SnapshotSerializer::deserialize_metadata(&file_data)?;

        Ok(SnapshotInfo {
            name: metadata.name,
            url: metadata.url,
            created_at: metadata.created_at,
        })
    }

    pub async fn save_snapshot(&self, snapshot: Snapshot) -> Result<()> {
        info!("Saving snapshot: {}", snapshot.name);

        // Ensure snapshots directory exists
        self.ensure_snapshots_dir()?;

        // Get snapshot file path
        let snapshot_path = self.get_snapshot_path(&snapshot.name);

        // Estimate snapshot size to decide on serialization method
        let estimated_size = self.estimate_snapshot_size(&snapshot);

        if estimated_size > 50 * 1024 * 1024 {
            // 50MB threshold for streaming
            info!(
                "Large snapshot detected ({}MB), using streaming serialization",
                estimated_size / 1024 / 1024
            );

            // Use streaming serialization for large snapshots
            let file = tokio::fs::File::create(&snapshot_path).await?;
            let writer = file.into_std().await;
            SnapshotSerializer::serialize_streaming(&snapshot, writer)?;
        } else {
            // Use regular serialization for smaller snapshots
            let serialized_data = SnapshotSerializer::serialize(&snapshot)?;
            tokio::fs::write(&snapshot_path, serialized_data).await?;
        }

        info!(
            "Successfully saved snapshot '{}' to {:?}",
            snapshot.name, snapshot_path
        );
        Ok(())
    }

    pub async fn load_snapshot(&self, name: &str) -> Result<Snapshot> {
        info!("Loading snapshot: {}", name);

        let snapshot_path = self.get_snapshot_path(name);

        // Check if snapshot file exists
        if !snapshot_path.exists() {
            return Err(crate::error::WebMockError::SnapshotNotFound(
                name.to_string(),
            ));
        }

        // Check file size to decide on loading method
        let metadata = tokio::fs::metadata(&snapshot_path).await?;
        let file_size = metadata.len();

        let snapshot = if file_size > 50 * 1024 * 1024 {
            // 50MB threshold for streaming
            info!(
                "Large snapshot detected ({}MB), using streaming deserialization",
                file_size / 1024 / 1024
            );

            // Use streaming deserialization for large files
            let file = tokio::fs::File::open(&snapshot_path).await?;
            let reader = file.into_std().await;
            SnapshotSerializer::deserialize_streaming(reader)?
        } else {
            // Use regular deserialization for smaller files
            let file_data = tokio::fs::read(&snapshot_path).await?;
            SnapshotSerializer::deserialize(&file_data)?
        };

        info!(
            "Successfully loaded snapshot '{}' from {:?}",
            name, snapshot_path
        );
        Ok(snapshot)
    }

    pub async fn list_snapshots(&self) -> Result<Vec<SnapshotInfo>> {
        info!("Listing all snapshots");

        let snapshots_dir = self.base_path.join("snapshots");

        // If snapshots directory doesn't exist, return empty list
        if !snapshots_dir.exists() {
            debug!("Snapshots directory doesn't exist, returning empty list");
            return Ok(Vec::new());
        }

        let mut snapshots = Vec::new();
        let mut entries = tokio::fs::read_dir(&snapshots_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Only process .msgpack files
            if let Some(extension) = path.extension() {
                if extension == "msgpack" {
                    if let Some(file_stem) = path.file_stem() {
                        if let Some(name) = file_stem.to_str() {
                            // Try to load snapshot metadata
                            match self.load_snapshot_metadata(name).await {
                                Ok(info) => snapshots.push(info),
                                Err(e) => {
                                    // Log error but continue with other snapshots
                                    debug!(
                                        "Failed to load metadata for snapshot '{}': {}",
                                        name, e
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        // Sort snapshots by creation date (newest first)
        snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        info!("Found {} snapshots", snapshots.len());
        Ok(snapshots)
    }

    /// Estimate the size of a snapshot in bytes
    fn estimate_snapshot_size(&self, snapshot: &Snapshot) -> usize {
        let metadata_size = 1024; // Rough estimate for metadata
        let requests_size = snapshot
            .requests
            .iter()
            .map(|req| {
                req.url.len()
                    + req.method.len()
                    + req
                        .headers
                        .iter()
                        .map(|(k, v)| k.len() + v.len())
                        .sum::<usize>()
                    + req.body.as_ref().map(|b| b.len()).unwrap_or(0)
                    + req.response.body.len()
                    + req
                        .response
                        .headers
                        .iter()
                        .map(|(k, v)| k.len() + v.len())
                        .sum::<usize>()
            })
            .sum::<usize>();

        metadata_size + requests_size
    }

    pub async fn delete_snapshot(&self, name: &str) -> Result<()> {
        info!("Deleting snapshot: {}", name);

        let snapshot_path = self.get_snapshot_path(name);

        // Check if snapshot exists
        if !snapshot_path.exists() {
            return Err(crate::error::WebMockError::SnapshotNotFound(
                name.to_string(),
            ));
        }

        // Delete the snapshot file
        tokio::fs::remove_file(&snapshot_path).await?;

        info!(
            "Successfully deleted snapshot '{}' from {:?}",
            name, snapshot_path
        );
        Ok(())
    }
}
