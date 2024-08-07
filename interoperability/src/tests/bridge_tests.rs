#[cfg(test)]
mod bridge_tests {
    use super::super::bridge::{Bridge, BridgeError};
    use super::super::cross_chain_transfer::CrossChainTransfer;
    use super::super::cross_chain_swap::CrossChainSwap;
    use std::collections::HashMap;
    
    #[test]
    fn test_bridge_initialization() {
        let bridge = Bridge::new();
        assert!(bridge.is_initialized());
    }

    #[test]
    fn test_register_chain() {
        let mut bridge = Bridge::new();
        assert!(bridge.register_chain("ChainA").is_ok());
        assert!(bridge.is_chain_registered("ChainA"));
    }

    #[test]
    fn test_register_chain_already_registered() {
        let mut bridge = Bridge::new();
        bridge.register_chain("ChainA").unwrap();
        let result = bridge.register_chain("ChainA");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), BridgeError::ChainAlreadyRegistered);
    }

    #[test]
    fn test_cross_chain_transfer() {
        let mut bridge = Bridge::new();
        bridge.register_chain("ChainA").unwrap();
        bridge.register_chain("ChainB").unwrap();

        let transfer = CrossChainTransfer {
            from_chain: "ChainA".to_string(),
            to_chain: "ChainB".to_string(),
            amount: 100,
            sender: "UserA".to_string(),
            receiver: "UserB".to_string(),
        };

        let result = bridge.execute_transfer(transfer);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cross_chain_transfer_unregistered_chain() {
        let mut bridge = Bridge::new();
        bridge.register_chain("ChainA").unwrap();

        let transfer = CrossChainTransfer {
            from_chain: "ChainA".to_string(),
            to_chain: "ChainB".to_string(),
            amount: 100,
            sender: "UserA".to_string(),
            receiver: "UserB".to_string(),
        };

        let result = bridge.execute_transfer(transfer);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), BridgeError::ChainNotRegistered);
    }

    #[test]
    fn test_cross_chain_swap() {
        let mut bridge = Bridge::new();
        bridge.register_chain("ChainA").unwrap();
        bridge.register_chain("ChainB").unwrap();

        let swap = CrossChainSwap {
            from_chain: "ChainA".to_string(),
            to_chain: "ChainB".to_string(),
            amount: 100,
            sender: "UserA".to_string(),
            receiver: "UserB".to_string(),
            exchange_rate: 1.2,
        };

        let result = bridge.execute_swap(swap);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cross_chain_swap_unregistered_chain() {
        let mut bridge = Bridge::new();
        bridge.register_chain("ChainA").unwrap();

        let swap = CrossChainSwap {
            from_chain: "ChainA".to_string(),
            to_chain: "ChainB".to_string(),
            amount: 100,
            sender: "UserA".to_string(),
            receiver: "UserB".to_string(),
            exchange_rate: 1.2,
        };

        let result = bridge.execute_swap(swap);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), BridgeError::ChainNotRegistered);
    }

    #[test]
    fn test_query_balance() {
        let mut bridge = Bridge::new();
        bridge.register_chain("ChainA").unwrap();

        let initial_balance = bridge.query_balance("ChainA", "UserA");
        assert_eq!(initial_balance, 0);

        let transfer = CrossChainTransfer {
            from_chain: "ChainA".to_string(),
            to_chain: "ChainB".to_string(),
            amount: 100,
            sender: "UserA".to_string(),
            receiver: "UserB".to_string(),
        };
        
        bridge.execute_transfer(transfer).unwrap();

        let new_balance = bridge.query_balance("ChainA", "UserA");
        assert_eq!(new_balance, 100);
    }

    #[test]
    fn test_query_balance_unregistered_chain() {
        let bridge = Bridge::new();
        let result = bridge.query_balance("ChainA", "UserA");
        assert_eq!(result, 0);
    }
}
