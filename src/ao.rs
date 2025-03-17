use crate::errors::AoErrors;
use crate::scheme::{
    ResponseCu, ResponseMu, DATA_PROTOCOL, DEFAULT_CU, DEFAULT_MU, SDK, TYPE_MESSAGE, TYPE_PROCESS,
    VARIANT,
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
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

    pub fn default_init(signer: SignerTypes) -> Result<Self, AoErrors> {
        Ok(Self {
            mu_url: DEFAULT_MU.to_string(),
            cu_url: DEFAULT_CU.to_string(),
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
        mut tags: Vec<Tag>,
    ) -> Result<ResponseMu, AoErrors> {
        let mut req_headers = HeaderMap::new();
        req_headers.insert(
            "Content-Type",
            HeaderValue::from_str("application/octet-stream").unwrap(),
        );
        req_headers.insert("Accept", HeaderValue::from_str("application/json").unwrap());

        let mut create_tx = Self::new_bundle_item(data, process_id, tags)?;
        let _ = create_tx.sign(self.raw_signer()?).await;
        let payload = create_tx
            .as_bytes()
            .map_err(|_| AoErrors::InvalidTransaction)?;

        let req = Client::new()
            .post(&self.mu_url)
            .body(payload)
            .headers(req_headers)
            .timeout(Duration::from_secs(60))
            .send()
            .await;

        match req {
            Ok(res) => {
                let res = res
                    .text()
                    .await
                    .map_err(|_| AoErrors::InvalidServerResponse)?;

                serde_json::from_str(&res).map_err(|_| AoErrors::InvalidResponseDeserialization)
            }
            Err(e) => Err(AoErrors::InvalidServerResponse),
        }
    }

    fn get_base_tags(msg_type: String) -> Vec<Tag> {
        vec![
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
        ]
    }

    pub async fn eval(&self, process_id: String, code: String) -> Result<ResponseMu, AoErrors> {
        let mut base_tags = Self::get_base_tags(TYPE_MESSAGE.to_string());
        base_tags.extend(vec![Tag {
            name: "Action".to_string(),
            value: "Eval".to_string(),
        }]);
        self.send(process_id, code.as_bytes().to_vec(), base_tags)
            .await
    }

    pub async fn spawn(
        &self,
        process_name: String,
        app_name: String,
        module: String,
        scheduler: String,
        tags: Vec<Tag>,
    ) -> Result<ResponseMu, AoErrors> {
        // Get the current time
        let now = SystemTime::now();
        // Get the time since the Unix epoch in nanoseconds
        let unix_nano = now
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos();
        let unix_nano_str = unix_nano.to_string();

        let base_tags = Self::get_base_tags("Process".to_string());

        let mut def_tags = vec![
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
            Tag {
                name: "Content-Type".to_string(),
                value: "text/plain".to_string(),
            },
        ];
        def_tags.extend(base_tags);
        def_tags.extend(tags);

        self.send("".to_string(), unix_nano_str.as_bytes().to_vec(), def_tags)
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
        tags.extend(Self::get_base_tags(TYPE_MESSAGE.to_string()));

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
    use crate::scheme::{DEFAULT_MODULE, DEFAULT_SCHEDULER};
    use crate::wallet::SignerTypes;
    use crate::scheme::Tag;

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
    pub async fn test_default_init() {
        let ao = Ao::default_init(SignerTypes::Arweave("test_key.json".to_string())).unwrap();
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
    pub async fn test_spawn() {
        let ao = Ao::default_init(SignerTypes::Arweave("test_key.json".to_string())).unwrap();
        let res = ao
            .spawn(
                "test1".to_string(),
                "rusty-ao".to_string(),
                DEFAULT_MODULE.to_string(),
                DEFAULT_SCHEDULER.to_string(),
                vec![],
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
