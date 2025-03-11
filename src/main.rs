use std::collections::HashMap;

use crypt::{generate_keypair, sign_messg, verify_signature};
use price::{cache_prices, read_prices};
use tokio::sync::mpsc;

mod crypt;
mod price;

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
            let mut map = HashMap::new();

            for process_number in 0..5 {
                let txcopy = tx.clone();
                let pair = generate_keypair().unwrap();
                let public_key_bytes = pair.0;
                map.insert(process_number, public_key_bytes);
                let handle = tokio::spawn(async move {
                    let avg_price = cache_prices(times, process_number).await;
                    let signed_messg = sign_messg(avg_price.to_string().as_bytes(), pair.1);
                    txcopy
                        .send((process_number, signed_messg, avg_price))
                        .await
                        .unwrap();
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
                let agg_values: Vec<f64> = values
                    .iter()
                    .filter(|x| {
                        verify_signature(x.2.to_string().as_bytes(), *map.get(&x.0).unwrap(), x.1)
                            .unwrap()
                    })
                    .map(|x| x.2)
                    .collect();
                let avg_price = agg_values.iter().sum::<f64>() / values.len() as f64;
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
