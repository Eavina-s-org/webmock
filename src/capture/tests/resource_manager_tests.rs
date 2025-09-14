use futures::FutureExt;
use tokio::time::{sleep, Duration};

use crate::{capture::ResourceManager, spawn_managed};

#[tokio::test]
async fn test_resource_manager_new() {
    let manager = ResourceManager::new();
    assert_eq!(manager.active_task_count().await, 0);
    assert!(manager.active_task_names().await.is_empty());
}

#[tokio::test]
async fn test_register_and_unregister_task() {
    let manager = ResourceManager::new();

    // Register a simple task
    let handle = tokio::spawn(async {
        sleep(Duration::from_millis(10)).await;
    });

    manager.register_task("test_task".to_string(), handle).await;
    assert_eq!(manager.active_task_count().await, 1);
    assert_eq!(manager.active_task_names().await, vec!["test_task"]);

    // Unregister without abort
    let handle = manager.unregister_task("test_task", false).await;
    assert!(handle.is_some());
    assert_eq!(manager.active_task_count().await, 0);
}

#[tokio::test]
async fn test_unregister_with_abort() {
    let manager = ResourceManager::new();

    let handle = tokio::spawn(async {
        sleep(Duration::from_secs(10)).await; // Long running task
    });

    manager.register_task("long_task".to_string(), handle).await;

    // Abort the task
    let result = manager.unregister_task("long_task", true).await;
    assert!(result.is_none()); // Should be aborted
    assert_eq!(manager.active_task_count().await, 0);
}

#[tokio::test]
async fn test_task_replacement() {
    let manager = ResourceManager::new();

    // Register first task
    let handle1 = tokio::spawn(async {
        sleep(Duration::from_millis(10)).await;
    });
    manager
        .register_task("duplicate".to_string(), handle1)
        .await;

    // Replace with second task
    let handle2 = tokio::spawn(async {
        sleep(Duration::from_millis(10)).await;
    });
    manager
        .register_task("duplicate".to_string(), handle2)
        .await;

    assert_eq!(manager.active_task_count().await, 1);
}

#[tokio::test]
async fn test_wait_for_task() {
    let manager = ResourceManager::new();

    let handle = tokio::spawn(async {
        sleep(Duration::from_millis(50)).await;
    });

    manager
        .register_task("waitable_task".to_string(), handle)
        .await;

    // Wait for task completion
    let result = manager.wait_for_task("waitable_task").await;
    assert!(result.is_ok());
    assert_eq!(manager.active_task_count().await, 0);
}

#[tokio::test]
async fn test_wait_for_nonexistent_task() {
    let manager = ResourceManager::new();

    let result = manager.wait_for_task("nonexistent").await;
    assert!(result.is_ok()); // Should not error for missing task
}

#[tokio::test]
async fn test_wait_for_all_tasks() {
    let manager = ResourceManager::new();

    // Spawn multiple tasks
    for i in 0..3 {
        manager
            .spawn_managed_task(format!("task_{}", i), move || async move {
                sleep(Duration::from_millis(10 * (i + 1) as u64)).await;
            })
            .await;
    }

    assert_eq!(manager.active_task_count().await, 3);

    // Wait with sufficient timeout
    let result = manager.wait_for_all_tasks(Duration::from_secs(1)).await;
    assert!(result.is_ok());
    assert_eq!(manager.active_task_count().await, 0);
}

#[tokio::test]
async fn test_wait_for_all_tasks_timeout() {
    let manager = ResourceManager::new();

    let handle = tokio::spawn(async {
        sleep(Duration::from_secs(10)).await; // Very long task
    });
    manager.register_task("long_task".to_string(), handle).await;

    // Should timeout
    let result = manager.wait_for_all_tasks(Duration::from_millis(100)).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Timeout waiting for tasks to complete");
}

#[tokio::test]
async fn test_cleanup_handlers() {
    let manager = ResourceManager::new();
    let cleanup_called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let cleanup_flag = cleanup_called.clone();

    manager
        .register_cleanup_handler(move || {
            let flag = cleanup_flag.clone();
            async move {
                flag.store(true, std::sync::atomic::Ordering::SeqCst);
            }
            .boxed()
        })
        .await;

    // Trigger shutdown
    manager.shutdown().await;

    assert!(cleanup_called.load(std::sync::atomic::Ordering::SeqCst));
}

#[tokio::test]
async fn test_spawn_managed_task() {
    let manager = ResourceManager::new();
    let task_completed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag = task_completed.clone();

    manager
        .spawn_managed_task("managed_task".to_string(), move || {
            let flag = flag.clone();
            async move {
                sleep(Duration::from_millis(10)).await;
                flag.store(true, std::sync::atomic::Ordering::SeqCst);
            }
        })
        .await;

    // Wait for task to complete
    manager.wait_for_task("managed_task").await.unwrap();

    assert!(task_completed.load(std::sync::atomic::Ordering::SeqCst));
}

#[tokio::test]
async fn test_scoped_resource_manager() {
    let scoped_manager = ResourceManager::scoped();
    let manager = scoped_manager.manager();

    let handle = tokio::spawn(async {
        sleep(Duration::from_millis(10)).await;
    });

    manager
        .register_task("scoped_task".to_string(), handle)
        .await;
    assert_eq!(manager.active_task_count().await, 1);

    // Convert to regular manager to prevent auto-cleanup
    let regular_manager = scoped_manager.into_manager();
    assert_eq!(regular_manager.active_task_count().await, 1);
}

#[tokio::test]
async fn test_macro_spawn_managed() {
    let manager = ResourceManager::new();
    let task_completed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag = task_completed.clone();

    spawn_managed!(manager, "macro_task", async move {
        sleep(Duration::from_millis(10)).await;
        flag.store(true, std::sync::atomic::Ordering::SeqCst);
    });

    manager.wait_for_task("macro_task").await.unwrap();
    assert!(task_completed.load(std::sync::atomic::Ordering::SeqCst));
}
