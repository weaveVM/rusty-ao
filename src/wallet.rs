use bundlr_sdk::currency::arweave::Arweave;
use bundlr_sdk::currency::solana::Solana;

#[derive(Clone)]
pub enum SignerTypes {
    Solana(String),
    Arweave(String),
}

pub enum Signers {
    Solana(Solana),
    Arweave(Arweave),
}
