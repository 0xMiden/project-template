//! Common helper functions for scripts and tests

use std::{borrow::Borrow, path::Path, sync::Arc};

use anyhow::{bail, Context, Result};
use cargo_miden::{run, OutputType};
use miden_client::{
    account::{
        component::{AccountComponentMetadata, BasicWallet, NoAuth},
        Account, AccountBuilder, AccountComponent, AccountStorageMode, AccountType, StorageSlot,
    },
    auth::{AuthSchemeId, AuthSecretKey, AuthSingleSig},
    builder::ClientBuilder,
    keystore::{FilesystemKeyStore, Keystore},
    rpc::{Endpoint, GrpcClient},
    utils::Deserializable,
    Client,
};
use miden_client_sqlite_store::ClientBuilderSqliteExt;
use miden_mast_package::{Package, SectionId};
use rand::RngCore;

/// Test setup configuration containing initialized client and keystore
pub struct ClientSetup {
    /// The configured Miden client instance.
    pub client: Client<FilesystemKeyStore>,
    /// The filesystem-backed keystore used by the client.
    pub keystore: Arc<FilesystemKeyStore>,
}

/// Initializes test infrastructure with client and keystore
///
/// # Returns
/// A `ClientSetup` containing the initialized client and keystore
///
/// # Errors
/// Returns an error if RPC connection fails, keystore initialization fails,
/// or client building fails
pub async fn setup_client() -> Result<ClientSetup> {
    // Initialize RPC connection
    let endpoint = Endpoint::testnet();
    let timeout_ms = 10_000;
    let rpc_client = Arc::new(GrpcClient::new(&endpoint, timeout_ms));

    // Initialize keystore
    let keystore_path = std::path::PathBuf::from("../keystore");

    let keystore =
        Arc::new(FilesystemKeyStore::new(keystore_path).context("Failed to initialize keystore")?);

    let store_path = std::path::PathBuf::from("../store.sqlite3");

    let client = ClientBuilder::new()
        .rpc(rpc_client)
        .sqlite_store(store_path)
        .authenticator(keystore.clone())
        .in_debug_mode(true.into())
        .build()
        .await
        .context("Failed to build Miden client")?;

    Ok(ClientSetup { client, keystore })
}

/// Builds a Miden project in the specified directory
///
/// # Arguments
/// * `dir` - Path to the directory containing the Cargo.toml
/// * `release` - Whether to build in release mode
///
/// # Returns
/// The compiled `Package`
///
/// # Errors
/// Returns an error if compilation fails or if the output is not in the expected format
pub fn build_project_in_dir(dir: &Path, release: bool) -> Result<Package> {
    let profile = if release { "--release" } else { "--debug" };
    let manifest_path = dir.join("Cargo.toml");
    let manifest_arg = manifest_path.to_string_lossy();

    let args = vec![
        "cargo",
        "miden",
        "build",
        profile,
        "--manifest-path",
        &manifest_arg,
    ];

    let output = run(args.into_iter().map(String::from), OutputType::Masm)
        .context("Failed to compile project")?
        .context("Cargo miden build returned None")?;

    let artifact_path = match output {
        cargo_miden::CommandOutput::BuildCommandOutput { output } => match output {
            cargo_miden::BuildOutput::Masm { artifact_path } => artifact_path,
            other => bail!("Expected Masm output, got {:?}", other),
        },
        other => bail!("Expected BuildCommandOutput, got {:?}", other),
    };

    let package_bytes = std::fs::read(&artifact_path).context(format!(
        "Failed to read compiled package from {}",
        artifact_path.display()
    ))?;

    Package::read_from_bytes(&package_bytes).context("Failed to deserialize package from bytes")
}

/// Configuration for creating an account with a custom component
#[derive(Clone)]
pub struct AccountCreationConfig {
    /// The account type to create.
    pub account_type: AccountType,
    /// The account storage visibility mode.
    pub storage_mode: AccountStorageMode,
    /// Initial storage slots to seed into the component.
    pub storage_slots: Vec<StorageSlot>,
}

impl Default for AccountCreationConfig {
    fn default() -> Self {
        Self {
            account_type: AccountType::RegularAccountImmutableCode,
            storage_mode: AccountStorageMode::Public,
            storage_slots: vec![],
        }
    }
}

/// Creates an account component from a compiled package
///
/// # Arguments
/// * `package` - The compiled package containing account component metadata
/// * `config` - Configuration for account creation
///
/// # Returns
/// An `AccountComponent` configured according to the provided config
///
/// # Errors
/// Returns an error if the package doesn't contain account component metadata or deserialization fails
pub fn account_component_from_package(
    package: Arc<Package>,
    config: &AccountCreationConfig,
) -> Result<AccountComponent> {
    // Find the account component metadata section in the package
    let account_component_metadata = package.sections.iter().find_map(|s| {
        if s.id == SectionId::ACCOUNT_COMPONENT_METADATA {
            Some(s.data.borrow())
        } else {
            None
        }
    });

    let account_component = match account_component_metadata {
        None => bail!("Package missing account component metadata"),
        Some(bytes) => {
            let metadata = AccountComponentMetadata::read_from_bytes(bytes)
                .context("Failed to deserialize account component metadata")?;

            let component = AccountComponent::new(
                package.mast.as_ref().clone(),
                config.storage_slots.clone(),
                metadata,
            )
            .context("Failed to create account component")?;
            component
        }
    };

    Ok(account_component)
}

/// Creates an account with a custom component from a compiled package
///
/// # Arguments
/// * `client` - The Miden client instance
/// * `package` - The compiled package containing the account component
/// * `config` - Configuration for account creation
///
/// # Returns
/// The created `Account`
///
/// # Errors
/// Returns an error if account creation or client operations fail
pub async fn create_account_from_package(
    client: &mut Client<FilesystemKeyStore>,
    package: Arc<Package>,
    config: AccountCreationConfig,
) -> Result<Account> {
    let account_component = account_component_from_package(package, &config)
        .context("Failed to create account component from package")?;

    let mut init_seed = [0_u8; 32];
    client.rng().fill_bytes(&mut init_seed);

    let account = AccountBuilder::new(init_seed)
        .account_type(config.account_type)
        .storage_mode(config.storage_mode)
        .with_component(account_component)
        .with_auth_component(NoAuth)
        .build()
        .context("Failed to build account")?;

    println!("Account ID: {:?}", account.id());

    client
        .add_account(&account, false)
        .await
        .context("Failed to add account to client")?;

    Ok(account)
}

/// Creates a basic wallet account with authentication
///
/// # Arguments
/// * `client` - The Miden client instance
/// * `keystore` - The keystore for storing authentication keys
/// * `config` - Configuration for account creation
///
/// # Returns
/// The created `Account` with basic wallet functionality
///
/// # Errors
/// Returns an error if account creation, key generation, or keystore operations fail
pub async fn create_basic_wallet_account(
    client: &mut Client<FilesystemKeyStore>,
    keystore: Arc<FilesystemKeyStore>,
    config: AccountCreationConfig,
) -> Result<Account> {
    let mut init_seed = [0_u8; 32];
    client.rng().fill_bytes(&mut init_seed);

    let key_pair = AuthSecretKey::new_falcon512_poseidon2_with_rng(client.rng());

    let builder = AccountBuilder::new(init_seed)
        .account_type(config.account_type)
        .storage_mode(config.storage_mode)
        .with_auth_component(AuthSingleSig::new(
            key_pair.public_key().to_commitment(),
            AuthSchemeId::Falcon512Poseidon2,
        ))
        .with_component(BasicWallet);

    let account = builder
        .build()
        .context("Failed to build basic wallet account")?;

    client
        .add_account(&account, false)
        .await
        .context("Failed to add account to client")?;

    keystore
        .add_key(&key_pair, account.id())
        .await
        .context("Failed to add key to keystore")?;

    Ok(account)
}
