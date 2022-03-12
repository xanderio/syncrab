mod command;
mod config;
mod handler;

use color_eyre::{eyre::eyre, Result};
use config::CONFIG;
use matrix_sdk::{
    config::{ClientConfig, SyncSettings},
    ruma::UserId,
    Client,
};
use std::env;
use tokio::fs;
use tracing_subscriber::EnvFilter;

async fn restore_or_login(user_id: &UserId, client: &Client) -> Result<()> {
    if fs::File::open(CONFIG.store.session_file()).await.is_err() {
        // there is no session to reuse, create a new one and save it
        tracing::info!("no session found, login in as {user_id}");
        client
            .login(user_id, &env::var("PASSWORD")?, None, None)
            .await?;

        let session = client
            .session()
            .await
            .ok_or_else(|| eyre!("no session in client after login"))?;
        fs::write(
            CONFIG.store.session_file(),
            serde_json::to_vec_pretty(&session)?,
        )
        .await?;
    } else {
        // restore client from the save session
        tracing::info!("restoring login from session");
        let buf = fs::read(CONFIG.store.session_file()).await?;
        let session = serde_json::from_slice(&buf)?;
        client.restore_login(session).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "syncrab=debug");
    }
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .pretty()
        .init();

    let store_config = matrix_sdk::store::make_store_config(
        &CONFIG.store.location,
        CONFIG.store.password.as_deref(),
    )
    .map_err(|err| eyre!(err))?;

    let client_config = ClientConfig::new().store_config(store_config);
    let client = Client::with_config(CONFIG.synapse.url.clone(), client_config).await?;

    restore_or_login(&CONFIG.synapse.user, &client).await?;

    // An initial sync to set up state and so our bot doesn't respond to old
    // messages. If the `StateStore` finds saved state in the location given the
    // initial sync will be skipped in favor of loading state from the store
    client.sync_once(SyncSettings::default()).await?;

    client
        .register_event_handler(handler::on_stripped_state_member)
        .await;

    client
        .register_event_handler(handler::on_room_message)
        .await;

    client.sync(SyncSettings::default()).await;

    Ok(())
}
