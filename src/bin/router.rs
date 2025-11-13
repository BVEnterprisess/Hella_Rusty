use clap::Parser;
use dotenvy::dotenv;
use redis::{AsyncCommands, Client};
use std::collections::HashMap;
use tracing::info;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Redis URL
    #[arg(short, long, default_value = "redis://localhost:6379")]
    redis_url: String,

    /// Router port
    #[arg(short, long, default_value = "8081")]
    port: u16,

    /// Number of worker agents
    #[arg(short, long, default_value = "3")]
    workers: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv().ok();

    // Parse command line arguments
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("Starting Chimera Router");

    // Connect to Redis
    let redis_client = Client::open(args.redis_url)?;
    let mut redis_conn = redis_client.get_async_connection().await?;

    // Test Redis connection
    let _: () = redis::cmd("PING").query_async(&mut redis_conn).await?;
    info!("Connected to Redis successfully");

    // Create request stream
    let stream_key = "chimera:requests";

    // Start request processing loop
    process_requests(redis_conn, stream_key).await?;

    Ok(())
}

async fn process_requests(
    mut redis_conn: redis::aio::Connection,
    stream_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting request processing loop");

    loop {
        // Read from Redis stream
        let results: Vec<(String, HashMap<String, redis::Value>)> =
            redis_conn.xread(&[stream_key], &[0]).await?;

        for (_id, fields) in results {
            // Process each request
            if let (Some(request_id), Some(request_data)) =
                (fields.get("request_id"), fields.get("data"))
            {
                if let (
                    redis::Value::Data(request_id_bytes),
                    redis::Value::Data(request_data_bytes),
                ) = (request_id, request_data)
                {
                    let request_id = String::from_utf8_lossy(&request_id_bytes);
                    let _request_data = String::from_utf8_lossy(&request_data_bytes);

                    info!("Processing request: {}", request_id);

                    // TODO: Implement actual request routing logic
                    // This would route requests to appropriate agents based on type/capability

                    // Simulate processing
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                    // Send response back via Redis
                    let response_key = format!("chimera:response:{}", request_id);
                    let response = format!(
                        "{{\"result\": \"processed\", \"request_id\": \"{}\"}}",
                        request_id
                    );

                    let _: () = redis_conn.set(&response_key, response).await?;
                    info!("Response sent for request: {}", request_id);
                }
            }
        }

        // Small delay to prevent busy waiting
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_redis_connection() {
        let client = match Client::open("redis://localhost:6379") {
            Ok(client) => client,
            Err(_) => return, // Skip when URL is invalid
        };

        match client.get_async_connection().await {
            Ok(mut conn) => {
                if let Ok(result) = redis::cmd("PING").query_async::<_, String>(&mut conn).await {
                    assert_eq!(result, "PONG");
                }
            }
            Err(_) => {
                // Redis is not available in the test environment; skip gracefully.
            }
        }
    }
}
