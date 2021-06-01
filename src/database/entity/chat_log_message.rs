use chrono::prelude::*;
use sqlx::{FromRow, PgPool};

#[derive(Clone, Debug, FromRow)]
pub struct ChatLogMessage {
    pub id: Option<i32>,
    pub chatter_login: String,
    pub message: String,
    pub posted_at: DateTime<Utc>,
    pub version: i16,
}

impl ChatLogMessage {
    const CURRENT_VERSION: i16 = 1_i16;

    #[allow(dead_code)]
    pub fn new(chatter_login: String, message: String, posted_at: DateTime<Utc>) -> Self {
        Self {
            id: Option::None,
            chatter_login,
            message,
            posted_at,
            version: ChatLogMessage::CURRENT_VERSION,
        }
    }

    pub async fn db_initialize(pool: &PgPool) -> anyhow::Result<()> {
        sqlx::query("\
            CREATE TABLE IF NOT EXISTS chat_logs (\
                id SERIAL PRIMARY KEY,\
                chatter_login varchar(255),\
                message varchar(255),\
                posted_at timestamptz,\
                version smallint\
            );\
        ").execute(pool).await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn insert(pool: &PgPool, chat_log_message: Self) -> anyhow::Result<()> {
        sqlx::query("\
            INSERT INTO chat_logs (chatter_login, message, posted_at, version) \
            VALUES ($1, $2, $3, $4)\
        ")
            .bind(chat_log_message.chatter_login)
            .bind(chat_log_message.message)
            .bind(chat_log_message.posted_at)
            .bind(chat_log_message.version)
            .execute(pool)
            .await?;

        Ok(())
    }
}
