

use mazze_addr::Network;
use std::fmt;

#[derive(Debug)]
pub struct RcpAddressNetworkInconsistent {
    pub from_network: Network,
    pub to_network: Network,
}

impl fmt::Display for RcpAddressNetworkInconsistent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "network prefix inconsistent in from({}) and to({})",
            self.from_network, self.to_network
        )
    }
}

pub fn check_rpc_address_network(
    rpc_request_network: Option<Network>, expected: &Network,
) -> Result<(), UnexpectedRpcAddressNetwork> {
    if let Some(rpc_network) = rpc_request_network {
        if rpc_network != *expected {
            return Err(UnexpectedRpcAddressNetwork {
                expected: *expected,
                got: rpc_network,
            });
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct UnexpectedRpcAddressNetwork {
    pub expected: Network,
    pub got: Network,
}

impl fmt::Display for UnexpectedRpcAddressNetwork {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "network prefix unexpected: ours {}, got {}",
            self.expected, self.got
        )
    }
}
