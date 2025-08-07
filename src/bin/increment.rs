use miden_client::{Word, account::AccountId, keystore::FilesystemKeyStore, rpc::Endpoint};
use miden_objects::account::NetworkId;
use std::{fs, path::Path};
use template::common::{
    create_basic_account, create_library, create_network_note, delete_keystore_and_store,
    instantiate_client, wait_for_tx,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    delete_keystore_and_store(None).await;

    // -------------------------------------------------------------------------
    // Instantiate client
    // -------------------------------------------------------------------------
    let endpoint = Endpoint::testnet();
    let mut client = instantiate_client(endpoint.clone(), None).await.unwrap();
    let keystore = FilesystemKeyStore::new("./keystore".into()).unwrap();

    let sync_summary = client.sync_state().await.unwrap();
    println!("⛓  Latest block: {}", sync_summary.block_num);

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
    // STEP 2 – Query Counter State
    // -------------------------------------------------------------------------
    let (_network_id, counter_contract_id) =
        AccountId::from_bech32("mtst1qr00n0fx70uaxsq4mxtf6csmduz0flwt").unwrap();

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

    let mut client = instantiate_client(endpoint, None).await.unwrap();

    client
        .import_account_by_id(counter_contract_id)
        .await
        .unwrap();

    let new_account_state = client.get_account(counter_contract_id).await.unwrap();

    if let Some(account) = new_account_state.as_ref() {
        let count: Word = account.account().storage().get_item(0).unwrap().into();
        let val = count.get(3).unwrap().as_int();
        println!("Counter count: {val}");
    }

    Ok(())
}
