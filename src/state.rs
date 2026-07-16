use std::path::Path;
use tokio::fs;


const STATE_DIR: &'static str = "./state/";
const LAST_SYNC_TIMESTAMP_FILE: &'static str = "last_sync_timestamp"; 

pub async fn create_state_dir_if_absent() -> Result<(), std::io::Error> {
    let path = Path::new(STATE_DIR);
    if !path.exists() {
        std::fs::create_dir(path)?;
    }

    return Ok(());
}


pub async fn update_last_sync_timestamp() -> Result<(), std::io::Error> {
    fs::write(
        STATE_DIR.to_string() + LAST_SYNC_TIMESTAMP_FILE,
        chrono::Utc::now().to_rfc3339().as_bytes()
    ).await?;
    return Ok(());
}

pub fn get_last_sync_timestamp() -> Result<String, std::io::Error> {
    return Ok(std::fs::read_to_string(STATE_DIR.to_string() + LAST_SYNC_TIMESTAMP_FILE)?);
}
