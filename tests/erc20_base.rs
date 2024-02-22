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
use tokio::sync::Mutex;
use tokio::sync::OnceCell;

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
type SignerClient = Arc<SignerMiddleware<Provider<Http>, LocalWallet>>;

struct Fixtures {
    alice_wallet: LocalWallet,
    bob_wallet: LocalWallet,
    alice_client: SignerClient,
    bob_client: SignerClient,
    erc20_token_address: Address,
    token_signer_alice: MyTokenType,
    token_signer_bob: MyTokenType,
}

static FIXTURES: OnceCell<Mutex<Fixtures>> = OnceCell::const_new();

#[tokio::test]
async fn mint_test() {
    let fixtures_mutex = init_fixtures().await.unwrap();
    let fixtures = fixtures_mutex.lock().await;

    let alice_address = fixtures.alice_wallet.address();
    let token_signer_alice = &fixtures.token_signer_alice;
    let amount: U256 = 1000.into();

    let alice_balance_before = balance_of(token_signer_alice, alice_address).await.unwrap();
    mint(token_signer_alice, alice_address, amount).await.unwrap();
    let alice_balance_after = balance_of(token_signer_alice, alice_address).await.unwrap();

    assert_eq!(alice_balance_after - alice_balance_before, amount);
}

#[tokio::test]
async fn transfer_test() {
    let fixtures_mutex = init_fixtures().await.unwrap();
    let fixtures = fixtures_mutex.lock().await;

    let alice_address = fixtures.alice_wallet.address();
    let bob_address = fixtures.bob_wallet.address();
    let token_signer_alice = &fixtures.token_signer_alice;
    let amount_mint: U256 = 1000.into();
    let amount_transfer: U256 = 100.into();

    mint(token_signer_alice, alice_address, amount_mint).await.unwrap();

    let alice_balance_before = balance_of(token_signer_alice, alice_address).await.unwrap();
    let bob_balance_before = balance_of(token_signer_alice, bob_address).await.unwrap();

    // from alice to bob
    transfer(token_signer_alice, bob_address, amount_transfer).await.unwrap();

    let alice_balance_after = balance_of(token_signer_alice, alice_address).await.unwrap();
    let bob_balance_after = balance_of(token_signer_alice, bob_address).await.unwrap();

    assert_eq!(alice_balance_before - alice_balance_after, amount_transfer);
    assert_eq!(bob_balance_after - bob_balance_before, amount_transfer);
}

#[tokio::test]
async fn transfer_from_test() {
    let fixtures_mutex = init_fixtures().await.unwrap();
    let fixtures = fixtures_mutex.lock().await;

    let alice_address = fixtures.alice_wallet.address();
    let bob_address = fixtures.bob_wallet.address();
    let token_signer_alice = &fixtures.token_signer_alice;
    let token_signer_bob = &fixtures.token_signer_bob;
    let amount_mint: U256 = 1000.into();
    let amount_transfer: U256 = 100.into();

    // give bob some tokens
    mint(token_signer_bob, bob_address, amount_mint).await.unwrap();
    // approve alice to spend bob's tokens, must be signed by bob
    approve(token_signer_bob, bob_address, alice_address, amount_transfer)
        .await
        .unwrap();

    let alice_balance_before = balance_of(token_signer_alice, alice_address).await.unwrap();
    let bob_balance_before = balance_of(token_signer_alice, bob_address).await.unwrap();

    // transfer from bob to alice but alice is the signer of tx
    transfer_from(token_signer_alice, bob_address, alice_address, amount_transfer)
        .await
        .unwrap();

    let alice_balance_after = balance_of(token_signer_alice, alice_address).await.unwrap();
    let bob_balance_after = balance_of(token_signer_alice, bob_address).await.unwrap();

    // TODO: remove after debug
    // println!("alice before {}, alice after {}", alice_balance_before, alice_balance_after);
    // println!("bob before {}, bob after {}", bob_balance_before, bob_balance_after);
    assert_eq!(alice_balance_after - alice_balance_before, amount_transfer);
    assert_eq!(bob_balance_before - bob_balance_after, amount_transfer);
}

/*** Erc20 helper functions ***/

async fn balance_of(erc20_token: &MyTokenType, account: Address) -> eyre::Result<U256> {
    let balance: U256 = erc20_token.balance_of(account).call().await?;
    Ok(balance)
}


async fn mint(erc20_token: &MyTokenType, to: Address, amount: U256) -> eyre::Result<TransactionReceipt> {
    let mint_tx: TransactionReceipt = erc20_token
        .mint(to, amount)
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .expect("Mint tx returned non");

    Ok(mint_tx)
}

async fn transfer(
    my_token: &MyTokenType,
    to: Address,
    amount: U256,
) -> eyre::Result<TransactionReceipt> {
    let tx: TransactionReceipt = my_token
        .transfer(to, amount)
        .send()
        .await?
        .await?
        .expect("transfer tx returned non");

    Ok(tx)
}

async fn approve(
    my_token_owner_signer: &MyTokenType,
    owner: Address,
    spender: Address,
    amount: U256,
) -> eyre::Result<TransactionReceipt> {
    let tx: TransactionReceipt = my_token_owner_signer
        .approve(spender, amount)
        .send()
        .await?
        .await?
        .expect("approve tx returned non");

    Ok(tx)
}

async fn transfer_from(
    my_token: &MyTokenType,
    from: Address,
    to: Address,
    amount: U256,
) -> eyre::Result<TransactionReceipt> {
    let tx: TransactionReceipt = my_token
        .transfer_from(from, to, amount)
        .send()
        .await?
        .await?
        .expect("transfer from tx returned non");

    Ok(tx)
}

/*** Fixtures helper functions  ***/

async fn init_fixtures() -> eyre::Result<&'static Mutex<Fixtures>> {
    let aa: eyre::Result<&'static Mutex<Fixtures>> = FIXTURES
        .get_or_try_init(|| async {
            let fixtures = fill_fixtures().await?;
            Ok(Mutex::new(fixtures))
        })
        .await;

    aa
}

async fn fill_fixtures() -> eyre::Result<Fixtures> {
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

    let token_signer_alice = MyToken::new(my_token_address, alice_client.clone());
    let token_signer_bob = MyToken::new(my_token_address, bob_client.clone());

    Ok(Fixtures {
        alice_wallet,
        bob_wallet,
        alice_client,
        bob_client,
        erc20_token_address: my_token_address,
        token_signer_alice,
        token_signer_bob,
    })
}

fn read_secret_from_file(fpath: &str) -> eyre::Result<String> {
    let f = std::fs::File::open(fpath)?;
    let mut buf_reader = BufReader::new(f);
    let mut secret = String::new();
    buf_reader.read_line(&mut secret)?;
    Ok(secret.trim().to_string())
}
