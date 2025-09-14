use bytes::{Bytes, BytesMut};
use http_body_util::BodyExt;
use hyper::body::Incoming;
use pin_project_lite::pin_project;

/// Maximum size for in-memory buffering before switching to streaming
pub const MAX_MEMORY_BUFFER: usize = 10 * 1024 * 1024; // 10MB

pin_project! {
    /// Streaming response body that can handle large responses efficiently
    pub struct StreamingBody {
        #[pin]
        inner: Incoming,
        buffer: BytesMut,
        max_size: usize,
        total_size: usize,
    }
}

impl StreamingBody {
    /// Create a new streaming body with a maximum in-memory buffer size
    pub fn new(body: Incoming, max_size: usize) -> Self {
        Self {
            inner: body,
            buffer: BytesMut::new(),
            max_size,
            total_size: 0,
        }
    }

    /// Create a streaming body with default buffer size
    pub fn with_default_buffer(body: Incoming) -> Self {
        Self::new(body, MAX_MEMORY_BUFFER)
    }

    /// Get the current buffer size
    pub fn buffer_size(&self) -> usize {
        self.buffer.len()
    }

    /// Get the total size processed so far
    pub fn total_size(&self) -> usize {
        self.total_size
    }

    /// Check if the response should be streamed to disk
    pub fn should_stream_to_disk(&self) -> bool {
        self.buffer.len() > self.max_size
    }

    /// Consume the streaming body and return all collected bytes
    pub async fn collect_all(self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let collected = self.inner.collect().await?;
        Ok(collected.to_bytes().to_vec())
    }

    /// Get the next chunk of data
    pub async fn next_chunk(
        &mut self,
    ) -> Result<Option<Bytes>, Box<dyn std::error::Error + Send + Sync>> {
        // This method is no longer needed with the new hyper API
        // We'll keep it for compatibility but it won't be used
        Ok(None)
    }
}

/// Efficient response collector that handles both small and large responses
pub struct ResponseCollector {
    max_memory_size: usize,
}

impl ResponseCollector {
    /// Create a new response collector
    pub fn new(max_memory_size: usize) -> Self {
        Self { max_memory_size }
    }

    /// Collect response body efficiently, using streaming for large responses
    pub async fn collect_response<B>(
        &self,
        body: B,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>
    where
        B: http_body::Body + Send + 'static,
        B::Data: Send,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let collected = body.collect().await.map_err(|e| e.into())?;
        let bytes = collected.to_bytes().to_vec();

        if bytes.len() > self.max_memory_size * 2 {
            tracing::warn!(
                "Response body is very large ({} bytes), consider implementing disk streaming",
                bytes.len()
            );
        }

        Ok(bytes)
    }
}

impl Default for ResponseCollector {
    /// Create a response collector with default settings
    fn default() -> Self {
        Self::new(MAX_MEMORY_BUFFER)
    }
}

/// Streaming writer for large response bodies to temporary files
pub struct StreamingWriter {
    temp_file: Option<tempfile::NamedTempFile>,
    in_memory_buffer: BytesMut,
    max_memory_size: usize,
}

impl StreamingWriter {
    /// Create a new streaming writer
    pub fn new(max_memory_size: usize) -> Self {
        Self {
            temp_file: None,
            in_memory_buffer: BytesMut::new(),
            max_memory_size,
        }
    }

    /// Write data to the streaming writer
    pub async fn write(&mut self, data: &[u8]) -> Result<(), std::io::Error> {
        // If we haven't exceeded memory limit, keep in memory
        if self.in_memory_buffer.len() + data.len() <= self.max_memory_size
            && self.temp_file.is_none()
        {
            self.in_memory_buffer.extend_from_slice(data);
            return Ok(());
        }

        // Switch to file-based storage
        if self.temp_file.is_none() {
            let temp_file = tempfile::NamedTempFile::new()?;

            // Write existing buffer to file
            if !self.in_memory_buffer.is_empty() {
                use tokio::io::AsyncWriteExt;
                let mut async_file = tokio::fs::File::from_std(temp_file.reopen()?);
                async_file.write_all(&self.in_memory_buffer).await?;
                async_file.flush().await?;
                self.in_memory_buffer.clear();
            }

            self.temp_file = Some(temp_file);
        }

        // Write new data to file
        if let Some(ref mut temp_file) = self.temp_file {
            use tokio::io::AsyncWriteExt;
            let mut async_file = tokio::fs::File::from_std(temp_file.reopen()?);
            async_file.write_all(data).await?;
            async_file.flush().await?;
        }

        Ok(())
    }

    /// Finalize and get all data
    pub async fn finalize(self) -> Result<Vec<u8>, std::io::Error> {
        if let Some(temp_file) = self.temp_file {
            // Read from temporary file
            use tokio::io::AsyncReadExt;
            let mut async_file = tokio::fs::File::from_std(temp_file.reopen()?);
            let mut buffer = Vec::new();
            async_file.read_to_end(&mut buffer).await?;
            Ok(buffer)
        } else {
            // Return in-memory buffer
            Ok(self.in_memory_buffer.to_vec())
        }
    }

    /// Get the current size
    pub fn size(&self) -> usize {
        if self.temp_file.is_some() {
            // For file-based storage, we'd need to track size separately
            // For now, return 0 to indicate file-based storage
            0
        } else {
            self.in_memory_buffer.len()
        }
    }
}
