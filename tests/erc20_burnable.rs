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
use util::fixture_init::SharedFixtures;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::OnceCell;

extern crate oz_stylus_erc;
use crate::oz_stylus_erc::tokens::my_token::MyTokenParams;

mod util;

abigen!(
    MyToken,
    r#"[
        function totalSupply() external view returns (uint256)
        function balanceOf(address account) external view returns (uint256)
        function approve(address spender, uint256 amount) external returns (bool)
        function mint(address account, uint256 amount) external
        function burn(uint256 amount) external
        function burnFrom(address account, uint256 amount) external
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
    pub const INSUFFICIENT_ALLOWANCE: &str = "0xa7718e26";
    pub const INSUFFICIENT_BALANCE: &str = "0x59eca5e6";
}

static FIXTURES: OnceCell<Mutex<Fixtures>> = OnceCell::const_new();


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

    // burn and check the difference
    burn(token_signer_alice, amount)
        .await
        .unwrap();
    let alice_balance_after = balance_of(token_signer_alice, alice_address).await.unwrap();

    assert_eq!(alice_balance_before - alice_balance_after, amount);
}

#[tokio::test]
async fn burn_from_test() {
    let fixtures_mutex = init_fixtures().await.unwrap();
    let fixtures = fixtures_mutex.lock().await;

    let alice_address = fixtures.alice_wallet.address();
    let bob_address = fixtures.bob_wallet.address();
    let token_signer_alice = &fixtures.token_signer_alice;
    let token_signer_bob = &fixtures.token_signer_bob;
    let amount: U256 = 1000.into();

    // give bob some tokens
    mint(token_signer_bob, bob_address, amount).await.unwrap();
    // approve alice to spend bob's tokens, must be signed by bob
    approve(token_signer_bob, alice_address, amount)
        .await
        .unwrap();

    // alice will burn bobs tokens, so we need to check bob balance
    let bob_balance_before = balance_of(token_signer_bob, bob_address).await.unwrap();

    // alice burns bob tokens; check the difference
    burn_from(token_signer_alice, bob_address, amount)
        .await
        .unwrap();
    let bob_balance_after = balance_of(token_signer_bob, bob_address).await.unwrap();

    assert_eq!(bob_balance_before - bob_balance_after, amount);
}

#[tokio::test]
async fn burn_balance_too_small_test() {
    let fixtures_mutex = init_fixtures().await.unwrap();
    let fixtures = fixtures_mutex.lock().await;

    let alice_address = fixtures.alice_wallet.address();
    let token_signer_alice = &fixtures.token_signer_alice;
    let amount: U256 = 1000.into();

    let alice_balance = token_signer_alice
        .balance_of(alice_address)
        .call()
        .await
        .unwrap();
    // burn all alice tokens - set alice account to 0 tokens
    burn(token_signer_alice, alice_balance)
        .await
        .unwrap();

    // try to burn and check error
    let tx = burn(token_signer_alice, amount).await;

    match tx {
        Ok(_) => panic!("burn tx should fail"),
        Err(report) => {
            assert!(report
                .to_string()
                .contains(erc20_error_selector::INSUFFICIENT_BALANCE));
        }
    }    
}

#[tokio::test]
async fn burn_from_amount_bigger_than_allowance_error_test() {
    let fixtures_mutex = init_fixtures().await.unwrap();
    let fixtures = fixtures_mutex.lock().await;

    let alice_address = fixtures.alice_wallet.address();
    let bob_address = fixtures.bob_wallet.address();
    let token_signer_alice = &fixtures.token_signer_alice;
    let token_signer_bob = &fixtures.token_signer_bob;
    let amount: U256 = 1000.into();
    let amount_to_burn: U256 = amount + 1;

    // give bob some tokens
    mint(token_signer_bob, bob_address, amount).await.unwrap();
    // approve alice to spend bob's tokens, must be signed by bob
    approve(token_signer_bob, alice_address, amount)
        .await
        .unwrap();

    // burn and check the difference
    let tx = burn_from(token_signer_alice, bob_address, amount_to_burn).await;

    match tx {
        Ok(_) => panic!("burn_from tx should fail"),
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

async fn burn_from(
    my_token_signer: &MyTokenType,
    account: Address,
    amount: U256,
) -> eyre::Result<TransactionReceipt> {
    my_token_signer
        .burn_from(account, amount)
        .send()
        .await?
        .await?
        .ok_or(Report::msg("burn tx error"))
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

/*** Fixtures helper functions  ***/

async fn init_fixtures() -> eyre::Result<&'static Mutex<Fixtures>> {
    FIXTURES
        .get_or_try_init(|| async {
            let fixtures = fill_local_fixtures().await?;
            Ok(Mutex::new(fixtures))
        })
        .await
}

async fn fill_local_fixtures() -> eyre::Result<Fixtures> {
    let shared_fixture: SharedFixtures = util::fixture_init::fill_fixtures().await?;
    let token_signer_alice = MyToken::new(shared_fixture.token_address, shared_fixture.alice_client.clone());
    let token_signer_bob = MyToken::new(shared_fixture.token_address, shared_fixture.bob_client.clone());

    Ok(Fixtures {
        alice_wallet: shared_fixture.alice_wallet,
        bob_wallet: shared_fixture.bob_wallet,
        token_signer_alice,
        token_signer_bob,
    })
}