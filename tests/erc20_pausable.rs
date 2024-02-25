use ethers::{
    middleware::SignerMiddleware,
    prelude::abigen,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, TransactionReceipt, U256},
};
use eyre::Report;
use util::fixture_init::SharedFixtures;
use tokio::sync::Mutex;
use tokio::sync::OnceCell;

extern crate oz_stylus_erc;

mod util;

abigen!(
    MyToken,
    r#"[
        function balanceOf(address account) external view returns (uint256)
        function approve(address spender, uint256 amount) external returns (bool)
        function mint(address account, uint256 amount) external
        function burn(uint256 amount) external
        function burnFrom(address account, uint256 amount) external
        function transfer(address recipient, uint256 amount) external returns (bool)
        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool)
        function paused() external view returns (bool)
        function isPaused() external view returns (bool)
        function pause() external
        function unpause() external
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
pub mod erc20_pausable_error_selector {
    pub const ENFORCE_PAUSE: &str = "0xd93c0665";
}

static FIXTURES: OnceCell<Mutex<Fixtures>> = OnceCell::const_new();


#[tokio::test]
async fn mint_revert_when_paused_works_when_unpaused_test() {
    let fixtures_mutex = init_fixtures().await.unwrap();
    let fixtures = fixtures_mutex.lock().await;

    let alice_address = fixtures.alice_wallet.address();
    let token_signer_alice = &fixtures.token_signer_alice;
    let amount: U256 = 1000.into();

    // try to init (we need to set cap), if already initialized ignore error

    mint(token_signer_alice, alice_address, amount)
        .await
        .unwrap();

    pause(token_signer_alice).await.unwrap();

    let tx = mint(token_signer_alice, alice_address, amount).await;
    match tx {
        Ok(_) => panic!("mint tx should fail"),
        Err(report) => {
            assert!(report
                .to_string()
                .contains(erc20_pausable_error_selector::ENFORCE_PAUSE));
        }
    }   
    // make sure we leave the contract unpaused
    unpause(token_signer_alice).await.unwrap();
}

#[tokio::test]
async fn burn_revert_when_paused_works_when_unpaused_test() {
    let fixtures_mutex = init_fixtures().await.unwrap();
    let fixtures = fixtures_mutex.lock().await;

    let alice_address = fixtures.alice_wallet.address();
    let token_signer_alice = &fixtures.token_signer_alice;
    let amount: U256 = 1000.into();

    mint(token_signer_alice, alice_address, amount)
        .await
        .unwrap();
    // burn should work here
    burn(token_signer_alice, amount / 4).await.unwrap();

    pause(token_signer_alice).await.unwrap();

    let tx = burn(token_signer_alice, amount / 4).await;
    match tx {
        Ok(_) => panic!("burn tx should fail"),
        Err(report) => {
            assert!(report
                .to_string()
                .contains(erc20_pausable_error_selector::ENFORCE_PAUSE));
        }
    }   
    // make sure we leave the contract unpaused
    unpause(token_signer_alice).await.unwrap();
}

#[tokio::test]
async fn transfer_revert_when_paused_works_when_unpaused_test() {
    let fixtures_mutex = init_fixtures().await.unwrap();
    let fixtures = fixtures_mutex.lock().await;

    let alice_address = fixtures.alice_wallet.address();
    let bob_address = fixtures.bob_wallet.address();
    let token_signer_alice = &fixtures.token_signer_alice;
    let amount: U256 = 1000.into();

    mint(token_signer_alice, alice_address, amount)
        .await
        .unwrap();
    // transfer should work here
    transfer(token_signer_alice, bob_address, amount / 4).await.unwrap();

    pause(token_signer_alice).await.unwrap();

    let tx = transfer(token_signer_alice, bob_address, amount / 4).await;
    match tx {
        Ok(_) => panic!("transfer tx should fail"),
        Err(report) => {
            assert!(report
                .to_string()
                .contains(erc20_pausable_error_selector::ENFORCE_PAUSE));
        }
    }   
    // make sure we leave the contract unpaused
    unpause(token_signer_alice).await.unwrap();
}

#[tokio::test]
async fn transfer_from_revert_when_paused_works_when_unpaused_test() {
    let fixtures_mutex = init_fixtures().await.unwrap();
    let fixtures = fixtures_mutex.lock().await;

    let alice_address = fixtures.alice_wallet.address();
    let bob_address = fixtures.bob_wallet.address();
    let token_signer_alice = &fixtures.token_signer_alice;
    let amount: U256 = 1000.into();

    mint(token_signer_alice, alice_address, amount).await.unwrap();
    approve(token_signer_alice, alice_address, amount).await.unwrap();
    // transfer should work here
    transfer_from(token_signer_alice, alice_address, bob_address, amount / 4).await.unwrap();

    pause(token_signer_alice).await.unwrap();

    let tx = transfer_from(token_signer_alice, alice_address, bob_address, amount / 4).await;
    match tx {
        Ok(_) => panic!("transfer_from tx should fail"),
        Err(report) => {
            assert!(report
                .to_string()
                .contains(erc20_pausable_error_selector::ENFORCE_PAUSE));
        }
    }   
    // make sure we leave the contract unpaused
    unpause(token_signer_alice).await.unwrap();
}

#[tokio::test]
async fn burn_from_revert_when_paused_works_when_unpaused_test() {
    let fixtures_mutex = init_fixtures().await.unwrap();
    let fixtures = fixtures_mutex.lock().await;

    let alice_address = fixtures.alice_wallet.address();
    let token_signer_alice = &fixtures.token_signer_alice;
    let amount: U256 = 1000.into();

    mint(token_signer_alice, alice_address, amount).await.unwrap();
    approve(token_signer_alice, alice_address, amount).await.unwrap();
    // burn_from should work here
    burn_from(token_signer_alice, alice_address, amount / 4).await.unwrap();

    pause(token_signer_alice).await.unwrap();

    let tx = burn_from(token_signer_alice, alice_address, amount / 4).await;
    match tx {
        Ok(_) => panic!("burn_from tx should fail"),
        Err(report) => {
            assert!(report
                .to_string()
                .contains(erc20_pausable_error_selector::ENFORCE_PAUSE));
        }
    }   
    // make sure we leave the contract unpaused
    unpause(token_signer_alice).await.unwrap();
}

/*** Erc20 helper functions ***/

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

async fn pause(
    my_token_signer: &MyTokenType
) -> eyre::Result<TransactionReceipt> {
    my_token_signer
        .pause()
        .send()
        .await?
        .await?
        .ok_or(Report::msg("pause tx error"))
}

async fn unpause(
    my_token_signer: &MyTokenType
) -> eyre::Result<TransactionReceipt> {
    my_token_signer
        .unpause()
        .send()
        .await?
        .await?
        .ok_or(Report::msg("unpause tx error"))
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