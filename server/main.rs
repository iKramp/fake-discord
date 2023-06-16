use tokio_postgres::{NoTls, Error};
use std::env;
use anyhow;

#[tokio::main] // By default, tokio_postgres uses the tokio crate as its runtime.
async fn main() -> anyhow::Result<()> {
    let username: String;
    if let Ok(user) = env::var("USERNAME") {
        username = user;
    } else {
        panic!("could not get the username environment variable");
    }
    // Connect to the database.
    let (client, connection) =
        tokio_postgres::connect(format!("host=localhost user={} dbname = mydb", username).as_str(), NoTls).await?;

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // Now we can execute a simple statement that just returns its parameter.
    let rows = client
        .query("SELECT $1::TEXT", &[&"hello world"])
        .await?;

    // And then check that we got back the same string we sent over.
    let value: &str = rows[0].get(0);
    assert_eq!(value, "hello world");

    Ok(())
}