use std::collections::HashMap;
use std::{future::Future, sync::Arc};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, warn};

type CleanupFuture = Box<dyn Future<Output = ()> + Send + Unpin>;
type CleanupHandler = Box<dyn Fn() -> CleanupFuture + Send + Sync>;
type CleanupHandlerList = Vec<CleanupHandler>;

/// Resource manager for tracking and cleaning up async tasks and resources
#[derive(Clone)]
pub struct ResourceManager {
    tasks: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
    cleanup_handlers: Arc<RwLock<CleanupHandlerList>>,
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            cleanup_handlers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a task with the resource manager
    pub async fn register_task(&self, name: String, handle: JoinHandle<()>) {
        debug!("Registering task: {}", name);
        let mut tasks = self.tasks.write().await;

        // If a task with this name already exists, abort it first
        if let Some(old_handle) = tasks.remove(&name) {
            warn!("Replacing existing task: {}", name);
            old_handle.abort();
        }

        tasks.insert(name, handle);
    }

    /// Unregister and optionally abort a task
    pub async fn unregister_task(&self, name: &str, abort: bool) -> Option<JoinHandle<()>> {
        debug!("Unregistering task: {} (abort: {})", name, abort);
        let mut tasks = self.tasks.write().await;

        if let Some(handle) = tasks.remove(name) {
            if abort {
                handle.abort();
                None
            } else {
                Some(handle)
            }
        } else {
            None
        }
    }

    /// Wait for a specific task to complete
    pub async fn wait_for_task(&self, name: &str) -> Result<(), tokio::task::JoinError> {
        let handle = {
            let mut tasks = self.tasks.write().await;
            tasks.remove(name)
        };

        if let Some(handle) = handle {
            debug!("Waiting for task to complete: {}", name);
            handle.await
        } else {
            debug!("Task not found: {}", name);
            Ok(())
        }
    }

    /// Get the number of active tasks
    pub async fn active_task_count(&self) -> usize {
        let tasks = self.tasks.read().await;
        tasks.len()
    }

    /// Get list of active task names
    pub async fn active_task_names(&self) -> Vec<String> {
        let tasks = self.tasks.read().await;
        tasks.keys().cloned().collect()
    }

    /// Register a cleanup handler
    pub async fn register_cleanup_handler<F, Fut>(&self, handler: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static + Unpin,
    {
        let mut handlers = self.cleanup_handlers.write().await;
        handlers.push(Box::new(move || {
            Box::new(handler()) as Box<dyn Future<Output = ()> + Send + Unpin>
        }));
    }

    /// Abort all tasks and run cleanup handlers
    pub async fn shutdown(&self) {
        debug!("Shutting down resource manager");

        // Abort all active tasks
        let task_names = {
            let tasks = self.tasks.read().await;
            tasks.keys().cloned().collect::<Vec<_>>()
        };

        for name in task_names {
            self.unregister_task(&name, true).await;
        }

        // Run cleanup handlers
        let handlers = {
            let handlers = self.cleanup_handlers.read().await;
            handlers.iter().map(|h| h()).collect::<Vec<_>>()
        };

        for handler in handlers {
            handler.await;
        }

        debug!("Resource manager shutdown complete");
    }

    /// Wait for all tasks to complete (with timeout)
    pub async fn wait_for_all_tasks(
        &self,
        timeout: std::time::Duration,
    ) -> Result<(), &'static str> {
        let start = std::time::Instant::now();

        loop {
            let count = self.active_task_count().await;
            if count == 0 {
                break;
            }

            if start.elapsed() > timeout {
                return Err("Timeout waiting for tasks to complete");
            }

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        Ok(())
    }

    /// Spawn a managed task
    pub async fn spawn_managed_task<F, Fut>(&self, name: String, future: F)
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let manager = self.clone();
        let name_for_handle = name.clone();
        let handle = tokio::spawn(async move {
            future().await;
            manager.unregister_task(&name_for_handle, false).await;
        });

        self.register_task(name, handle).await;
    }

    /// Create a scoped resource manager that automatically cleans up
    pub fn scoped() -> ScopedResourceManager {
        ScopedResourceManager {
            manager: Self::new(),
        }
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Scoped resource manager that automatically cleans up when dropped
pub struct ScopedResourceManager {
    manager: ResourceManager,
}

impl ScopedResourceManager {
    /// Get a reference to the underlying resource manager
    pub fn manager(&self) -> &ResourceManager {
        &self.manager
    }

    /// Convert to the underlying resource manager
    pub fn into_manager(self) -> ResourceManager {
        // We need to prevent Drop from running
        let manager = unsafe { std::ptr::read(&self.manager) };
        std::mem::forget(self);
        manager
    }
}

impl Drop for ScopedResourceManager {
    fn drop(&mut self) {
        // Spawn a task to handle cleanup since Drop can't be async
        let _manager = ResourceManager::new();
        let tasks = self.manager.tasks.clone();
        let handlers = self.manager.cleanup_handlers.clone();

        tokio::spawn(async move {
            // Abort all tasks
            let task_handles = {
                let mut tasks = tasks.write().await;
                tasks.drain().collect::<Vec<_>>()
            };

            for (name, handle) in task_handles {
                debug!("Aborting task on drop: {}", name);
                handle.abort();
            }

            // Run cleanup handlers
            let handlers = {
                let handlers = handlers.read().await;
                handlers.iter().map(|h| h()).collect::<Vec<_>>()
            };

            for handler in handlers {
                handler.await;
            }
        });
    }
}

/// Convenience macro for creating managed tasks
#[macro_export]
macro_rules! spawn_managed {
    ($manager:expr, $name:expr, $future:expr) => {
        $manager
            .spawn_managed_task($name.to_string(), || $future)
            .await
    };
}
