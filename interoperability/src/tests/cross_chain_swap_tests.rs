#[cfg(test)]
mod cross_chain_swap_tests {
    use super::*;
    use crate::cross_chain_swap::{initiate_swap, complete_swap, SwapStatus};
    use crate::cross_chain_transfer::{transfer_tokens, TransferStatus};
    use crate::bridge::Bridge;

    #[test]
    fn test_initiate_swap_success() {
        let bridge = Bridge::new();
        let from_chain = "ChainA";
        let to_chain = "ChainB";
        let from_address = "address1";
        let to_address = "address2";
        let amount = 100;

        let result = initiate_swap(&bridge, from_chain, to_chain, from_address, to_address, amount);

        assert_eq!(result.status, SwapStatus::Initiated);
        assert_eq!(result.from_chain, from_chain);
        assert_eq!(result.to_chain, to_chain);
        assert_eq!(result.from_address, from_address);
        assert_eq!(result.to_address, to_address);
        assert_eq!(result.amount, amount);
    }

    #[test]
    fn test_complete_swap_success() {
        let bridge = Bridge::new();
        let swap_id = "swap123";
        let to_address = "address2";

        // Assuming a swap has been initiated and is pending
        let initiate_result = initiate_swap(&bridge, "ChainA", "ChainB", "address1", to_address, 100);
        assert_eq!(initiate_result.status, SwapStatus::Initiated);

        let result = complete_swap(&bridge, swap_id, to_address);

        assert_eq!(result.status, SwapStatus::Completed);
        assert_eq!(result.swap_id, swap_id);
        assert_eq!(result.to_address, to_address);
    }

    #[test]
    fn test_transfer_tokens_during_swap() {
        let bridge = Bridge::new();
        let from_chain = "ChainA";
        let to_chain = "ChainB";
        let from_address = "address1";
        let to_address = "address2";
        let amount = 100;

        let swap_result = initiate_swap(&bridge, from_chain, to_chain, from_address, to_address, amount);
        assert_eq!(swap_result.status, SwapStatus::Initiated);

        let transfer_result = transfer_tokens(&bridge, from_chain, to_chain, from_address, to_address, amount);

        assert_eq!(transfer_result.status, TransferStatus::Completed);
        assert_eq!(transfer_result.from_chain, from_chain);
        assert_eq!(transfer_result.to_chain, to_chain);
        assert_eq!(transfer_result.from_address, from_address);
        assert_eq!(transfer_result.to_address, to_address);
        assert_eq!(transfer_result.amount, amount);
    }

    #[test]
    fn test_initiate_swap_insufficient_balance() {
        let bridge = Bridge::new();
        let from_chain = "ChainA";
        let to_chain = "ChainB";
        let from_address = "address1";
        let to_address = "address2";
        let amount = 1000000; // Exceeding balance

        let result = initiate_swap(&bridge, from_chain, to_chain, from_address, to_address, amount);

        assert_eq!(result.status, SwapStatus::Failed);
        assert_eq!(result.error_message.unwrap(), "Insufficient balance");
    }

    #[test]
    fn test_complete_swap_invalid_swap_id() {
        let bridge = Bridge::new();
        let invalid_swap_id = "invalid_swap";
        let to_address = "address2";

        let result = complete_swap(&bridge, invalid_swap_id, to_address);

        assert_eq!(result.status, SwapStatus::Failed);
        assert_eq!(result.error_message.unwrap(), "Invalid swap ID");
    }
}
