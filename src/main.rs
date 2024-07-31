use lambda_runtime::{handler_fn, Context, Error};
use serde::{Deserialize, Serialize};
use tokio_postgres::{NoTls, Client};

#[derive(Serialize)]
struct Response {
    message: String,
}

#[derive(Deserialize, Debug)]
struct Request {
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

async fn function_handler(event: Request, _: Context) -> Result<Response, Error> {
    println!("Received request: {:?}", event); // Log the incoming request

    let connection_str = format!(
        "host={} port={} user={} password={} dbname={}",
        event.db_host, event.db_port, event.db_user, event.db_password, event.db_name
    );
    println!("Connecting to database with connection string: {}", connection_str);

    let (client, connection) = tokio_postgres::connect(&connection_str, NoTls).await?;
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
        if check_database(&client).await? {
            println!("Database is ready!");
            return Ok(Response {
                message: "Database is ready!".to_string(),
            });
        } else {
            println!("Waiting for database to be available...");
            tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
            elapsed += interval;
        }
    }

    println!("Database did not become available within 5 minutes.");
    Ok(Response {
        message: "Database did not become available within 5 minutes.".to_string(),
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Starting Lambda function...");
    let func = handler_fn(function_handler);
    lambda_runtime::run(func).await?;
    println!("Lambda function finished.");
    Ok(())
}
