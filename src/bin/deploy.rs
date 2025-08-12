use std::{fs, path::Path};

use template::common::{
    create_basic_account, create_library, create_network_account, create_network_note,
    delete_keystore_and_store, instantiate_client, wait_for_tx,
};

use miden_client::{
    Word, keystore::FilesystemKeyStore, rpc::Endpoint, transaction::TransactionRequestBuilder,
};
use miden_lib::utils::ScriptBuilder;
use miden_objects::account::NetworkId;
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    delete_keystore_and_store(None).await;

    let endpoint = Endpoint::testnet();
    let mut client = instantiate_client(endpoint.clone(), None).await.unwrap();

    let keystore = FilesystemKeyStore::new("./keystore".into()).unwrap();

    let sync_summary = client.sync_state().await.unwrap();
    println!("Latest block: {}", sync_summary.block_num);

    // -------------------------------------------------------------------------
    // STEP 1: Create Basic User Account
    // -------------------------------------------------------------------------
    let (alice_account, _) = create_basic_account(&mut client, keystore.clone())
        .await
        .unwrap();
    println!(
        "alice account id: {:?}",
        alice_account.id().to_bech32(NetworkId::Testnet)
    );

    // -------------------------------------------------------------------------
    // STEP 2: Create Counter Smart Contract
    // -------------------------------------------------------------------------
    let counter_code = fs::read_to_string(Path::new("./masm/accounts/counter.masm")).unwrap();

    let (counter_contract, counter_seed) = create_network_account(&mut client, &counter_code)
        .await
        .unwrap();
    println!(
        "contract id: {:?}",
        counter_contract.id().to_bech32(NetworkId::Testnet)
    );

    // Save the counter contract ID to .env file
    let env_content = format!(
        "COUNTER_CONTRACT_ID={}",
        counter_contract.id().to_bech32(NetworkId::Testnet)
    );
    fs::write(".env", env_content).expect("Failed to write .env file");
    println!("Counter contract ID saved to .env file");

    client
        .add_account(&counter_contract, Some(counter_seed), false)
        .await
        .unwrap();

    // -------------------------------------------------------------------------
    // STEP 3: Deploy Network Account
    // -------------------------------------------------------------------------
    let script_code =
        fs::read_to_string(Path::new("./masm/scripts/increment_script.masm")).unwrap();

    let account_code = fs::read_to_string(Path::new("./masm/accounts/counter.masm")).unwrap();
    let library_path = "external_contract::counter_contract";

    let library = create_library(account_code, library_path).unwrap();

    let tx_script = ScriptBuilder::default()
        .with_dynamically_linked_library(&library)
        .unwrap()
        .compile_tx_script(script_code)
        .unwrap();

    let tx_increment_request = TransactionRequestBuilder::new()
        .custom_script(tx_script)
        .build()
        .unwrap();

    let tx_result = client
        .new_transaction(counter_contract.id(), tx_increment_request)
        .await
        .unwrap();

    let _ = client.submit_transaction(tx_result.clone()).await;

    let tx_id = tx_result.executed_transaction().id();
    println!(
        "View transaction on MidenScan: https://testnet.midenscan.com/tx/{:?}",
        tx_id
    );

    // Wait for the transaction to be committed
    wait_for_tx(&mut client, tx_id).await.unwrap();

    // Wait for network note to be consumed
    sleep(Duration::from_secs(5)).await;

    // -------------------------------------------------------------------------
    // STEP 4: Validate Updated State
    // -------------------------------------------------------------------------

    delete_keystore_and_store(None).await;

    let mut client = instantiate_client(endpoint, None).await.unwrap();

    client
        .import_account_by_id(counter_contract.id())
        .await
        .unwrap();

    let new_account_state = client.get_account(counter_contract.id()).await.unwrap();

    if let Some(account) = new_account_state.as_ref() {
        let count: Word = account.account().storage().get_item(0).unwrap().into();
        let val = count.get(3).unwrap().as_int();
        assert_eq!(val, 1);
        println!("counter contract count state: {:?}", val);
    }

    Ok(())
}
