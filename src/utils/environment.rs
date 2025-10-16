use std::env;

use once_cell::sync::Lazy;

pub static API_PORT: Lazy<usize> = Lazy::new(|| {
    env::var("API_PORT")
        .expect("Missing API_PORT variable")
        .parse()
        .unwrap()
});

pub static RTC_MIN_PORT: Lazy<usize> = Lazy::new(|| {
    env::var("RTC_MIN_PORT")
        .expect("Missing API_PORT variable")
        .parse()
        .unwrap()
});

pub static RTC_MAX_PORT: Lazy<usize> = Lazy::new(|| {
    env::var("RTC_MAX_PORT")
        .expect("Missing API_PORT variable")
        .parse()
        .unwrap()
});

pub static APP_NAME: Lazy<String> =
    Lazy::new(|| env::var("APP_NAME").expect("Missing APP_NAME variable"));

pub static JWT_SECRET_KEY: Lazy<String> =
    Lazy::new(|| env::var("JWT_SECRET_KEY").expect("Missing JWT_SECRET_KEY variable"));

pub static REDIS_URL: Lazy<String> =
    Lazy::new(|| env::var("REDIS_URL").expect("Missing REDIS_URL variable"));

pub static DATABASE_NAME: Lazy<String> =
    Lazy::new(|| env::var("DATABASE_NAME").expect("Missing DATABASE_NAME variable"));

pub static DB_CONNECTION_STRING: Lazy<String> =
    Lazy::new(|| env::var("DB_CONNECTION_STRING").expect("Missing DB_CONNECTION_STRING variable"));

pub static HASH_ROUND: Lazy<u32> = Lazy::new(|| {
    env::var("HASH_ROUND")
        .expect("Missing HASH_ROUND variable")
        .parse()
        .unwrap()
});

pub static MEDIASOUP_LISTEN_IP: Lazy<String> =
    Lazy::new(|| env::var("MEDIASOUP_LISTEN_IP").expect("Missing MEDIASOUP_LISTEN_IP variable"));

pub static MEDIASOUP_ANNOUNCED_IP: Lazy<String> = Lazy::new(|| {
    env::var("MEDIASOUP_ANNOUNCED_IP").expect("Missing MEDIASOUP_ANNOUNCED_IP variable")
});

pub static NUM_WORKER: Lazy<usize> = Lazy::new(|| {
    env::var("NUM_WORKER")
        .expect("Missing NUM_WORKER variable")
        .parse()
        .unwrap()
});
