#[cfg(test)]
mod tests {
    use super::*;
    use crate::cross_chain_transfer::{CrossChainTransfer, TransferStatus};
    use crate::bridge::{Bridge, BridgeError};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_successful_transfer() {
        let bridge = Arc::new(Mutex::new(Bridge::new()));
        let transfer = CrossChainTransfer::new("source_chain", "target_chain", 100, bridge.clone());

        let result = transfer.execute().await;

        assert!(result.is_ok());
        assert_eq!(transfer.status().await, TransferStatus::Completed);
    }

    #[tokio::test]
    async fn test_insufficient_funds() {
        let bridge = Arc::new(Mutex::new(Bridge::new()));
        bridge.lock().unwrap().set_balance("source_chain", 50);
        let transfer = CrossChainTransfer::new("source_chain", "target_chain", 100, bridge.clone());

        let result = transfer.execute().await;

        assert!(matches!(result, Err(BridgeError::InsufficientFunds)));
        assert_eq!(transfer.status().await, TransferStatus::Failed);
    }

    #[tokio::test]
    async fn test_bridge_unavailable() {
        let bridge = Arc::new(Mutex::new(Bridge::new()));
        bridge.lock().unwrap().set_available(false);
        let transfer = CrossChainTransfer::new("source_chain", "target_chain", 100, bridge.clone());

        let result = transfer.execute().await;

        assert!(matches!(result, Err(BridgeError::Unavailable)));
        assert_eq!(transfer.status().await, TransferStatus::Failed);
    }

    #[tokio::test]
    async fn test_partial_transfer() {
        let bridge = Arc::new(Mutex::new(Bridge::new()));
        bridge.lock().unwrap().set_balance("source_chain", 75);
        let transfer = CrossChainTransfer::new("source_chain", "target_chain", 100, bridge.clone());

        let result = transfer.execute().await;

        assert!(result.is_ok());
        assert_eq!(transfer.status().await, TransferStatus::Partial);
    }

    #[tokio::test]
    async fn test_transfer_retries_on_failure() {
        let bridge = Arc::new(Mutex::new(Bridge::new()));
        let transfer = CrossChainTransfer::new("source_chain", "target_chain", 100, bridge.clone());

        // Simulate bridge being temporarily unavailable
        tokio::spawn({
            let bridge = bridge.clone();
            async move {
                sleep(Duration::from_secs(1)).await;
                bridge.lock().unwrap().set_available(true);
            }
        });

        let result = transfer.execute().await;

        assert!(result.is_ok());
        assert_eq!(transfer.status().await, TransferStatus::Completed);
    }
}
