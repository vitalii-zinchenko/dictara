use keyring::Entry;
use serde::{de::DeserializeOwned, Serialize};

use crate::error;

#[cfg(debug_assertions)]
const BUNDLE: &str = "app.dictara.dev";

#[cfg(not(debug_assertions))]
const BUNDLE: &str = "app.dictara";

#[derive(strum::AsRefStr, strum::EnumString)]
#[strum(serialize_all = "camelCase")]
pub enum ProviderAccount {
    OpenAI,
    AzureOpenAI,
}

/// Save provider configuration as JSON to keychain
pub fn save_provider_config<T: Serialize>(
    account: ProviderAccount,
    config: &T,
) -> Result<(), error::Error> {
    let account_name = account.as_ref();
    let entry = Entry::new(BUNDLE, account_name)?;
    let json = serde_json::to_string(config)?;
    entry.set_password(&json)?;
    Ok(())
}

/// Load provider configuration from keychain as JSON
pub fn load_provider_config<T: DeserializeOwned>(
    account: ProviderAccount,
) -> Result<Option<T>, error::Error> {
    let account_name = account.as_ref();
    let entry = Entry::new(BUNDLE, account_name)?;

    match entry.get_password() {
        Ok(json) => {
            let config: T = serde_json::from_str(&json)?;
            Ok(Some(config))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Delete provider configuration from keychain
pub fn delete_provider_config(account: ProviderAccount) -> Result<(), error::Error> {
    let account_name = account.as_ref();
    let entry = Entry::new(BUNDLE, account_name)?;

    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
