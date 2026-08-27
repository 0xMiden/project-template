//! Common helper functions for scripts and tests

use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};

use anyhow::{bail, Context, Result};
use miden_client::{
    account::{
        component::{BasicWallet, InitStorageData, NoAuth},
        Account, AccountBuilder, AccountComponent, AccountType, StorageSlotName,
    },
    auth::{Approver, AuthSchemeId, AuthSecretKey, AuthSingleSig},
    builder::ClientBuilder,
    keystore::{FilesystemKeyStore, Keystore},
    rpc::Endpoint,
    utils::Deserializable,
    Client, Felt, Word,
};
use miden_client_sqlite_store::ClientBuilderSqliteExt;
use miden_mast_package::Package;
use rand::Rng;

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
    let endpoint = Endpoint::devnet();
    let timeout_ms = 10_000;

    // Initialize keystore
    let keystore_path = std::path::PathBuf::from("../keystore");

    let keystore =
        Arc::new(FilesystemKeyStore::new(keystore_path).context("Failed to initialize keystore")?);

    let store_path = std::path::PathBuf::from("../store.sqlite3");

    let client = ClientBuilder::new()
        .grpc_client(&endpoint, Some(timeout_ms))
        .sqlite_store(store_path)
        .authenticator(keystore.clone())
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
    const EXPECTED_CARGO_MIDEN_VERSION: &str = "cargo-miden 0.10.0-rc.1";

    let cargo_home = env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cargo")))
        .context("CARGO_HOME and HOME are both unset; cannot locate the v0.16 compiler")?;
    let cargo_miden = cargo_home
        .join("miden-v16-0.10.0-rc.1")
        .join("bin")
        .join("cargo-miden");

    if !cargo_miden.is_absolute() {
        bail!(
            "v0.16 cargo-miden path is not absolute: {}",
            cargo_miden.display()
        );
    }
    if !cargo_miden.is_file() {
        bail!(
            "required v0.16 cargo-miden executable does not exist at {}",
            cargo_miden.display()
        );
    }

    let version_output = Command::new(&cargo_miden)
        .args(["miden", "--version"])
        .output()
        .with_context(|| {
            format!(
                "Failed to execute v0.16 compiler at {}",
                cargo_miden.display()
            )
        })?;
    let version_stdout = String::from_utf8(version_output.stdout)
        .context("cargo-miden version output was not valid UTF-8")?;
    let version_stdout = version_stdout.trim_end_matches(['\r', '\n']);
    let version_stderr = String::from_utf8_lossy(&version_output.stderr);
    if !version_output.status.success()
        || version_stdout != EXPECTED_CARGO_MIDEN_VERSION
        || !version_stderr.is_empty()
    {
        bail!(
            "compiler at {} reported stdout {:?}, stderr {:?}, and status {}; expected exactly {:?}",
            cargo_miden.display(),
            version_stdout,
            version_stderr,
            version_output.status,
            EXPECTED_CARGO_MIDEN_VERSION
        );
    }

    let profile = if release { "--release" } else { "--debug" };
    let project_dir = dir
        .canonicalize()
        .with_context(|| format!("Failed to resolve project directory {}", dir.display()))?;
    let manifest_path = project_dir.join("Cargo.toml");

    let output = Command::new(&cargo_miden)
        .args(["miden", "build", profile, "--manifest-path"])
        .arg(&manifest_path)
        .env("CARGO_MIDEN", &cargo_miden)
        .current_dir(&project_dir)
        .output()
        .context("Failed to compile project")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        bail!(
            "Failed to compile project {} (status {}):\nstdout:\n{}\nstderr:\n{}",
            project_dir.display(),
            output.status,
            stdout,
            stderr
        );
    }

    let artifact_reports = stdout
        .lines()
        .chain(stderr.lines())
        .filter_map(|line| line.trim().strip_prefix("Compiled "))
        .collect::<Vec<_>>();
    let [artifact_report] = artifact_reports.as_slice() else {
        bail!(
            "cargo-miden must report exactly one compiled artifact, but reported {}:\nstdout:\n{}\nstderr:\n{}",
            artifact_reports.len(),
            stdout,
            stderr
        );
    };
    let artifact_path = PathBuf::from(artifact_report);
    let artifact_path = if artifact_path.is_absolute() {
        artifact_path
    } else {
        project_dir.join(artifact_path)
    };
    if !artifact_path.is_file() {
        bail!(
            "cargo-miden reported an artifact that is not a regular file: {}",
            artifact_path.display()
        );
    }

    let package_bytes = std::fs::read(&artifact_path).context(format!(
        "Failed to read compiled package from {}",
        artifact_path.display()
    ))?;

    Package::read_from_bytes(&package_bytes).context("Failed to deserialize package from bytes")
}

/// The fixed key used by the counter contract to store the counter value.
pub const COUNTER_STORAGE_KEY: Word = Word::new([Felt::ZERO, Felt::ZERO, Felt::ZERO, Felt::ONE]);

/// Returns the storage slot name used by the counter account component.
///
/// # Errors
/// Returns an error if the fixed storage slot name is invalid.
pub fn counter_storage_slot() -> Result<StorageSlotName> {
    StorageSlotName::new("counter_account::counter_contract::count_map")
        .context("invalid counter storage slot name")
}

/// Configuration for creating an account with a custom component
pub struct AccountCreationConfig {
    /// The account type to create. This also encodes the
    /// storage visibility (`AccountType::Public` / `AccountType::Private`).
    pub account_type: AccountType,
    /// Initial component storage data keyed by storage slot schema.
    pub init_storage_data: InitStorageData,
}

impl Default for AccountCreationConfig {
    fn default() -> Self {
        Self {
            account_type: AccountType::Public,
            init_storage_data: InitStorageData::default(),
        }
    }
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
    let account_component =
        AccountComponent::from_package(package.as_ref(), &config.init_storage_data)
            .context("Failed to create account component from package")?;

    let mut init_seed = [0_u8; 32];
    client.rng().fill_bytes(&mut init_seed);

    let account = AccountBuilder::new(init_seed)
        .account_type(config.account_type)
        .with_component(account_component)
        .with_component(NoAuth)
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
        .with_component(AuthSingleSig::new(Approver::new(
            key_pair.public_key().to_commitment(),
            AuthSchemeId::Falcon512Poseidon2,
        )))
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
