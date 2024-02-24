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
        function mint(address account, uint256 amount) external
        function burn(uint256 amount) external
    ]"#
);

type MyTokenType = MyToken<SignerMiddleware<Provider<Http>, LocalWallet>>;

struct Fixtures {
    alice_wallet: LocalWallet,
    bob_wallet: LocalWallet,
    token_signer_alice: MyTokenType,
    token_signer_bob: MyTokenType,
}

/// Errors signatures
/// you can obtain them by calculating the Error selector same as for function
/// eg: selector for Erc20InvalidSpender(address) =>
///  -> bytes4(keccak256(bytes("Erc20InvalidSpender(address)"))) == 0xf886f534
pub mod erc20_error_selector {
    pub const INVALID_SPENDER: &str = "0xf886f534";
    pub const INVALID_RECEIVER: &str = "0x5d908336";
    pub const INVALID_APPROVER: &str = "0xd15b3125";
    pub const INSUFFICIENT_ALLOWANCE: &str = "0xa7718e26";
    pub const INSUFFICIENT_BALANCE: &str = "0x59eca5e6";
}

static FIXTURES: OnceCell<Mutex<Fixtures>> = OnceCell::const_new();

#[tokio::test]
async fn erc20_params() {
    let fixtures_mutex = init_fixtures().await.unwrap();
    let fixtures = fixtures_mutex.lock().await;

    let token_signer_alice = &fixtures.token_signer_alice;

    let token_name = token_signer_alice.name().call().await.unwrap();
    let token_symbol = token_signer_alice.symbol().call().await.unwrap();
    let token_decimals = token_signer_alice.decimals().call().await.unwrap();

    assert_eq!(token_name, MyTokenParams::NAME);
    assert_eq!(token_symbol, MyTokenParams::SYMBOL);
    assert_eq!(token_decimals, MyTokenParams::DECIMALS);
}

#[tokio::test]
async fn mint_test() {
    let fixtures_mutex = init_fixtures().await.unwrap();
    let fixtures = fixtures_mutex.lock().await;

    let alice_address = fixtures.alice_wallet.address();
    let token_signer_alice = &fixtures.token_signer_alice;
    let amount: U256 = 1000.into();

    let alice_balance_before = balance_of(token_signer_alice, alice_address).await.unwrap();
    mint(token_signer_alice, alice_address, amount)
        .await
        .unwrap();
    let alice_balance_after = balance_of(token_signer_alice, alice_address).await.unwrap();

    assert_eq!(alice_balance_after - alice_balance_before, amount);
}

#[tokio::test]
async fn burn_test() {
    let fixtures_mutex = init_fixtures().await.unwrap();
    let fixtures = fixtures_mutex.lock().await;

    let alice_address = fixtures.alice_wallet.address();
    let token_signer_alice = &fixtures.token_signer_alice;
    let amount: U256 = 1000.into();

    // first get some tokens
    mint(token_signer_alice, alice_address, amount)
        .await
        .unwrap();
    let alice_balance_before = balance_of(token_signer_alice, alice_address).await.unwrap();
    println!("alice_balance_before: {}", alice_balance_before);

    // burn and check the difference
    burn(token_signer_alice, amount)
        .await
        .unwrap();
    let alice_balance_after = balance_of(token_signer_alice, alice_address).await.unwrap();
    println!("alice_balance_after: {}", alice_balance_after);

    assert_eq!(alice_balance_before - alice_balance_after, amount);
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
    approve(token_signer_bob, alice_address, amount_transfer)
        .await
        .unwrap();

    let alice_balance_before = balance_of(token_signer_alice, alice_address).await.unwrap();
    let bob_balance_before = balance_of(token_signer_alice, bob_address).await.unwrap();

    // transfer from bob to alice but alice is the signer of tx
    transfer_from(
        token_signer_alice,
        bob_address,
        alice_address,
        amount_transfer,
    )
    .await
    .unwrap();

    let alice_balance_after = balance_of(token_signer_alice, alice_address).await.unwrap();
    let bob_balance_after = balance_of(token_signer_alice, bob_address).await.unwrap();

    assert_eq!(alice_balance_after - alice_balance_before, amount_transfer);
    assert_eq!(bob_balance_before - bob_balance_after, amount_transfer);
}

#[tokio::test]
async fn approve_test() {
    let fixtures_mutex = init_fixtures().await.unwrap();
    let fixtures = fixtures_mutex.lock().await;

    let alice_address = fixtures.alice_wallet.address();
    let bob_address = fixtures.bob_wallet.address();
    let token_signer_alice = &fixtures.token_signer_alice;
    let amount: U256 = 100.into();

    approve(token_signer_alice, bob_address, 0.into())
        .await
        .unwrap();
    let allowance_before = token_signer_alice
        .allowance(alice_address, bob_address)
        .await
        .unwrap();

    approve(token_signer_alice, bob_address, amount)
        .await
        .unwrap();
    let allowance_after = token_signer_alice
        .allowance(alice_address, bob_address)
        .await
        .unwrap();

    assert_eq!(allowance_before, 0.into());
    assert_eq!(allowance_after, amount);
}

#[tokio::test]
async fn approve_account_address_0_error_test() {
    let fixtures_mutex = init_fixtures().await.unwrap();
    let fixtures = fixtures_mutex.lock().await;

    let token_signer_alice = &fixtures.token_signer_alice;
    let amount: U256 = 100.into();

    let tx = approve(token_signer_alice, Address::zero(), amount).await;
    match tx {
        Ok(_) => panic!("approve tx should fail"),
        Err(report) => {
            assert!(report
                .to_string()
                .contains(erc20_error_selector::INVALID_SPENDER));
        }
    }
}

#[tokio::test]
async fn transfer_balance_too_small_error_test() {
    let fixtures_mutex = init_fixtures().await.unwrap();
    let fixtures = fixtures_mutex.lock().await;

    let alice_address = fixtures.alice_wallet.address();
    let bob_address = fixtures.bob_wallet.address();
    let token_signer_alice = &fixtures.token_signer_alice;
    let amount_mint: U256 = 1000.into();
    let amount_transfer: U256 = amount_mint * 2;

    let alice_balance = token_signer_alice
        .balance_of(alice_address)
        .call()
        .await
        .unwrap();
    // burn all alice tokens - set alice account to 0 tokens
    burn(token_signer_alice, alice_balance)
        .await
        .unwrap();
    // from alice to bob
    let tx = transfer(token_signer_alice, bob_address, amount_transfer).await;

    match tx {
        Ok(_) => panic!("transfer tx should fail"),
        Err(report) => {
            assert!(report
                .to_string()
                .contains(erc20_error_selector::INSUFFICIENT_BALANCE));
        }
    }
}

#[tokio::test]
async fn transfer_receiver_address_0_error_test() {
    let fixtures_mutex = init_fixtures().await.unwrap();
    let fixtures = fixtures_mutex.lock().await;

    let alice_address = fixtures.alice_wallet.address();
    let token_signer_alice = &fixtures.token_signer_alice;

    mint(token_signer_alice, alice_address, 1000.into())
        .await
        .unwrap();
    let tx = transfer(token_signer_alice, Address::zero(), 100.into()).await;

    match tx {
        Ok(_) => panic!("transfer tx should fail"),
        Err(report) => {
            assert!(report
                .to_string()
                .contains(erc20_error_selector::INVALID_RECEIVER));
        }
    }
}

#[tokio::test]
async fn transfer_from_amount_bigger_than_allowance_error_test() {
    let fixtures_mutex = init_fixtures().await.unwrap();
    let fixtures = fixtures_mutex.lock().await;

    let alice_address = fixtures.alice_wallet.address();
    let bob_address = fixtures.bob_wallet.address();
    let token_signer_alice = &fixtures.token_signer_alice;
    let token_signer_bob = &fixtures.token_signer_bob;
    let amount_allowance: U256 = 100.into();
    let amount_transfer: U256 = amount_allowance * 2;

    // give bob some tokens
    mint(token_signer_bob, bob_address, 1000.into()).await.unwrap();
    // approve alice to spend bob's tokens, must be signed by bob
    approve(token_signer_bob, alice_address, amount_allowance)
        .await
        .unwrap();

    // transfer from bob to alice but alice is the signer of tx
    // amount_transfer is x2 allowance
    let tx = transfer_from(
        token_signer_alice,
        bob_address,
        alice_address,
        amount_transfer,
    )
    .await;

    match tx {
        Ok(_) => panic!("transfer from tx should fail"),
        Err(report) => {
            assert!(report
                .to_string()
                .contains(erc20_error_selector::INSUFFICIENT_ALLOWANCE));
        }
    }
}

/*** Erc20 helper functions ***/

async fn balance_of(my_token_signer: &MyTokenType, account: Address) -> eyre::Result<U256> {
    let balance: U256 = my_token_signer.balance_of(account).call().await?;
    Ok(balance)
}

async fn mint(
    my_token_signer: &MyTokenType,
    account: Address,
    amount: U256,
) -> eyre::Result<TransactionReceipt> {
    my_token_signer
        .mint(account, amount)
        .send()
        .await?
        .await?
        .ok_or(Report::msg("mint tx error"))
}

async fn burn(
    my_token_signer: &MyTokenType,
    amount: U256,
) -> eyre::Result<TransactionReceipt> {
    my_token_signer
        .burn(amount)
        .send()
        .await?
        .await?
        .ok_or(Report::msg("burn tx error"))
}

async fn transfer(
    my_token_signer: &MyTokenType,
    to: Address,
    amount: U256,
) -> eyre::Result<TransactionReceipt> {
    my_token_signer
        .transfer(to, amount)
        .send()
        .await?
        .await?
        .ok_or(Report::msg("transfer tx error"))
}

async fn approve(
    my_token_signer: &MyTokenType,
    spender: Address,
    amount: U256,
) -> eyre::Result<TransactionReceipt> {
    my_token_signer
        .approve(spender, amount)
        .send()
        .await?
        .await?
        .ok_or(Report::msg("transfer tx error"))
}

async fn transfer_from(
    my_token_signer: &MyTokenType,
    from: Address,
    to: Address,
    amount: U256,
) -> eyre::Result<TransactionReceipt> {
    my_token_signer
        .transfer_from(from, to, amount)
        .send()
        .await?
        .await?
        .ok_or(Report::msg("transfer from tx error"))
}

/*** Fixtures helper functions  ***/

async fn init_fixtures() -> eyre::Result<&'static Mutex<Fixtures>> {
    FIXTURES
        .get_or_try_init(|| async {
            let fixtures = fill_fixtures().await?;
            Ok(Mutex::new(fixtures))
        })
        .await
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
        token_signer_alice,
        token_signer_bob,
    })
}

fn read_secret_from_file(fpath: &str) -> eyre::Result<String> {
    Ok(std::fs::read_to_string(fpath)?)
}
