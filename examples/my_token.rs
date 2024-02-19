use dotenv::dotenv;
use ethers::{
    middleware::SignerMiddleware,
    prelude::abigen,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, TransactionReceipt, U256},
};
use eyre::eyre;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use std::sync::Arc;

/// deployer private key file path.
const DEPLOYER_PRIV_KEY_PATH: &str = "DEPLOYER_PRIV_KEY_PATH";

/// second burner wallet
const SECOND_BURNER_WALLET: &str = "SECOND_BURNER_WALLET";

/// Stylus RPC endpoint url.
const RPC_URL: &str = "RPC_URL";

/// Deployed program address.
const STYLUS_PROGRAM_ADDRESS: &str = "STYLUS_PROGRAM_ADDRESS";

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

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();

    let program_address = std::env::var(STYLUS_PROGRAM_ADDRESS)
        .map_err(|_| eyre!("No {} env var set", STYLUS_PROGRAM_ADDRESS))?;
    let priv_key_path = std::env::var(DEPLOYER_PRIV_KEY_PATH)
        .map_err(|_| eyre!("No {} env var set", DEPLOYER_PRIV_KEY_PATH))?;
    let rpc_url = std::env::var(RPC_URL).map_err(|_| eyre!("No {} env var set", RPC_URL))?;
    let second_burner_wallet =
        std::env::var(SECOND_BURNER_WALLET).map_err(|_| eyre!("No {} env var set", SECOND_BURNER_WALLET))?;

    let provider = Provider::<Http>::try_from(rpc_url)?;
    let my_token_address: Address = program_address.parse()?;
    let burner_wallet_2: Address = second_burner_wallet.parse()?;

    let private_key = read_secret_from_file(&priv_key_path)?;
    let wallet = LocalWallet::from_str(&private_key)?;
    let chain_id = provider.get_chainid().await?.as_u64();
    let client = Arc::new(SignerMiddleware::new(
        provider,
        wallet.clone().with_chain_id(chain_id),
    ));

    let my_token = MyToken::new(my_token_address, client);

    // call MyToken contracts methods

    let token_name: String = my_token.name().call().await?;
    println!("token name: {}", token_name);

    mint(my_token.clone(), wallet.clone()).await?; // TODO: borrow

    transfer(
        my_token.clone(),
        wallet.address(),
        burner_wallet_2,
        100.into(),
    ).await?;

    Ok(())
}

async fn mint(
    my_token: MyToken<SignerMiddleware<Provider<Http>, LocalWallet>>,
    wallet: LocalWallet, // TODO: use Address
) -> eyre::Result<()> {
    let deployer_balance: U256 = my_token.balance_of(wallet.address()).call().await?;
    println!("deployer balance before : {}", deployer_balance);

    let mint_tx: TransactionReceipt = my_token
        .mint(wallet.address(), 1000.into())
        .send()
        .await?
        .await?
        .expect("Mint tx returned non");
    println!("mint tx: {:?}", mint_tx.transaction_hash);

    let deployer_balance: U256 = my_token.balance_of(wallet.address()).call().await?;
    println!("deployer balance after mint : {}", deployer_balance);

    Ok(())
}

async fn transfer(
    my_token: MyToken<SignerMiddleware<Provider<Http>, LocalWallet>>,
    from: Address,
    to: Address,
    amount: U256,
) -> eyre::Result<()> {
    let from_balance_before: U256 = my_token.balance_of(from).call().await?;
    println!("deployer balance before : {}", from_balance_before);

    let tx: TransactionReceipt = my_token
        .transfer(to, amount)
        .send()
        .await?
        .await?
        .expect("transfer tx returned non");
    println!("transfer tx: {:?}", tx.transaction_hash);

    let from_balance_after = my_token.balance_of(from).call().await?;
    println!("deployer balance after mint : {}", from_balance_after);

    Ok(())
}

fn read_secret_from_file(fpath: &str) -> eyre::Result<String> {
    let f = std::fs::File::open(fpath)?;
    let mut buf_reader = BufReader::new(f);
    let mut secret = String::new();
    buf_reader.read_line(&mut secret)?;
    Ok(secret.trim().to_string())
}
