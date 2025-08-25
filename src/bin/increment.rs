use miden_client::{
    Word,
    account::AccountId,
    builder::ClientBuilder,
    keystore::FilesystemKeyStore,
    rpc::{Endpoint, TonicRpcClient},
};
use miden_objects::account::NetworkId;
use std::{env, fs, path::Path, sync::Arc};
use template::common::{
    create_basic_account, create_library, create_network_note, delete_keystore_and_store,
    wait_for_tx,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    delete_keystore_and_store(None).await;

    let endpoint = Endpoint::testnet();
    let timeout_ms = 10_000;
    let rpc_api = Arc::new(TonicRpcClient::new(&endpoint, timeout_ms));

    let mut client = ClientBuilder::new()
        .rpc(rpc_api)
        .filesystem_keystore("./keystore")
        .in_debug_mode(true)
        .build()
        .await?;

    let keystore = FilesystemKeyStore::new("./keystore".into()).unwrap();

    let sync_summary = client.sync_state().await.unwrap();
    println!("⛓  Latest block: {}", sync_summary.block_num);

    // -------------------------------------------------------------------------
    // STEP 1: Create Basic User Account
    // -------------------------------------------------------------------------
    let (alice_account, _) = create_basic_account(&mut client, keystore).await.unwrap();
    println!(
        "alice account id: {:?}",
        alice_account.id().to_bech32(NetworkId::Testnet)
    );

    // -------------------------------------------------------------------------
    // STEP 2 – Read Counter Contract ID from .env file
    // -------------------------------------------------------------------------
    let counter_contract_id_bech32 = env::var("COUNTER_CONTRACT_ID").expect(
        "COUNTER_CONTRACT_ID not found in .env file. Please run 'cargo run --bin deploy' first.",
    );

    let (_, counter_contract_id) = AccountId::from_bech32(&counter_contract_id_bech32)
        .expect("Invalid COUNTER_CONTRACT_ID format in .env file");

    println!(
        "Using counter contract ID: {}",
        counter_contract_id.to_bech32(NetworkId::Testnet)
    );

    client
        .import_account_by_id(counter_contract_id)
        .await
        .unwrap();

    let account_state = client
        .get_account(counter_contract_id)
        .await?
        .expect("counter contract not found");

    let word: Word = account_state.account().storage().get_item(0)?.into();
    let counter_val = word.get(3).unwrap().as_int();
    println!("🔢 Counter value before tx: {}", counter_val);

    // -------------------------------------------------------------------------
    // STEP 3: Prepare & Create the Network Note
    // -------------------------------------------------------------------------
    let note_code = fs::read_to_string(Path::new("./masm/notes/increment_note.masm")).unwrap();
    let account_code = fs::read_to_string(Path::new("./masm/accounts/counter.masm")).unwrap();

    let library_path = "external_contract::counter_contract";
    let library = create_library(account_code, library_path).unwrap();

    let (_increment_note, tx_id) = create_network_note(
        &mut client,
        note_code.clone(),
        library.clone(),
        alice_account.clone(),
        counter_contract_id,
    )
    .await
    .unwrap();

    println!("increment note tx submitted, waiting for onchain commitment");
    wait_for_tx(&mut client, tx_id).await?;

    // -------------------------------------------------------------------------
    // STEP 4: Validate Updated State
    // -------------------------------------------------------------------------
    delete_keystore_and_store(None).await;

    let mut client = ClientBuilder::new()
        .rpc(Arc::new(TonicRpcClient::new(&endpoint, timeout_ms)))
        .filesystem_keystore("./keystore")
        .in_debug_mode(true)
        .build()
        .await?;

    // Sync to get the latest state after the transaction
    client.sync_state().await.unwrap();

    client
        .import_account_by_id(counter_contract_id)
        .await
        .unwrap();

    let new_account_state = client.get_account(counter_contract_id).await.unwrap();

    if let Some(account) = new_account_state.as_ref() {
        let count: Word = account.account().storage().get_item(0).unwrap().into();
        let val = count.get(3).unwrap().as_int();
        println!("🔢 Counter value after tx: {val}");
    }

    Ok(())
}
