use crate::utils::environment::REDIS_URL;
use deadpool_redis::{redis::cmd, Config, Pool, Runtime};

pub struct RedisAdapter {
    pub pool: Pool,
}

impl Default for RedisAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl RedisAdapter {
    pub fn new() -> Self {
        let cfg = Config::from_url(REDIS_URL.clone());
        let pool = cfg
            .create_pool(Some(Runtime::Tokio1))
            .expect("Failed to create Redis pool");

        Self { pool }
    }

    pub async fn get_user_by_token(&self, token: &str) -> Option<String> {
        let key = format!("user:token:{}", token);

        match self.pool.get().await {
            Ok(mut conn) => match cmd("GET").arg(&key).query_async::<String>(&mut *conn).await {
                Ok(value) => Some(value),
                Err(_) => None,
            },
            Err(_) => None,
        }
    }

    pub async fn set_user_token(&self, user_code: &str, token: &str) -> bool {
        let key = format!("user:token:{}", token);

        match self.pool.get().await {
            Ok(mut conn) => cmd("SET")
                .arg(&key)
                .arg(user_code)
                .arg("EX")
                .arg(60 * 60)
                .query_async::<String>(&mut *conn)
                .await
                .is_ok(),
            Err(_) => false,
        }
    }

    pub async fn get_conversation_user_key(
        &self,
        user_code: &str,
        conversation_id: &str,
    ) -> String {
        let key = format!("CONVERSATION:{}:{}", user_code, conversation_id);

        match self.pool.get().await {
            Ok(mut conn) => match cmd("GET").arg(&key).query_async(&mut *conn).await {
                Ok(val) => val,
                Err(_) => "None".to_string(),
            },
            Err(_) => "None".to_string(),
        }
    }

    pub async fn set_conversation_user_key(
        &self,
        user_code: &str,
        conversation_id: &str,
        other_user_code: &str,
    ) -> bool {
        let key = format!("CONVERSATION:{}:{}", user_code, conversation_id);
        match self.pool.get().await {
            Ok(mut conn) => cmd("SETEX")
                .arg(&key)
                .arg(3600)
                .arg(other_user_code)
                .query_async::<()>(&mut *conn)
                .await
                .is_ok(),
            Err(_) => {
                println!("Redis error");
                false
            }
        }
    }
}
