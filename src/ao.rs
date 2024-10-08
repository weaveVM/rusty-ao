use crate::errors::AoErrors;
use crate::scheme::{
    ResponseCu, ResponseMu, DATA_PROTOCOL, SDK, TYPE_MESSAGE, TYPE_PROCESS, VARIANT,
};
use crate::wallet::{SignerTypes, Signers};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use bundlr_sdk;
use bundlr_sdk::currency::arweave::ArweaveBuilder;
use bundlr_sdk::currency::solana::{Solana, SolanaBuilder};
use bundlr_sdk::currency::Currency;
use bundlr_sdk::tags::Tag;
use bundlr_sdk::{BundlrTx, Signer};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Method, Request, RequestBuilder, Response, Url};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Ao {
    mu_url: String,
    cu_url: String,
    signer_type: SignerTypes,
    signer: Signers,
}

impl Ao {
    pub fn new(mu_url: String, cu_url: String, signer: SignerTypes) -> Result<Self, AoErrors> {
        Ok(Self {
            mu_url,
            cu_url,
            signer_type: signer.clone(),
            signer: Self::signer(&signer)?,
        })
    }

    fn new_bundle_item(
        data: Vec<u8>,
        target: String,
        tags: Vec<Tag>,
    ) -> Result<BundlrTx, AoErrors> {
        BundlrTx::new(
            BASE64_STANDARD
                .decode(target)
                .map_err(|e| AoErrors::Base64ReadingError)?,
            data,
            tags,
        )
        .map_err(|e| AoErrors::BundlrError)
    }

    fn signer(signer: &SignerTypes) -> Result<Signers, AoErrors> {
        match signer {
            SignerTypes::Solana(p) => Ok(Signers::Solana(
                SolanaBuilder::new()
                    .wallet(p)
                    .build()
                    .map_err(|_| AoErrors::ErrorConstructingSigner)?,
            )),
            SignerTypes::Arweave(keypair_path) => Ok(Signers::Arweave(
                ArweaveBuilder::new()
                    .keypair_path(PathBuf::from(keypair_path.clone()))
                    .build()
                    .map_err(|_| AoErrors::ErrorConstructingSigner)?,
            )),
        }
    }

    pub fn raw_signer(&self) -> Result<&dyn Signer, AoErrors> {
        match &self.signer {
            Signers::Solana(solana) => solana.get_signer().map_err(|_| AoErrors::InvalidSigner),
            Signers::Arweave(ar) => ar.get_signer().map_err(|_| AoErrors::InvalidSigner),
        }
    }

    pub async fn send(
        &self,
        process_id: String,
        data: Vec<u8>,
        msg_type: String,
        mut tags: Vec<Tag>,
    ) -> Result<ResponseMu, AoErrors> {
        Self::add_base_tags(msg_type, &mut tags);

        let mut req_headers = HeaderMap::new();
        req_headers.insert(
            "content-type",
            HeaderValue::from_str("application/octet-stream").unwrap(),
        );
        req_headers.insert("accept", HeaderValue::from_str("application/json").unwrap());

        let mut create_tx = Self::new_bundle_item(data, process_id, tags)?;
        let _ = create_tx.sign(self.raw_signer()?).await;
        let payload = create_tx
            .as_bytes()
            .map_err(|_| AoErrors::InvalidTransaction)?;

        let req = Client::new()
            .post(&self.mu_url)
            .body(payload)
            .headers(req_headers)
            .send()
            .await
            .map_err(|_| AoErrors::InvalidServerResponse)?;

        req.json::<ResponseMu>()
            .await
            .map_err(|_| AoErrors::InvalidServerResponse)
    }

    fn add_base_tags(msg_type: String, tags: &mut Vec<Tag>) {
        tags.extend(vec![
            Tag {
                name: "Data-Protocol".to_string(),
                value: DATA_PROTOCOL.to_string(),
            },
            Tag {
                name: "Variant".to_string(),
                value: VARIANT.to_string(),
            },
            Tag {
                name: "Type".to_string(),
                value: msg_type,
            },
            Tag {
                name: "SDK".to_string(),
                value: SDK.to_string(),
            },
        ]);
    }

    pub async fn eval(&self, process_id: String, code: String) -> Result<ResponseMu, AoErrors> {
        self.send(
            process_id,
            code.as_bytes().to_vec(),
            TYPE_MESSAGE.to_string(),
            vec![Tag {
                name: "Action".to_string(),
                value: "Eval".to_string(),
            }],
        )
        .await
    }

    pub async fn spawn(
        &self,
        process_name: String,
        app_name: String,
        module: String,
        scheduler: String,
    ) -> Result<ResponseMu, AoErrors> {
        // Get the current time
        let now = SystemTime::now();
        // Get the time since the Unix epoch in nanoseconds
        let unix_nano = now
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos();
        let unix_nano_str = unix_nano.to_string();

        self.send(
            "".to_string(),
            unix_nano_str.as_bytes().to_vec(),
            TYPE_PROCESS.to_string(),
            vec![
                Tag {
                    name: "Name".to_string(),
                    value: process_name,
                },
                Tag {
                    name: "App-Name".to_string(),
                    value: app_name,
                },
                Tag {
                    name: "Module".to_string(),
                    value: module,
                },
                Tag {
                    name: "Scheduler".to_string(),
                    value: scheduler,
                },
            ],
        )
        .await
    }

    pub async fn get(
        &self,
        process_id: String,
        message_id: String,
    ) -> Result<ResponseCu, AoErrors> {
        let res = reqwest::get(format!(
            "{}/result/{}?process-id={}",
            self.cu_url, message_id, process_id
        ))
        .await
        .map_err(|_| AoErrors::InvalidServerResponse)?;

        if res.status().is_redirection() {
            let new_ao = self.create_ao_from_redirection(&res)?;
            return Box::pin(async { new_ao.get(process_id, message_id).await }).await;
        } else {
            res.json::<ResponseCu>()
                .await
                .map_err(|_| AoErrors::InvalidResponseDeserialization)
        }
    }

    pub async fn dry_run(
        &self,
        process_id: String,
        data: String,
        mut tags: Vec<Tag>,
    ) -> Result<ResponseCu, AoErrors> {
        let original_tags = tags.clone();
        Self::add_base_tags(TYPE_MESSAGE.to_string(), &mut tags);

        #[derive(Serialize, Deserialize, Debug)]
        struct Item {
            pub Id: String,
            pub Target: String,
            pub Owner: String,
            pub Data: String,
            pub Tags: Vec<Tag>,
            pub Anchor: Option<String>, // Anchor is optional in Go struct
        }

        let res = Client::new()
            .post(format!(
                "{}/dry-run?process-id={}",
                &self.cu_url, process_id
            ))
            .json(&Item {
                Id: "0000000000000000000000000000000000000000001".to_string(),
                Target: process_id.clone(),
                Owner: "0000000000000000000000000000000000000000001".to_string(),
                Data: data.clone(),
                Tags: tags,
                Anchor: None,
            })
            .send()
            .await
            .map_err(|_| AoErrors::InvalidServerResponse)?;

        if res.status().is_redirection() {
            let new_ao = self.create_ao_from_redirection(&res)?;
            return Box::pin(async {
                new_ao
                    .dry_run(process_id, data.clone(), original_tags)
                    .await
            })
            .await;
        } else {
            res.json::<ResponseCu>()
                .await
                .map_err(|_| AoErrors::InvalidResponseDeserialization)
        }
    }

    fn create_ao_from_redirection(&self, res: &Response) -> Result<Ao, AoErrors> {
        let new_ao = Self::new(
            "".to_string(),
            res.headers()
                .get("Location")
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
            self.signer_type.clone(),
        )?;
        Ok(new_ao)
    }
}

#[cfg(test)]
mod tests {
    use crate::ao::Ao;
    use crate::wallet::SignerTypes;
    use bundlr_sdk::tags::Tag;

    #[tokio::test]
    pub async fn test_init() {
        let ao = Ao::new(
            "https://mu.ao-testnet.xyz".to_string(),
            "https://cu.ao-testnet.xyz".to_string(),
            SignerTypes::Arweave("test_key.json".to_string()),
        )
        .unwrap();
    }

    #[tokio::test]
    pub async fn test_result() {
        let ao = Ao::new(
            "https://mu.ao-testnet.xyz".to_string(),
            "https://cu.ao-testnet.xyz".to_string(),
            SignerTypes::Arweave("test_key.json".to_string()),
        )
        .unwrap();
        let res = ao
            .get(
                "ya9XinY0qXeYyf7HXANqzOiKns8yiXZoDtFqUMXkX0Q".to_string(),
                "5JtjkYy1hk0Zce5mP6gDWIOdt9rCSQAFX-K9jZnqniw".to_string(),
            )
            .await;
        println!("{:?}", res);
        assert!(res.is_ok());
        println!("{}", serde_json::to_string(&res.unwrap()).unwrap())
    }

    #[tokio::test]
    pub async fn test_dry_run() {
        let ao = Ao::new(
            "https://mu.ao-testnet.xyz".to_string(),
            "https://cu.ao-testnet.xyz".to_string(),
            SignerTypes::Arweave("test_key.json".to_string()),
        )
        .unwrap();
        let res = ao
            .dry_run(
                "xU9zFkq3X2ZQ6olwNVvr1vUWIjc3kXTWr7xKQD6dh10".to_string(),
                "".to_string(),
                vec![Tag {
                    name: "Action".to_string(),
                    value: "Info".to_string(),
                }],
            )
            .await;
        assert!(res.is_ok());
        println!("{}", serde_json::to_string(&res.unwrap()).unwrap())
    }
}
