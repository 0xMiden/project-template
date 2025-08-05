use miden_client::{
    Client, ClientError, Felt, Word,
    account::{Account, AccountBuilder, AccountId, AccountStorageMode, AccountType, StorageSlot},
    auth::AuthSecretKey,
    builder::ClientBuilder,
    crypto::SecretKey,
    keystore::FilesystemKeyStore,
    note::{
        Note, NoteAssets, NoteExecutionHint, NoteInputs, NoteMetadata, NoteRecipient,
        NoteRelevance, NoteScript, NoteTag, NoteType,
    },
    rpc::{Endpoint, TonicRpcClient},
    store::{InputNoteRecord, NoteFilter},
    transaction::{OutputNote, TransactionRequestBuilder, TransactionScript},
};
use miden_lib::account::wallets::BasicWallet;
use miden_lib::transaction::TransactionKernel;
use miden_objects::{
    account::AccountComponent,
    assembly::{Assembler, DefaultSourceManager, Library, LibraryPath, Module, ModuleKind},
};
use rand::{RngCore, rngs::StdRng};
use serde::de::value::Error;
use std::{fs, path::Path, sync::Arc};

/// Helper to instantiate a `Client` for interacting with Miden.
///
/// # Arguments
///
/// * `endpoint` - The endpoint of the RPC server to connect to.
/// * `store_path` - An optional path to the SQLite store.
///
/// # Returns
///
/// Returns a `Result` containing the `Client` if successful, or a `ClientError` if an error occurs.
pub async fn instantiate_client(
    endpoint: Endpoint,
    store_path: Option<&str>,
) -> Result<Client, ClientError> {
    let timeout_ms = 10_000;
    let rpc_api = Arc::new(TonicRpcClient::new(&endpoint, timeout_ms));

    let client = ClientBuilder::new()
        .rpc(rpc_api.clone())
        .filesystem_keystore("./keystore")
        .sqlite_store(store_path.unwrap_or("./store.sqlite3"))
        .in_debug_mode(true)
        .build()
        .await?;

    Ok(client)
}

/// Creates a public note with the specified parameters and submits it to the network.
///
/// This function compiles the note script, creates a note with public visibility,
/// and submits a transaction containing the note to the Miden network.
///
/// # Arguments
///
/// * `client` - A mutable reference to the Miden client for network operations
/// * `note_code` - The MASM code that defines the note's behavior
/// * `account_library` - The library containing account-related code dependencies
/// * `creator_account` - The account that will create and own this note
/// * `assets` - The assets to be included in the note
///
/// # Returns
///
/// Returns a `Result` containing the created `Note` if successful, or an `Error` if creation fails.
pub async fn create_network_note(
    client: &mut Client,
    note_code: String,
    account_library: Library,
    creator_account: Account,
    counter_contract_id: AccountId,
) -> Result<Note, Error> {
    let assembler = TransactionKernel::assembler()
        .with_library(&account_library)
        .unwrap()
        .with_debug_mode(true);
    let rng = client.rng();
    let serial_num = rng.inner_mut().draw_word();
    let note_script = NoteScript::compile(note_code, assembler.clone()).unwrap();
    let note_inputs = NoteInputs::new([].to_vec()).unwrap();
    let recipient = NoteRecipient::new(serial_num, note_script, note_inputs.clone());

    let tag = NoteTag::from_account_id(counter_contract_id);
    let metadata = NoteMetadata::new(
        creator_account.id(),
        NoteType::Public,
        tag,
        NoteExecutionHint::none(),
        Felt::new(0),
    )
    .unwrap();

    let note = Note::new(NoteAssets::default(), metadata, recipient);

    let note_req = TransactionRequestBuilder::new()
        .own_output_notes(vec![OutputNote::Full(note.clone())])
        .build()
        .unwrap();
    let tx_result = client
        .new_transaction(creator_account.id(), note_req)
        .await
        .unwrap();

    let _ = client.submit_transaction(tx_result.clone()).await;

    let tx_id = tx_result.executed_transaction().id();
    println!(
        "View transaction on MidenScan: https://testnet.midenscan.com/tx/{:?}",
        tx_id
    );
    client.sync_state().await.unwrap();
    Ok(note)
}

/// Creates a basic wallet account with RpoFalcon512 authentication.
///
/// This function generates a new account with updatable code, public storage mode,
/// and basic wallet functionality. The account is automatically added to the client
/// and the authentication key is stored in the provided keystore.
///
/// # Arguments
///
/// * `client` - A mutable reference to the Miden client
/// * `keystore` - The filesystem keystore where the authentication key will be stored
///
/// # Returns
///
/// Returns a `Result` containing a tuple of the created `Account` and its `SecretKey`,
/// or a `ClientError` if account creation fails.
pub async fn create_basic_account(
    client: &mut Client,
    keystore: FilesystemKeyStore<StdRng>,
) -> Result<(miden_client::account::Account, SecretKey), ClientError> {
    let mut init_seed = [0_u8; 32];
    client.rng().fill_bytes(&mut init_seed);

    let assembler = TransactionKernel::assembler().with_debug_mode(true);
    let incr_nonce_code = fs::read_to_string(Path::new("./masm/auth/no_auth.masm")).unwrap();

    let incr_nonce_component = AccountComponent::compile(
        incr_nonce_code.to_string(),
        assembler.clone(),
        vec![StorageSlot::Value([
            Felt::new(0),
            Felt::new(0),
            Felt::new(0),
            Felt::new(0),
        ])],
    )
    .unwrap()
    .with_supports_all_types();

    let key_pair = SecretKey::with_rng(client.rng());
    let builder = AccountBuilder::new(init_seed)
        .account_type(AccountType::RegularAccountUpdatableCode)
        .storage_mode(AccountStorageMode::Public)
        .with_auth_component(incr_nonce_component)
        .with_component(BasicWallet);
    let (account, seed) = builder.build().unwrap();
    client.add_account(&account, Some(seed), false).await?;
    keystore
        .add_key(&AuthSecretKey::RpoFalcon512(key_pair.clone()))
        .unwrap();

    Ok((account, key_pair))
}

/// Creates an account component with no authentication requirements.
///
/// This function reads the no-auth MASM code from the filesystem and compiles it
/// into an account component that supports all transaction types without requiring
/// authentication signatures.
///
/// # Returns
///
/// Returns a `Result` containing the compiled `AccountComponent` if successful,
/// or an `Error` if compilation fails.
///
/// # Note
///
/// This component should only be used for testing or specific use cases where
/// authentication is not required, as it provides no security.
pub async fn create_no_auth_component() -> Result<AccountComponent, Error> {
    let assembler: Assembler = TransactionKernel::assembler().with_debug_mode(true);
    let no_auth_code = fs::read_to_string(Path::new("./masm/auth/no_auth.masm")).unwrap();
    let no_auth_component = AccountComponent::compile(no_auth_code, assembler.clone(), vec![])
        .unwrap()
        .with_supports_all_types();

    Ok(no_auth_component)
}

/// Creates a public immutable network smart contract account from the provided MASM code.
///
/// This function compiles the provided account code into a contract with immutable code,
/// public storage mode, and no authentication requirements. The contract is initialized
/// with a zero-value storage slot.
///
/// # Arguments
///
/// * `client` - A mutable reference to the Miden client
/// * `account_code` - The MASM code that defines the contract's behavior
///
/// # Returns
///
/// Returns a `Result` containing a tuple of the created contract `Account` and its seed `Word`,
/// or a `ClientError` if contract creation fails.
pub async fn create_network_account(
    client: &mut Client,
    account_code: &str,
) -> Result<(Account, Word), ClientError> {
    let assembler: Assembler = TransactionKernel::assembler().with_debug_mode(true);

    let counter_component = AccountComponent::compile(
        account_code.to_string(),
        assembler.clone(),
        vec![StorageSlot::Value([
            Felt::new(0),
            Felt::new(0),
            Felt::new(0),
            Felt::new(0),
        ])],
    )
    .unwrap()
    .with_supports_all_types();

    let mut init_seed = [0_u8; 32];
    client.rng().fill_bytes(&mut init_seed);
    let no_auth_component = create_no_auth_component().await.unwrap();
    let (counter_contract, counter_seed) = AccountBuilder::new(init_seed)
        .account_type(AccountType::RegularAccountImmutableCode)
        .storage_mode(AccountStorageMode::Network)
        .with_auth_component(no_auth_component)
        .with_component(counter_component.clone())
        .build()
        .unwrap();

    Ok((counter_contract, counter_seed))
}

/// Waits for a specific note to become available in the client's state.
///
/// This function continuously polls the client's state until the expected note
/// is found either in the consumable notes or committed notes. It uses a 2-second
/// polling interval to check for the note's availability.
///
/// # Arguments
///
/// * `client` - A mutable reference to the Miden client
/// * `account_id` - An optional account to filter consumable notes by
/// * `expected` - A reference to the note we're waiting for
///
/// # Returns
///
/// Returns `Ok(())` when the note is found, or a `ClientError` if synchronization fails.
///
/// # Behavior
///
/// The function will loop indefinitely until the note is found, printing status
/// messages every 2 seconds. It checks both consumable and committed note collections.
pub async fn wait_for_note(
    client: &mut Client,
    account_id: Option<Account>,
    expected: &Note,
) -> Result<(), ClientError> {
    use tokio::time::{Duration, sleep};

    loop {
        client.sync_state().await?;

        // Notes that can be consumed right now
        let consumable: Vec<(InputNoteRecord, Vec<(AccountId, NoteRelevance)>)> = client
            .get_consumable_notes(account_id.as_ref().map(|acc| acc.id()))
            .await?;

        // Notes submitted that are now committed
        let committed: Vec<InputNoteRecord> = client.get_input_notes(NoteFilter::Committed).await?;

        // Check both vectors
        let found = consumable.iter().any(|(rec, _)| rec.id() == expected.id())
            || committed.iter().any(|rec| rec.id() == expected.id());

        if found {
            println!("✅ note found {}", expected.id().to_hex());
            break;
        }

        println!("Note {} not found. Waiting...", expected.id().to_hex());
        sleep(Duration::from_secs(2)).await;
    }

    Ok(())
}

/// Creates a Miden library from the provided account code and library path.
///
/// # Arguments
///
/// * `account_code` - The account code in MASM format.
/// * `library_path` - The path where the library is located.
///
/// # Returns
///
/// Returns the resulting `Library` if successful, or an error if the library cannot be created.
pub fn create_library(
    account_code: String,
    library_path: &str,
) -> Result<Library, Box<dyn std::error::Error>> {
    let assembler: Assembler = TransactionKernel::assembler().with_debug_mode(true);
    let source_manager = Arc::new(DefaultSourceManager::default());
    let module = Module::parser(ModuleKind::Library).parse_str(
        LibraryPath::new(library_path)?,
        account_code,
        &source_manager,
    )?;
    let library = assembler.clone().assemble_library([module])?;
    Ok(library)
}

/// Creates a transaction script based on the provided code and optional library.
///
/// # Arguments
///
/// * `script_code` - The code for the transaction script, typically written in MASM.
/// * `library` - An optional library to use with the script.
///
/// # Returns
///
/// Returns a `TransactionScript` if successfully created, or an error.
pub fn create_tx_script(
    script_code: String,
    library: Option<Library>,
) -> Result<TransactionScript, Box<dyn std::error::Error>> {
    let assembler = TransactionKernel::assembler();

    let assembler = match library {
        Some(lib) => assembler.with_library(lib)?,
        None => assembler.with_debug_mode(true),
    };

    let tx_script = TransactionScript::compile(script_code, assembler)?;
    Ok(tx_script)
}

/// Deletes the keystore and store files.
///
/// # Arguments
///
/// * `store_path` - An optional path to the SQLite store that should be deleted. Defaults to `./store.sqlite3` if not provided.
///
/// This function removes all files from the keystore and deletes the SQLite store file, if they exist.
pub async fn delete_keystore_and_store(store_path: Option<&str>) {
    let store_path = store_path.unwrap_or("./store.sqlite3");
    if tokio::fs::metadata(store_path).await.is_ok() {
        if let Err(e) = tokio::fs::remove_file(store_path).await {
            eprintln!("failed to remove {}: {}", store_path, e);
        } else {
            println!("cleared sqlite store: {}", store_path);
        }
    } else {
        println!("store not found: {}", store_path);
    }

    let keystore_dir = "./keystore";
    match tokio::fs::read_dir(keystore_dir).await {
        Ok(mut dir) => {
            while let Ok(Some(entry)) = dir.next_entry().await {
                let file_path = entry.path();
                if let Err(e) = tokio::fs::remove_file(&file_path).await {
                    eprintln!("failed to remove {}: {}", file_path.display(), e);
                } else {
                    println!("removed file: {}", file_path.display());
                }
            }
        }
        Err(e) => eprintln!("failed to read directory {}: {}", keystore_dir, e),
    }
}
