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
        function init(uint256) external
        function setCap(uint256) external
        function mint(address account, uint256 amount) external
        function totalSupply() external view returns (uint256)
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
pub mod erc20_cap_error_selector {
    pub const EXCEEDED_CAP: &str = "0x9e79f854";
    pub const INVALID_CAP: &str = "0x392e1e27";
    pub const ALREADY_INITIALIZED: &str = "0x0dc149f0";
}

static FIXTURES: OnceCell<Mutex<Fixtures>> = OnceCell::const_new();

#[tokio::test]
async fn init_revert_run_more_than_once_test() {
    let fixtures_mutex = init_fixtures().await.unwrap();
    let fixtures = fixtures_mutex.lock().await;

    let token_signer_alice = &fixtures.token_signer_alice;
    let amount: U256 = 1000.into();

    // we run init to make sure, there are at least two calls to it
    let _ = init(token_signer_alice, amount).await;

    // second inti must fail
    let tx = init(token_signer_alice, amount).await;
    match tx {
        Ok(_) => panic!("init tx should fail"),
        Err(report) => {
            assert!(report.to_string().contains(erc20_cap_error_selector::ALREADY_INITIALIZED));
        }
    }
}



#[tokio::test]
async fn mint_revert_over_cap_test() {
    let fixtures_mutex = init_fixtures().await.unwrap();
    let fixtures = fixtures_mutex.lock().await;

    let alice_address = fixtures.alice_wallet.address();
    let token_signer_alice = &fixtures.token_signer_alice;
    let amount: U256 = 1000.into();

    let total_supply = token_signer_alice.total_supply().call().await.unwrap();
    // we need to set cap to total_supply + amount as this will not always be a fresh deployment
    set_cap(token_signer_alice, total_supply + amount).await.unwrap();

    // first mint should work
    mint(token_signer_alice, alice_address, amount)
        .await
        .unwrap();

    // second mint trying to increase supply over amount should fail    
    let tx = mint(token_signer_alice, alice_address, amount).await;
    match tx {
        Ok(_) => panic!("mint tx should fail"),
        Err(report) => {
            assert!(report
                .to_string()
                .contains(erc20_cap_error_selector::EXCEEDED_CAP));
        }
    }
    // set cap for MAX value so other tests can run
    set_cap(token_signer_alice, U256::MAX).await.unwrap();
}

#[tokio::test]
async fn set_cap_revert_when_0_test() {
    let fixtures_mutex = init_fixtures().await.unwrap();
    let fixtures = fixtures_mutex.lock().await;

    let token_signer_alice = &fixtures.token_signer_alice;

    let tx = set_cap(token_signer_alice, 0.into()).await;
    match tx {
        Ok(_) => panic!("set_cap tx should fail"),
        Err(report) => {
            assert!(report
                .to_string()
                .contains(erc20_cap_error_selector::INVALID_CAP));
        }
    }
}

/*** Erc20 helper functions ***/

async fn init(
    my_token_signer: &MyTokenType,
    amount: U256,
) -> eyre::Result<TransactionReceipt> {
    my_token_signer
        .init(amount)
        .send()
        .await?
        .await?
        .ok_or(Report::msg("init tx error"))
}

async fn set_cap(
    my_token_signer: &MyTokenType,
    amount: U256,
) -> eyre::Result<TransactionReceipt> {
    my_token_signer
        .set_cap(amount)
        .send()
        .await?
        .await?
        .ok_or(Report::msg("set_cap tx error"))
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