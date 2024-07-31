use lambda_http::{service_fn, Body, Error, Request as LambdaRequest, RequestExt, Response as LambdaResponse};
use serde::{Deserialize, Serialize};
use tokio_postgres::{NoTls, Client};

#[derive(Serialize)]
struct Response {
    message: String,
}

#[derive(Deserialize, Debug)]
struct DbParams {
    db_host: String,
    db_port: String,
    db_user: String,
    db_password: String,
    db_name: String,
}

async fn check_database(client: &Client) -> Result<bool, tokio_postgres::Error> {
    let query = "SELECT 1";
    println!("Running database query: {}", query);
    let result = client.query(query, &[]).await;
    match result {
        Ok(_) => {
            println!("Database query succeeded");
            Ok(true)
        },
        Err(err) => {
            eprintln!("Database query error: {}", err);
            Ok(false)
        },
    }
}

async fn function_handler(event: LambdaRequest) -> Result<LambdaResponse<Body>, Error> {
    let body = event.body();
    let params: DbParams = serde_json::from_slice(body)?;

    println!("Received request: {:?}", params); // Log the incoming request

    let connection_str = format!(
        "host={} port={} user={} password={} dbname={}",
        params.db_host, params.db_port, params.db_user, params.db_password, params.db_name
    );
    println!("Connecting to database with connection string: {}", connection_str);

    let (client, connection) = match tokio_postgres::connect(&connection_str, NoTls).await {
        Ok((client, connection)) => (client, connection),
        Err(e) => {
            eprintln!("Error connecting to database: {}", e);
            return Ok(LambdaResponse::builder()
                .status(500)
                .body(Body::Text(format!("Error connecting to database: {}", e)))
                .unwrap());
        }
    };
    println!("Connection established, spawning connection handler...");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let mut elapsed = 0;
    let timeout = 300; // 5 minutes
    let interval = 5;

    while elapsed < timeout {
        println!("Checking database connection, attempt after {} seconds...", elapsed);
        match check_database(&client).await {
            Ok(true) => {
                println!("Database is ready!");
                return Ok(LambdaResponse::builder()
                    .status(200)
                    .body(Body::Text("Database is ready!".to_string()))
                    .unwrap());
            },
            Ok(false) => {
                println!("Database not ready yet, retrying...");
            },
            Err(e) => {
                eprintln!("Error checking database: {}", e);
                return Ok(LambdaResponse::builder()
                    .status(500)
                    .body(Body::Text(format!("Error checking database: {}", e)))
                    .unwrap());
            }
        }

        println!("Waiting for database to be available...");
        tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
        elapsed += interval;
    }

    println!("Database did not become available within 5 minutes.");
    Ok(LambdaResponse::builder()
        .status(500)
        .body(Body::Text("Database did not become available within 5 minutes.".to_string()))
        .unwrap())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Starting Lambda function...");
    let func = service_fn(function_handler);
    lambda_http::run(func).await?;
    println!("Lambda function finished.");
    Ok(())
}
