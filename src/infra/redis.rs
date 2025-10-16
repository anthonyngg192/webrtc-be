pub fn create_redis_client(url: &str) -> Result<redis::Client, redis::RedisError> {
    redis::Client::open(url)
}
