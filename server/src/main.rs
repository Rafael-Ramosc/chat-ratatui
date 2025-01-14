mod handles;
mod server_error;
mod server_state;

use dotenv::dotenv;

use handles::connection_handles::handle_connection;
use server_state::State;
use std::env;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let server_adress = "127.0.0.1:8080".to_string();
    println!("Server adress: {}", &server_adress);
    let listener = TcpListener::bind(server_adress).await?;

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let server_limit_connections: u32 = 30;
    let state = Arc::new(
        State::new(server_limit_connections, &database_url)
            .await
            .expect("Failed to create state with database connection"),
    );

    state
        .test_connection()
        .await
        .expect("Failed to connect to database");

    println!("Successfully connected to database");

    // accept connections
    loop {
        let (socket, addr) = listener.accept().await?; //TODO: handle error kind
        println!("{} is connecting...", addr);

        let state = state.clone();

        // new task
        tokio::spawn(async move {
            let id = State::id_increment(&state).await;
            match handle_connection(socket, state.clone(), id).await {
                Ok((addr, mut user)) => {
                    if let Err(e) = user.set_status(&state.db_pool).await {
                        println!("Error updating user status: {:?}", e);
                    }
                    println!("{} Connection closed", addr)
                }
                Err(e) => println!("Error: {:?}", e),
            };
        });
    }
}
