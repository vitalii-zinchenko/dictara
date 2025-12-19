use keyring::Entry;

const SERVICE: &str = "zinjvi.typefree";

// Account names for different providers
const OPENAI_ACCOUNT: &str = "openai_api_key";
const AZURE_ACCOUNT: &str = "azure_api_key";

pub enum KeychainAccount {
    OpenAI,
    Azure,
}

impl KeychainAccount {
    fn as_str(&self) -> &str {
        match self {
            KeychainAccount::OpenAI => OPENAI_ACCOUNT,
            KeychainAccount::Azure => AZURE_ACCOUNT,
        }
    }
}

pub fn save_api_key(account: KeychainAccount, key: &str) -> Result<(), keyring::Error> {
    let account_name = account.as_str();
    let entry = Entry::new(SERVICE, account_name)?;

    match entry.set_password(key) {
        Ok(()) => {
            println!(
                "[Keychain] ✅ API key saved successfully to macOS Keychain ({})",
                account_name
            );
            Ok(())
        }
        Err(e) => {
            eprintln!(
                "[Keychain] ❌ Failed to save API key ({}): {:?}",
                account_name, e
            );
            Err(e)
        }
    }
}

pub fn load_api_key(account: KeychainAccount) -> Result<Option<String>, keyring::Error> {
    let account_name = account.as_str();
    println!("[Keychain] Attempting to load API key ({})", account_name);
    println!(
        "[Keychain] Service: '{}', Account: '{}'",
        SERVICE, account_name
    );

    let entry = Entry::new(SERVICE, account_name)?;

    match entry.get_password() {
        Ok(password) => {
            println!(
                "[Keychain] ✅ API key loaded successfully (length: {}, account: {})",
                password.len(),
                account_name
            );
            Ok(Some(password))
        }
        Err(keyring::Error::NoEntry) => {
            println!(
                "[Keychain] ℹ️  No API key found in keychain ({})",
                account_name
            );
            Ok(None)
        }
        Err(e) => {
            eprintln!(
                "[Keychain] ❌ Error loading API key ({}): {:?}",
                account_name, e
            );
            Err(e)
        }
    }
}

pub fn delete_api_key(account: KeychainAccount) -> Result<(), keyring::Error> {
    let account_name = account.as_str();
    println!("[Keychain] Attempting to delete API key ({})", account_name);

    let entry = Entry::new(SERVICE, account_name)?;

    match entry.delete_credential() {
        Ok(()) => {
            println!(
                "[Keychain] ✅ API key deleted successfully ({})",
                account_name
            );
            Ok(())
        }
        Err(keyring::Error::NoEntry) => {
            println!(
                "[Keychain] ℹ️  No API key to delete (not found, {})",
                account_name
            );
            Ok(())
        }
        Err(e) => {
            eprintln!(
                "[Keychain] ❌ Error deleting API key ({}): {:?}",
                account_name, e
            );
            Err(e)
        }
    }
}
