use std::{collections::HashMap, sync::Arc};

use anyhow::{Error, Result};

use crate::{
    chain::{ethereum::EthereumProvider, starknet::StarkNetProvider, ChainProvider},
    configuration::Network,
    persistence::NetworkName,
};

pub struct NetworkManager {
    networks: HashMap<NetworkName, Network>,
}

impl NetworkManager {
    pub fn new(networks: HashMap<NetworkName, Network>) -> NetworkManager {
        NetworkManager { networks }
    }

    pub async fn provider_for_network(
        &self,
        network_name: &NetworkName,
    ) -> Result<Arc<dyn ChainProvider>> {
        let network_config = self
            .networks
            .get(network_name)
            .ok_or_else(|| Error::msg(format!("network not found: {}", network_name)))?;

        match network_config {
            Network::StarkNet(starknet) => {
                let provider = StarkNetProvider::new(&starknet.provider_url)?;
                Ok(Arc::new(provider))
            }
            Network::Ethereum(ethereum) => {
                let provider = EthereumProvider::new(&ethereum.provider_url).await?;
                Ok(Arc::new(provider))
            }
        }
    }
}