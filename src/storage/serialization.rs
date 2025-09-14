use super::types::{Snapshot, SnapshotData, SnapshotMetadata};
use crate::error::Result;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use std::io::{Read, Write};

/// Threshold for enabling compression (1MB)
const COMPRESSION_THRESHOLD: usize = 1024 * 1024;

pub struct SnapshotSerializer;

impl SnapshotSerializer {
    /// Serialize snapshot data to MessagePack format with optional compression
    pub fn serialize(snapshot: &Snapshot) -> Result<Vec<u8>> {
        let snapshot_data = SnapshotData {
            metadata: SnapshotMetadata {
                name: snapshot.name.clone(),
                url: snapshot.url.clone(),
                created_at: snapshot.created_at,
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            requests: snapshot.requests.clone(),
        };

        // First serialize to MessagePack
        let msgpack_data = rmp_serde::to_vec(&snapshot_data)?;

        // If data is large enough, apply compression
        if msgpack_data.len() > COMPRESSION_THRESHOLD {
            Self::compress_data(&msgpack_data)
        } else {
            Ok(msgpack_data)
        }
    }

    /// Serialize snapshot data with streaming for very large snapshots
    pub fn serialize_streaming<W: Write>(snapshot: &Snapshot, mut writer: W) -> Result<()> {
        let snapshot_data = SnapshotData {
            metadata: SnapshotMetadata {
                name: snapshot.name.clone(),
                url: snapshot.url.clone(),
                created_at: snapshot.created_at,
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            requests: snapshot.requests.clone(),
        };

        // Estimate size to decide on compression
        let estimated_size = Self::estimate_size(&snapshot_data);

        if estimated_size > COMPRESSION_THRESHOLD {
            let mut encoder = GzEncoder::new(&mut writer, Compression::default());
            rmp_serde::encode::write(&mut encoder, &snapshot_data)?;
            encoder.finish()?;
        } else {
            rmp_serde::encode::write(&mut writer, &snapshot_data)?;
        }

        Ok(())
    }

    /// Deserialize snapshot data from MessagePack format with automatic decompression
    pub fn deserialize(data: &[u8]) -> Result<Snapshot> {
        let snapshot_data: SnapshotData = if Self::is_compressed(data) {
            let decompressed = Self::decompress_data(data)?;
            rmp_serde::from_slice(&decompressed)?
        } else {
            rmp_serde::from_slice(data)?
        };

        Ok(Snapshot {
            name: snapshot_data.metadata.name,
            url: snapshot_data.metadata.url,
            created_at: snapshot_data.metadata.created_at,
            requests: snapshot_data.requests,
        })
    }

    /// Deserialize snapshot data with streaming for large files
    pub fn deserialize_streaming<R: Read>(reader: R) -> Result<Snapshot> {
        // Try to detect if the stream is compressed by reading the first few bytes
        let mut buffered_reader = std::io::BufReader::new(reader);
        let _magic_bytes = [0u8; 2];

        use std::io::BufRead;
        buffered_reader.fill_buf()?;
        let available = buffered_reader.buffer();

        let snapshot_data: SnapshotData =
            if available.len() >= 2 && available[0] == 0x1f && available[1] == 0x8b {
                // Gzip magic bytes detected
                let decoder = GzDecoder::new(buffered_reader);
                rmp_serde::decode::from_read(decoder)?
            } else {
                rmp_serde::decode::from_read(buffered_reader)?
            };

        Ok(Snapshot {
            name: snapshot_data.metadata.name,
            url: snapshot_data.metadata.url,
            created_at: snapshot_data.metadata.created_at,
            requests: snapshot_data.requests,
        })
    }

    /// Deserialize only metadata from MessagePack format (optimized for listing)
    pub fn deserialize_metadata(data: &[u8]) -> Result<SnapshotMetadata> {
        // For metadata-only access, we still need to deserialize the full structure
        // but we can optimize this in the future by storing metadata separately
        let snapshot_data: SnapshotData = if Self::is_compressed(data) {
            let decompressed = Self::decompress_data(data)?;
            rmp_serde::from_slice(&decompressed)?
        } else {
            rmp_serde::from_slice(data)?
        };

        Ok(snapshot_data.metadata)
    }

    /// Compress data using gzip
    fn compress_data(data: &[u8]) -> Result<Vec<u8>> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }

    /// Decompress gzip data
    fn decompress_data(data: &[u8]) -> Result<Vec<u8>> {
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }

    /// Check if data is gzip compressed by looking at magic bytes
    pub fn is_compressed(data: &[u8]) -> bool {
        data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b
    }

    /// Estimate the serialized size of snapshot data
    fn estimate_size(snapshot_data: &SnapshotData) -> usize {
        // Rough estimation: metadata + requests
        let metadata_size = 1024; // Rough estimate for metadata
        let requests_size = snapshot_data
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

    /// Get compression ratio for a given snapshot
    pub fn get_compression_ratio(snapshot: &Snapshot) -> Result<f64> {
        let uncompressed = rmp_serde::to_vec(&SnapshotData {
            metadata: SnapshotMetadata {
                name: snapshot.name.clone(),
                url: snapshot.url.clone(),
                created_at: snapshot.created_at,
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            requests: snapshot.requests.clone(),
        })?;

        let compressed = Self::compress_data(&uncompressed)?;

        Ok(compressed.len() as f64 / uncompressed.len() as f64)
    }
}
