use dotenv::dotenv;
use ethers::{
    middleware::SignerMiddleware,
    prelude::abigen,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, TransactionReceipt, U256},
};
use eyre::{eyre, Report};
use oz_stylus_erc::tokens::erc20::Erc20Params;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::OnceCell;

extern crate oz_stylus_erc;
use crate::oz_stylus_erc::tokens::my_token::MyTokenParams;


pub type SignerClient =  Arc<SignerMiddleware<Provider<Http>, LocalWallet>>;

/// deployer private key file path.
const ALICE_PRIV_KEY_PATH: &str = "ALICE_PRIV_KEY_PATH";

/// deployer private key file path.
const BOB_PRIV_KEY_PATH: &str = "BOB_PRIV_KEY_PATH";

/// Stylus RPC endpoint url.
const RPC_URL: &str = "RPC_URL";

/// Deployed program address.
const MY_TOKEN_PROGRAM_ADDRESS: &str = "STYLUS_PROGRAM_ADDRESS";

pub struct SharedFixtures {
    pub alice_wallet: LocalWallet,
    pub bob_wallet: LocalWallet,
    pub token_address: Address,
    pub alice_client: SignerClient,
    pub bob_client: SignerClient,
}

pub async fn fill_fixtures() -> eyre::Result<SharedFixtures> {
    dotenv().ok();

    let program_address = std::env::var(MY_TOKEN_PROGRAM_ADDRESS)
        .map_err(|_| eyre!("No {} env var set", MY_TOKEN_PROGRAM_ADDRESS))?;
    let alice_key_path = std::env::var(ALICE_PRIV_KEY_PATH)
        .map_err(|_| eyre!("No {} env var set", ALICE_PRIV_KEY_PATH))?;
    let rpc_url = std::env::var(RPC_URL).map_err(|_| eyre!("No {} env var set", RPC_URL))?;
    let bob_key_path = std::env::var(BOB_PRIV_KEY_PATH)
        .map_err(|_| eyre!("No {} env var set", BOB_PRIV_KEY_PATH))?;

    let provider = Provider::<Http>::try_from(rpc_url)?;
    let my_token_address: Address = program_address.parse()?;

    let alice_private_key = read_secret_from_file(&alice_key_path)?;
    let alice_wallet = LocalWallet::from_str(&alice_private_key)?;
    let chain_id = provider.get_chainid().await?.as_u64();
    let alice_client = Arc::new(SignerMiddleware::new(
        provider.clone(),
        alice_wallet.clone().with_chain_id(chain_id),
    ));

    let bob_private_key = read_secret_from_file(&bob_key_path)?;
    let bob_wallet = LocalWallet::from_str(&bob_private_key)?;
    let bob_client = Arc::new(SignerMiddleware::new(
        provider.clone(),
        bob_wallet.clone().with_chain_id(chain_id),
    ));

    Ok(SharedFixtures {
        alice_wallet,
        bob_wallet,
        token_address: my_token_address,
        alice_client,
        bob_client,
    })
}

pub fn read_secret_from_file(fpath: &str) -> eyre::Result<String> {
    Ok(std::fs::read_to_string(fpath)?)
}