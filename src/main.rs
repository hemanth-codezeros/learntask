use futures_util::StreamExt;
use serde_json::Value;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

const FILE_PATH: &str = "btc_price_data.txt";
const BINANCE_WS_URL: &str = "wss://stream.binance.com:9443/ws/btcusdt@trade";

async fn cache_prices(times: u64) -> f64 {
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
                        // println!("Received price: {:.3}", price);
                        prices.push(price);
                    }
                }
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;
    println!(
        "Cache complete. The average USD price of BTC is: {:.4}",
        avg_price
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

fn read_prices() {
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

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Please use this format: --mode=<cache|read> [--times=<seconds>]");
        return;
    }
    println!("Arguments received: Mode: {}, Times: {}", args[1], args[2]);

    match args[1].as_str() {
        "--mode=cache" => {
            if args.len() < 3 {
                eprintln!("Please use this format: ./simple --mode=cache --times=<seconds>");
                return;
            }
            let times = args[2].replace("--times=", "").parse::<u64>().unwrap_or(10);

            let mut handles = Vec::new();

            // for Question 1 // cache_prices(times).await; 

            // Question 2
            // Creating async tasks for 5 client process, and run join on task_futures to run simultaneously
            let (tx, mut rx) = mpsc::channel(32);

            for _ in 0..5 {
                let txcopy: mpsc::Sender<_> = tx.clone();
                let handle = tokio::spawn(async move {
                    txcopy.send(cache_prices(times).await).await.unwrap();
                });
                handles.push(handle);
            }
            drop(tx);

            futures::future::join_all(handles).await;
            // Aggregator receiver process which keeps receiving data from client process
            let agg_handle = tokio::spawn(async move {
                let mut values = Vec::new();
                while let Some(val) = rx.recv().await {
                    values.push(val);
                }
                let avg_price = values.iter().sum::<f64>() / values.len() as f64;
                println!(
                    "FINAL AVERAGE received from all client processes: {}",
                    avg_price
                );
            });
            let _ = agg_handle.await;
        }
        "--mode=read" => read_prices(),
        _ => eprintln!("Invalid mode. Use --mode=cache or --mode=read"),
    }
}
