use crate::scheme::HB_NODE_ENDPOINT;
use crate::{
    errors::HbErrors,
    wallet::{SignerTypes, Signers},
};
use bundlr_sdk::currency::arweave::ArweaveBuilder;
use bundlr_sdk::currency::solana::{Solana, SolanaBuilder};
use std::path::PathBuf;

pub struct Hyperbeam {
    pub node_endpoint: String,
    pub signer_type: SignerTypes,
    pub signer: Signers,
}

impl Hyperbeam {
    pub fn new(node_endpoint: String, signer: SignerTypes) -> Result<Self, HbErrors> {
        Ok(Self {
            node_endpoint: node_endpoint,
            signer_type: signer.clone(),
            signer: Self::signer(&signer)?,
        })
    }

    pub fn default_init(signer: SignerTypes) -> Result<Self, HbErrors> {
        Ok(Self {
            node_endpoint: HB_NODE_ENDPOINT.to_string(),
            signer_type: signer.clone(),
            signer: Self::signer(&signer)?,
        })
    }

    fn signer(signer: &SignerTypes) -> Result<Signers, HbErrors> {
        match signer {
            SignerTypes::Solana(p) => Ok(Signers::Solana(
                SolanaBuilder::new()
                    .wallet(p)
                    .build()
                    .map_err(|_| HbErrors::ErrorConstructingSigner)?,
            )),
            SignerTypes::Arweave(keypair_path) => Ok(Signers::Arweave(
                ArweaveBuilder::new()
                    .keypair_path(PathBuf::from(keypair_path.clone()))
                    .build()
                    .map_err(|_| HbErrors::ErrorConstructingSigner)?,
            )),
        }
    }

    pub async fn process_now(self, process_id: String) -> Result<serde_json::Value, HbErrors> {
        let dev_process_url = format!("{}/{}~process@1.0/now", self.node_endpoint, process_id);
        let state = reqwest::Client::new()
            .get(dev_process_url)
            .send()
            .await
            .map_err(|_| HbErrors::InvalidServerResponse)?
            .text()
            .await
            .map_err(|_| HbErrors::InvalidServerResponse)?;

        // target section header
        let target_header = "content-disposition: form-data;name=\"overview/data\"";

        if let Some(section_start) = state.find(target_header) {
            if let Some(data_start) = state[section_start..].find("\r\n\r\n") {
                let content_start = section_start + data_start + 4; // +4 to skip \r\n\r\n

                if let Some(next_boundary) = state[content_start..].find("\r\n--") {
                    let content_end = content_start + next_boundary;
                    let data = &state[content_start..content_end].trim();

                    match serde_json::from_str::<serde_json::Value>(data) {
                        Ok(state) => return Ok(state),
                        Err(e) => return Err(HbErrors::JsonError),
                    }
                }
            }
        }

        Err(HbErrors::ErrorProcessNow)
    }
}

#[cfg(test)]
mod tests {
    use crate::hyperbeam::Hyperbeam;
    use crate::scheme::HB_NODE_ENDPOINT;
    use crate::wallet::SignerTypes;
    #[tokio::test]
    pub async fn test_init() {
        let hb = Hyperbeam::new(
            HB_NODE_ENDPOINT.to_string(),
            SignerTypes::Arweave("test_key.json".to_string()),
        )
        .unwrap();
    }

    #[tokio::test]
    pub async fn test_default_init() {
        let hb =
            Hyperbeam::default_init(SignerTypes::Arweave("test_key.json".to_string())).unwrap();
    }

    #[tokio::test]
    pub async fn test_process_now() {
        let hb =
            Hyperbeam::default_init(SignerTypes::Arweave("test_key.json".to_string())).unwrap();
        let state = hb
            .process_now("oQZQd1-MztVOxODecwrxFR9UGUnsrX5wGseMJ9iSH38".to_string())
            .await
            .unwrap();
        println!("{:?}", state);
    }
}
