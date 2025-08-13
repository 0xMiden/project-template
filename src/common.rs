use miden_client::{
    Client, ClientError, Felt, Word,
    account::{Account, AccountBuilder, AccountId, AccountStorageMode, AccountType, StorageSlot},
    auth::AuthSecretKey,
    builder::ClientBuilder,
    crypto::SecretKey,
    keystore::FilesystemKeyStore,
    note::{
        Note, NoteAssets, NoteExecutionHint, NoteInputs, NoteMetadata, NoteRecipient,
        NoteRelevance, NoteTag, NoteType,
    },
    rpc::{Endpoint, TonicRpcClient},
    store::{InputNoteRecord, NoteFilter, TransactionFilter},
    transaction::{OutputNote, TransactionId, TransactionRequestBuilder, TransactionStatus},
};
use miden_lib::{account::auth::RpoFalcon512, transaction::TransactionKernel};
use miden_lib::{
    account::{auth, wallets::BasicWallet},
    utils::ScriptBuilder,
};
use miden_objects::{
    account::AccountComponent,
    assembly::{Assembler, DefaultSourceManager, Library, LibraryPath, Module, ModuleKind},
};
use rand::{RngCore, rngs::StdRng};
use serde::de::value::Error;
use std::sync::Arc;
use tokio::time::{Duration, sleep};

/// Counter component for creating counter accounts.
///
/// This component supports all account types and provides a simple counter
/// functionality with a single storage slot initialized to zero.
pub struct Counter {
    initial_value: u64,
}

impl Counter {
    /// Creates a new [`Counter`] component with the specified initial value.
    pub fn new(initial_value: u64) -> Self {
        Self { initial_value }
    }

    /// Creates a new [`Counter`] component with initial value of 0.
    pub fn default() -> Self {
        Self::new(0)
    }
}

impl From<Counter> for AccountComponent {
    fn from(counter: Counter) -> Self {
        let storage_slots = vec![StorageSlot::Value([
            Felt::new(counter.initial_value),
            Felt::new(0),
            Felt::new(0),
            Felt::new(0),
        ])];

        // We need to compile the counter account code
        let account_code = include_str!("../masm/accounts/counter.masm");
        let assembler: Assembler = TransactionKernel::assembler().with_debug_mode(true);

        AccountComponent::compile(account_code.to_string(), assembler, storage_slots)
            .expect(
                "Counter component should satisfy the requirements of a valid account component",
            )
            .with_supports_all_types()
    }
}

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
) -> Result<(Note, TransactionId), Error> {
    let rng = client.rng();
    let serial_num = rng.inner_mut().draw_word();

    let note_script = ScriptBuilder::default()
        .with_dynamically_linked_library(&account_library)
        .unwrap()
        .compile_note_script(note_code)
        .unwrap();
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
    Ok((note, tx_id))
}

/// Creates a private note with the specified parameters and submits it to the network.
///
/// This function compiles the note script, creates a note with private visibility,
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
pub async fn create_private_note(
    client: &mut Client,
    note_code: String,
    account_library: Library,
    creator_account: Account,
    assets: NoteAssets,
) -> Result<Note, Error> {
    let rng = client.rng();
    let serial_num = rng.inner_mut().draw_word();

    let note_script = ScriptBuilder::default()
        .with_dynamically_linked_library(&account_library)
        .unwrap()
        .compile_note_script(note_code)
        .unwrap();
    let note_inputs = NoteInputs::new([].to_vec()).unwrap();
    let recipient = NoteRecipient::new(serial_num, note_script, note_inputs.clone());

    let tag = NoteTag::from_account_id(creator_account.id());
    let metadata = NoteMetadata::new(
        creator_account.id(),
        NoteType::Private,
        tag,
        NoteExecutionHint::none(),
        Felt::new(0),
    )
    .unwrap();

    let note = Note::new(assets, metadata, recipient);

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

    let key_pair = SecretKey::with_rng(client.rng());
    let builder = AccountBuilder::new(init_seed)
        .account_type(AccountType::RegularAccountUpdatableCode)
        .storage_mode(AccountStorageMode::Public)
        .with_auth_component(RpoFalcon512::new(key_pair.public_key()))
        .with_component(BasicWallet);
    let (account, seed) = builder.build().unwrap();
    client.add_account(&account, Some(seed), false).await?;
    keystore
        .add_key(&AuthSecretKey::RpoFalcon512(key_pair.clone()))
        .unwrap();

    Ok((account, key_pair))
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
    _account_code: &str,
) -> Result<(Account, Word), ClientError> {
    let counter_component = Counter::default();

    let mut init_seed = [0_u8; 32];
    client.rng().fill_bytes(&mut init_seed);

    let (counter_contract, counter_seed) = AccountBuilder::new(init_seed)
        .account_type(AccountType::RegularAccountImmutableCode)
        .storage_mode(AccountStorageMode::Network)
        .with_auth_component(auth::NoAuth)
        .with_component(counter_component)
        .build()
        .unwrap();

    Ok((counter_contract, counter_seed))
}

pub async fn create_public_account(
    client: &mut Client,
    _account_code: &str,
) -> Result<(Account, Word), ClientError> {
    let counter_component = Counter::default();

    let mut init_seed = [0_u8; 32];
    client.rng().fill_bytes(&mut init_seed);

    let (counter_contract, counter_seed) = AccountBuilder::new(init_seed)
        .account_type(AccountType::RegularAccountImmutableCode)
        .storage_mode(AccountStorageMode::Public)
        .with_auth_component(auth::NoAuth)
        .with_component(counter_component)
        .build()
        .unwrap();

    Ok((counter_contract, counter_seed))
}

/// Waits for a specific note to become available in the client's state and checks transaction commitment.
///
/// This function continuously polls the client's state until the expected note
/// is found either in the consumable notes or committed notes. It also checks if the
/// associated transaction has been committed. It uses a 2-second polling interval.
///
/// # Arguments
///
/// * `client` - A mutable reference to the Miden client
/// * `account_id` - An optional account to filter consumable notes by
/// * `expected` - A reference to the note we're waiting for
/// * `tx_id` - The transaction ID to check for commitment status
///
/// # Returns
///
/// Returns `Ok(())` when the note is found and transaction is committed, or a `ClientError` if synchronization fails.
///
/// # Behavior
///
/// The function will loop indefinitely until the note is found and the transaction is committed,
/// printing status messages every 2 seconds. It checks both consumable and committed note collections
/// as well as transaction commitment status.
pub async fn wait_for_note(
    client: &mut Client,
    account_id: Option<Account>,
    expected: &Note,
    tx_id: TransactionId,
) -> Result<(), ClientError> {
    loop {
        client.sync_state().await?;

        // Check transaction status
        let txs = client
            .get_transactions(TransactionFilter::Ids(vec![tx_id]))
            .await?;
        let tx_committed = if !txs.is_empty() {
            matches!(txs[0].status, TransactionStatus::Committed(_))
        } else {
            false
        };

        // Notes that can be consumed right now
        let consumable: Vec<(InputNoteRecord, Vec<(AccountId, NoteRelevance)>)> = client
            .get_consumable_notes(account_id.as_ref().map(|acc| acc.id()))
            .await?;

        // Notes submitted that are now committed
        let committed: Vec<InputNoteRecord> = client.get_input_notes(NoteFilter::Committed).await?;

        // Check both vectors
        let note_found = consumable.iter().any(|(rec, _)| rec.id() == expected.id())
            || committed.iter().any(|rec| rec.id() == expected.id());

        if note_found && tx_committed {
            println!(
                "✅ note found {} and transaction committed",
                expected.id().to_hex()
            );
            break;
        }

        if note_found && !tx_committed {
            println!(
                "Note {} found but transaction not yet committed. Waiting...",
                expected.id().to_hex()
            );
        } else if !note_found {
            println!("Note {} not found. Waiting...", expected.id().to_hex());
        }

        sleep(Duration::from_secs(2)).await;
    }

    Ok(())
}

/// Waits for a specific transaction to be committed.
///
/// This function continuously polls the client's state until the specified transaction
/// has been committed. It uses a 2-second polling interval to check for the transaction's
/// commitment status.
///
/// # Arguments
///
/// * `client` - A mutable reference to the Miden client
/// * `tx_id` - The transaction ID to check for commitment status
///
/// # Returns
///
/// Returns `Ok(())` when the transaction is committed, or a `ClientError` if synchronization fails.
///
/// # Behavior
///
/// The function will loop indefinitely until the transaction is committed,
/// printing status messages every 2 seconds.
pub async fn wait_for_tx(client: &mut Client, tx_id: TransactionId) -> Result<(), ClientError> {
    loop {
        client.sync_state().await?;

        // Check transaction status
        let txs = client
            .get_transactions(TransactionFilter::Ids(vec![tx_id]))
            .await?;
        let tx_committed = if !txs.is_empty() {
            matches!(txs[0].status, TransactionStatus::Committed(_))
        } else {
            false
        };

        if tx_committed {
            println!("✅ transaction {} committed", tx_id.to_hex());
            break;
        }

        println!(
            "Transaction {} not yet committed. Waiting...",
            tx_id.to_hex()
        );
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
