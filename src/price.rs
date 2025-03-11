use futures_util::StreamExt;
use serde_json::Value;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};

const FILE_PATH: &str = "btc_price_data.txt";
const BINANCE_WS_URL: &str = "wss://stream.binance.com:9443/ws/btcusdt@trade";

pub async fn cache_prices(times: u64, process_number: i32) -> f64 {
    let (ws_stream, _) = connect_async(BINANCE_WS_URL).await.unwrap();
    let (mut _write, mut read) = ws_stream.split();

    let mut prices = Vec::new();
    for _ in 0..times {
        if let Some(msg) = read.next().await {
            if let Ok(Message::Text(text)) = msg {
                // println!("Message from websocket {}", text);
                if let Ok(json) = serde_json::from_str::<Value>(&text) {
                    if let Some(price) = json["p"].as_str() {
                        let price: f64 = price.parse().unwrap_or(0.0);
                        println!(
                            "Received price: {:.3} by process number {}",
                            price, process_number
                        );
                        prices.push(price);
                    }
                }
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;
    println!(
        "Cache complete by Process {}. The average USD price of BTC is: {:.4}",
        process_number, avg_price
    );

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(FILE_PATH)
        .unwrap();
    writeln!(file, "{:.4}", avg_price).unwrap();
    for price in prices {
        writeln!(file, "{:.4}", price).unwrap();
    }
    avg_price
}

pub fn read_prices() {
    if Path::new(FILE_PATH).exists() {
        let file = fs::File::open(FILE_PATH).unwrap();
        let reader = io::BufReader::new(file);

        for line in reader.lines() {
            println!("{}", line.unwrap());
        }
    } else {
        println!("File doesn't exist. No cached data found.");
    }
}
