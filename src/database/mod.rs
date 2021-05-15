use clap::ArgMatches;
use sqlx::postgres::PgPoolOptions;
use std::env;

pub async fn connect_db<'a>(args: &ArgMatches<'a>) -> Result<sqlx::Pool<sqlx::Postgres>, sqlx::Error> {
    let db_url = env::var("DATABASE_URL").unwrap_or(String::from("mysql://postgres@127.0.0.1/develbot"));

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url).await?;

    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&pool).await?;

    println!("query result: {:?}", row);

    Ok(pool)
}
