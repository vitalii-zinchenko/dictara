use keyring::Entry;
use serde::{de::DeserializeOwned, Serialize};

#[cfg(debug_assertions)]
const SERVICE: &str = "app.dictara.dev";

#[cfg(not(debug_assertions))]
const SERVICE: &str = "app.dictara";

// Account names for provider configurations
const OPENAI_CONFIG_ACCOUNT: &str = "provider:openai";
const AZURE_OPENAI_CONFIG_ACCOUNT: &str = "provider:azure_openai";

pub enum ProviderAccount {
    OpenAI,
    AzureOpenAI,
}

impl ProviderAccount {
    fn as_str(&self) -> &str {
        match self {
            ProviderAccount::OpenAI => OPENAI_CONFIG_ACCOUNT,
            ProviderAccount::AzureOpenAI => AZURE_OPENAI_CONFIG_ACCOUNT,
        }
    }
}

/// Save provider configuration as JSON to keychain
pub fn save_provider_config<T: Serialize>(
    account: ProviderAccount,
    config: &T,
) -> Result<(), keyring::Error> {
    let account_name = account.as_str();
    let entry = Entry::new(SERVICE, account_name)?;

    let json = serde_json::to_string(config).map_err(|e| {
        eprintln!(
            "[Keychain] ❌ Failed to serialize config ({}): {:?}",
            account_name, e
        );
        keyring::Error::Invalid("config".to_string(), format!("Failed to serialize: {}", e))
    })?;

    match entry.set_password(&json) {
        Ok(()) => {
            println!(
                "[Keychain] ✅ Config saved successfully to macOS Keychain ({})",
                account_name
            );
            Ok(())
        }
        Err(e) => {
            eprintln!(
                "[Keychain] ❌ Failed to save config ({}): {:?}",
                account_name, e
            );
            Err(e)
        }
    }
}

/// Load provider configuration from keychain as JSON
pub fn load_provider_config<T: DeserializeOwned>(
    account: ProviderAccount,
) -> Result<Option<T>, keyring::Error> {
    let account_name = account.as_str();
    println!("[Keychain] Attempting to load config ({})", account_name);

    let entry = Entry::new(SERVICE, account_name)?;

    match entry.get_password() {
        Ok(json) => {
            println!(
                "[Keychain] ✅ Config loaded successfully (length: {}, account: {})",
                json.len(),
                account_name
            );

            let config: T = serde_json::from_str(&json).map_err(|e| {
                eprintln!(
                    "[Keychain] ❌ Failed to deserialize config ({}): {:?}",
                    account_name, e
                );
                keyring::Error::Invalid(
                    "config".to_string(),
                    format!("Failed to deserialize: {}", e),
                )
            })?;

            Ok(Some(config))
        }
        Err(keyring::Error::NoEntry) => {
            println!(
                "[Keychain] ℹ️  No config found in keychain ({})",
                account_name
            );
            Ok(None)
        }
        Err(e) => {
            eprintln!(
                "[Keychain] ❌ Error loading config ({}): {:?}",
                account_name, e
            );
            Err(e)
        }
    }
}

/// Delete provider configuration from keychain
pub fn delete_provider_config(account: ProviderAccount) -> Result<(), keyring::Error> {
    let account_name = account.as_str();
    println!("[Keychain] Attempting to delete config ({})", account_name);

    let entry = Entry::new(SERVICE, account_name)?;

    match entry.delete_credential() {
        Ok(()) => {
            println!(
                "[Keychain] ✅ Config deleted successfully ({})",
                account_name
            );
            Ok(())
        }
        Err(keyring::Error::NoEntry) => {
            println!(
                "[Keychain] ℹ️  No config to delete (not found, {})",
                account_name
            );
            Ok(())
        }
        Err(e) => {
            eprintln!(
                "[Keychain] ❌ Error deleting config ({}): {:?}",
                account_name, e
            );
            Err(e)
        }
    }
}
