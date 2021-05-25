use std::sync::Arc;

use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use tokio::sync::RwLock;

use crate::config::Config;
use crate::database::entity::chatter::Chatter;
use crate::database::entity::chat_log_message::ChatLogMessage;

pub mod entity;

pub async fn connect_db(config: Arc<RwLock<Config>>) -> anyhow::Result<PgPool> {
    let config = config.read().await;

    let connect_options = PgConnectOptions::new()
        .host(config.app_config.database.host.as_str())
        .port(config.app_config.database.port)
        .username(config.app_config.database.username.as_str())
        .password(config.app_config.database.password.as_str())
        .database(config.app_config.database.database.as_str());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options).await?;

    sqlx::query("SET application_name = 'develbot';").execute(&pool).await?;

    initialize_schema(&pool).await?;

    initialize_tables(&pool).await?;

    Ok(pool)
}

pub async fn initialize_schema(pg_pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query("CREATE SCHEMA IF NOT EXISTS AUTHORIZATION session_user")
        .execute(pg_pool)
        .await?;

    Ok(())
}

pub async fn initialize_tables(pg_pool: &sqlx::Pool<sqlx::Postgres>) -> anyhow::Result<()> {
    Chatter::db_initialize(pg_pool).await?;
    ChatLogMessage::db_initialize(pg_pool).await?;

    Ok(())
}
