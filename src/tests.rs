use std::convert::Into;
use std::env;
use std::ops::Add;
use std::str::FromStr;

use anyhow::Result;
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use lazy_static::lazy_static;
use log::LevelFilter;
use once_cell::sync::Lazy;
use parking_lot::{ReentrantMutex, ReentrantMutexGuard};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::sqlx_macros::migrate;
use sqlx::{ConnectOptions, Executor, PgPool};

use crate::types::{PeopleID, RangeTime, ServiceID, SpecializationID};

pub static MUTEX: Lazy<ReentrantMutex<()>> = Lazy::new(ReentrantMutex::default);
const DATABASE_URL: &str = "postgres://postgres:postgres@localhost/explany_test";

pub static TEST_SERVICE_1: ServiceID = 4;
pub static TEST_SERVICE_1_SPECIALIZATION_1: SpecializationID = 9;
pub static TEST_SERVICE_1_SPECIALIZATION_2: SpecializationID = 10;
pub static TEST_SPEC_1: PeopleID = 1;
pub static TEST_USER_1: PeopleID = 2;
pub static TEST_DATE_FROM: RangeTime = 1670792284;
pub static TEST_DATE_TO: RangeTime = 1970792284;
pub static TEST_DATE_FROM_HOUR: RangeTime = 1679792284;

lazy_static! {
    pub static ref TEST_DATE_FROM_NATIVE: DateTime<Utc> = Utc::now();
    pub static ref TEST_DATE_TO_NATIVE: DateTime<Utc> = Utc::now().add(Duration::days(720));
}
pub struct PgPoolMutex {
    pub pool: PgPool,
    pub mutex: ReentrantMutexGuard<'static, ()>,
}

impl PgPoolMutex {
    async fn create(database_url: &str, size: u32) -> PgPoolMutex {
        let pool = init_pg_pool(database_url, size).await.expect("connect to db");
        Self {
            pool,
            mutex: MUTEX.lock(),
        }
    }
}
pub async fn init_db() -> PgPoolMutex {
    let pool_mutex = PgPoolMutex::create(&get_database_url(), 10).await;
    pool_mutex
        .pool
        .execute("drop schema public cascade")
        .await
        .expect("Failed to drop schema");
    pool_mutex
        .pool
        .execute("create schema public")
        .await
        .expect("Failed to create schema");
    migrate!("./migrations/")
        .run(&pool_mutex.pool)
        .await
        .expect("Failed to run migrations");
    init_dictionaries(&pool_mutex.pool).await;
    pool_mutex
}

async fn init_dictionaries(pool: &PgPool) {
    pool.execute(include_str!("../tests/dict.sql")).await.unwrap();
}

fn get_database_url() -> String {
    env::var("DATABASE_TEST_URL").unwrap_or_else(|_| DATABASE_URL.to_string())
}

async fn init_pg_pool(db_url: &str, pool_size: u32) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(pool_size)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .after_connect(|conn, _| Box::pin(async move { conn.execute("select 1").await.map(|_| ()) }))
        .connect_with(
            PgConnectOptions::from_str(db_url)?
                .log_statements(LevelFilter::Debug)
                .log_slow_statements(LevelFilter::Debug, std::time::Duration::from_secs(10))
                .clone(),
        )
        .await?;

    Ok(pool)
}
