use reqwest::{header::USER_AGENT, Client};
use std::io;

#[tokio::main]
async fn main() {
    loop {
        let mut input = String::new();
        println!("Choose what to do:");
        println!("\"q\" - quit");
        println!("\"s\" - send request");
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        match input.trim().to_lowercase().as_str() {
            "q" => break,
            "s" => {
                println!("Sending http request ...");
                let http_client = Client::new();
                let response = http_client
                    .get("http://localhost:5000/greet/foo/bar?test=100")
                    .header(USER_AGENT, "terminal")
                    .send()
                    .await;
                match response {
                    Ok(response) => {
                        let data = response.text().await;
                        println!("Response: {:?}", data);
                    }
                    Err(err) => println!("Error: {}", err),
                }
            }
            _ => println!("Invalid option, try again."),
        }
    }
}
