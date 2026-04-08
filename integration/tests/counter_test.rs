use anyhow::Context;
use integration::helpers::{
    build_project_in_dir, create_testing_account_from_package, create_testing_note_from_package,
    AccountCreationConfig, NoteCreationConfig,
};

use miden_client::{
    account::{
        component::InitStorageData, StorageMap, StorageMapKey, StorageSlot, StorageSlotName,
    },
    auth::AuthSchemeId,
    transaction::RawOutputNote,
    Felt, Word,
};
use miden_testing::{Auth, MockChain};
use std::{collections::BTreeMap, path::Path, sync::Arc};

#[tokio::test]
async fn counter_test() -> anyhow::Result<()> {
    // Test that after executing the increment note, the counter value is incremented by 1
    let mut builder = MockChain::builder();

    // Crate note sender account
    let sender = builder.add_existing_wallet(Auth::BasicAuth {
        auth_scheme: AuthSchemeId::Falcon512Poseidon2,
    })?;

    // Build contracts
    let contract_package = Arc::new(build_project_in_dir(
        Path::new("../contracts/counter-account"),
        true,
    )?);
    let note_package = Arc::new(build_project_in_dir(
        Path::new("../contracts/increment-note"),
        true,
    )?);

    // Create the counter account with initial storage and no-auth auth component
    let count_storage_key = Word::from([Felt::new(0), Felt::new(0), Felt::new(0), Felt::new(1)]);
    let initial_count = Word::from([Felt::new(0), Felt::new(0), Felt::new(0), Felt::new(0)]);

    // The slot name is constructed as
    // `miden::component::[to_underscore(Cargo.toml:package.metadata.component.package)]::[field_name]`
    let counter_storage_slot =
        StorageSlotName::new("miden_counter_account::counter_contract::count_map").unwrap();
    let storage_slots = vec![StorageSlot::with_map(
        counter_storage_slot.clone(),
        StorageMap::with_entries([(StorageMapKey::new(count_storage_key), initial_count)]).unwrap(),
    )];
    let counter_cfg = AccountCreationConfig {
        storage_slots,
        ..Default::default()
    };

    let map_entries = BTreeMap::from([(
        counter_storage_slot.clone(),
        vec![(count_storage_key.into(), initial_count.into())],
    )]);
    let inital_storage_data = InitStorageData::new(BTreeMap::default(), map_entries)
        .context("Failed to create initial storage data")?;
    // create testing counter account
    let mut counter_account = create_testing_account_from_package(
        contract_package.clone(),
        counter_cfg,
        inital_storage_data,
    )
    .await?;

    // create testing increment note
    let counter_note = create_testing_note_from_package(
        note_package.clone(),
        sender.id(),
        NoteCreationConfig::default(),
    )?;

    // add counter account and note to mockchain
    builder.add_account(counter_account.clone())?;
    builder.add_output_note(RawOutputNote::Full(counter_note.clone()));

    // Build the mock chain
    let mut mock_chain = builder.build()?;
    // Build the transaction context
    let tx_context = mock_chain
        .build_tx_context(counter_account.id(), &[counter_note.id()], &[])?
        .build()?;

    // Execute the transaction
    let executed_transaction = tx_context.execute().await?;

    // Apply the account delta to the counter account
    counter_account.apply_delta(executed_transaction.account_delta())?;

    // Add the executed transaction to the mockchain
    mock_chain.add_pending_executed_transaction(&executed_transaction)?;
    mock_chain.prove_next_block()?;

    // Get the count from the updated counter account
    let count = counter_account
        .storage()
        .get_map_item(&counter_storage_slot, count_storage_key)
        .expect("Failed to get counter value from storage slot");

    // Assert that the count value is equal to 1 after executing the transaction
    assert_eq!(
        count,
        Word::from([Felt::new(0), Felt::new(0), Felt::new(0), Felt::new(1)]),
        "Count value is not equal to 1"
    );

    println!("Test passed!");
    Ok(())
}
