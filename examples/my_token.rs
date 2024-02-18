use ethers::{
    middleware::SignerMiddleware,
    prelude::abigen,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::Address,
};
use eyre::eyre;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use std::sync::Arc;
use dotenv::dotenv;

/// Your private key file path.
const PRIV_KEY_PATH: &str = "PRIV_KEY_PATH";

/// Stylus RPC endpoint url.
const RPC_URL: &str = "RPC_URL";

/// Deployed program address.
const STYLUS_PROGRAM_ADDRESS: &str = "STYLUS_PROGRAM_ADDRESS";

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();

    let program_address = std::env::var(STYLUS_PROGRAM_ADDRESS)
        .map_err(|_| eyre!("No {} env var set", STYLUS_PROGRAM_ADDRESS))?;
    let priv_key_path = std::env::var(PRIV_KEY_PATH)
        .map_err(|_| eyre!("No {} env var set", PRIV_KEY_PATH))?;
    let rpc_url =
        std::env::var(RPC_URL).map_err(|_| eyre!("No {} env var set", RPC_URL))?;

    abigen!(
        MyToken,
        r#"[
            function name() external view returns (string)
            function symbol() external view returns (string)
            function decimals() external view returns (uint8)
            function totalSupply() external view returns (uint256)
            function balanceOf(address account) external view returns (uint256)
            function transfer(address recipient, uint256 amount) external returns (bool)
            function allowance(address owner, address spender) external view returns (uint256)
            function approve(address spender, uint256 amount) external returns (bool)
            function transferFrom(address sender, address recipient, uint256 amount) external returns (bool)
            function mint(address to, uint256 amount) external
        ]"#
    );

    let provider = Provider::<Http>::try_from(rpc_url)?;
    let address: Address = program_address.parse()?;

    let private_key = read_secret_from_file(&priv_key_path)?;
    let wallet = LocalWallet::from_str(&private_key)?;
    let chain_id = provider.get_chainid().await?.as_u64();
    let client = Arc::new(SignerMiddleware::new(
        provider,
        wallet.clone().with_chain_id(chain_id),
    ));

    let my_token = MyToken::new(address, client);
    let token_name: String = my_token.name().call().await?;
    println!("token name: {}", token_name);

    


    Ok(())    
}

fn read_secret_from_file(fpath: &str) -> eyre::Result<String> {
    let f = std::fs::File::open(fpath)?;
    let mut buf_reader = BufReader::new(f);
    let mut secret = String::new();
    buf_reader.read_line(&mut secret)?;
    Ok(secret.trim().to_string())
}