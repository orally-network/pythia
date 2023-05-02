use anyhow::Result;

use ic_web3::{
    types::H160,
    ic::get_eth_addr, 
};

use crate::{
    PythiaError,
    KEY_NAME,
    types::subs::Sub
};

#[derive(Default, Debug, Clone)]
pub struct User {
    pub pub_key: H160,
    pub exec_addr: H160,
    pub subs: Vec<Sub>,
}

impl User {
    pub async fn new(pub_key: H160) -> Result<Self> {
        let derivation_path = vec![pub_key.as_bytes().to_vec()];

        let key_name = KEY_NAME.with(|key_name_state| {
            key_name_state.borrow().clone()
        });

        let exec_addr = get_eth_addr(None, Some(derivation_path), key_name)
            .await
            .map_err(|err| { PythiaError::FailedToGetEthAddress(err) })?;

        Ok(Self {
            pub_key,
            exec_addr,
            ..Default::default()
        })
    }
}