use chrono::prelude::*;
use sqlx::{FromRow, PgPool};
use std::time::SystemTime;

#[derive(Clone, Debug, FromRow)]
pub struct Chatter {
    pub login: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub version: i16,
}

impl Chatter {
    const CURRENT_VERSION: i16 = 1_i16;

    #[allow(dead_code)]
    pub fn new(login: String, name: String) -> Self {
        Self {
            login,
            name,
            created_at: DateTime::<Utc>::from(SystemTime::now()),
            version: Chatter::CURRENT_VERSION
        }
    }

    pub async fn db_initialize(pool: &PgPool) -> anyhow::Result<()> {
        sqlx::query("\
            CREATE TABLE IF NOT EXISTS chatters (\
                login varchar(255) PRIMARY KEY,\
                name varchar(255),\
                created_at timestamptz,\
                version smallint\
            );\
        ").execute(pool).await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn find_one(pool: &PgPool, login: String) -> anyhow::Result<Chatter> {
        let result = sqlx::query_as::<_, Chatter>("\
            SELECT * from chatters \
            WHERE login = $1\
        ")
            .bind(login)
            .fetch_one(pool)
            .await?;

        Ok(result)
    }

    #[allow(dead_code)]
    pub async fn upsert(pool: &PgPool, chatter: Self) -> anyhow::Result<()> {
        sqlx::query("\
            INSERT INTO chatters (login, name, created_at, version) \
            VALUES ($1, $2, $3, $4) \
            ON CONFLICT DO \
            UPDATE SET \
                name = $2, \
                created_at = $3, \
                version = $4 \
            WHERE login = $1\
        ")
            .bind(chatter.login)
            .bind(chatter.name)
            .bind(chatter.created_at)
            .bind(chatter.version)
            .execute(pool)
            .await?;

        Ok(())
    }
}
