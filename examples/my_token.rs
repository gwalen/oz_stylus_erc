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
const ALICE_PRIV_KEY_PATH: &str = "ALICE_PRIV_KEY_PATH";

/// deployer private key file path.
const BOB_PRIV_KEY_PATH: &str = "BOB_PRIV_KEY_PATH";

/// Stylus RPC endpoint url.
const RPC_URL: &str = "RPC_URL";

/// Deployed program address.
const MY_TOKEN_PROGRAM_ADDRESS: &str = "STYLUS_PROGRAM_ADDRESS";

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

type MyTokenType = MyToken<SignerMiddleware<Provider<Http>, LocalWallet>>;

#[tokio::main]
async fn main() -> eyre::Result<()> {
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

    let my_token_alice_signer = MyToken::new(my_token_address, alice_client);
    let my_token_bob_signer = MyToken::new(my_token_address, bob_client);

    /****  call MyToken contracts methods ****/

    let token_name: String = my_token_alice_signer.name().call().await?;
    println!("token name: {}", token_name);

    // Alice is the deployer
    mint(&my_token_alice_signer, alice_wallet.address()).await?;

    transfer(
        &my_token_alice_signer,
        alice_wallet.address(),
        bob_wallet.address(),
        100.into(),
    )
    .await?;

    approve(
        &my_token_bob_signer,
        bob_wallet.address(),
        alice_wallet.address(),
        100.into(),
    )
    .await?; // approve for bob's funds for alice

    transfer_from(
        &my_token_alice_signer,
        bob_wallet.address(),
        alice_wallet.address(),
        100.into(),
    )
    .await?; // alice is calling to make a transfer from bob to herself based on allowance

    Ok(())
}

async fn mint(my_token: &MyTokenType, to: Address) -> eyre::Result<()> {
    println!("--- Mint");
    let deployer_balance: U256 = my_token.balance_of(to).call().await?;
    println!("mint 'to' balance before : {}", deployer_balance);

    let mint_tx: TransactionReceipt = my_token
        .mint(to, 1000.into())
        .send()
        .await?
        .await?
        .expect("Mint tx returned non");
    println!("mint tx: {:?}", mint_tx.transaction_hash);

    let deployer_balance: U256 = my_token.balance_of(to).call().await?;
    println!("mint 'to' balance after : {}", deployer_balance);

    Ok(())
}

async fn transfer(
    my_token: &MyTokenType,
    from: Address,
    to: Address,
    amount: U256,
) -> eyre::Result<()> {
    println!("--- Transfer");
    let from_balance_before: U256 = my_token.balance_of(from).call().await?;
    println!("from balance before : {}", from_balance_before);
    let to_balance_before: U256 = my_token.balance_of(to).call().await?;
    println!("to balance before : {}", to_balance_before);

    let tx: TransactionReceipt = my_token
        .transfer(to, amount)
        .send()
        .await?
        .await?
        .expect("transfer tx returned non");
    println!("transfer tx: {:?}", tx.transaction_hash);

    let from_balance_after = my_token.balance_of(from).call().await?;
    println!("from balance after : {}", from_balance_after);
    let to_balance_after = my_token.balance_of(to).call().await?;
    println!("to balance after : {}", to_balance_after);

    Ok(())
}

async fn approve(
    my_token_owner_signer: &MyTokenType,
    owner: Address,
    spender: Address,
    amount: U256,
) -> eyre::Result<()> {
    println!("--- Approve");
    let mut approved_amount: U256 = my_token_owner_signer.allowance(owner, spender).call().await?;
    println!("approved amount before : {}", approved_amount);

    let approve_tx: TransactionReceipt = my_token_owner_signer
        .approve(spender, amount)
        .send()
        .await?
        .await?
        .expect("approve tx returned non");
    println!("approve tx: {:?}", approve_tx.transaction_hash);

    approved_amount = my_token_owner_signer.allowance(owner, spender).call().await?;
    println!("approved amount after : {}", approved_amount);

    Ok(())
}

async fn transfer_from(
    my_token: &MyTokenType,
    from: Address,
    to: Address,
    amount: U256,
) -> eyre::Result<()> {
    println!("--- Transfer from");
    let from_balance_before: U256 = my_token.balance_of(from).call().await?;
    println!("from balance before : {}", from_balance_before);
    let to_balance_before: U256 = my_token.balance_of(to).call().await?;
    println!("to balance before : {}", to_balance_before);

    let tx: TransactionReceipt = my_token
        .transfer_from(from, to, amount)
        .send()
        .await?
        .await?
        .expect("transfer from tx returned non");
    println!("transfer from tx: {:?}", tx.transaction_hash);

    let from_balance_after = my_token.balance_of(from).call().await?;
    println!("from balance after : {}", from_balance_after);
    let to_balance_after = my_token.balance_of(to).call().await?;
    println!("to balance after : {}", to_balance_after);

    Ok(())
}

fn read_secret_from_file(fpath: &str) -> eyre::Result<String> {
    let f = std::fs::File::open(fpath)?;
    let mut buf_reader = BufReader::new(f);
    let mut secret = String::new();
    buf_reader.read_line(&mut secret)?;
    Ok(secret.trim().to_string())
}
