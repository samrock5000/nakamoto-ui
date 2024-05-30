// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod logger;
mod wallet;
use client::{Client, Config, Network};
use crossbeam_channel as chan;
use nakamoto_client as client;
use nakamoto_client::traits::Handle;

use std::net::{Ipv4Addr, SocketAddr};
use wallet::Wallet;

use std::sync::Mutex;
use tauri::Manager;
mod error;
use microserde::{json, Deserialize, Serialize};
use std::{net, thread};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Payload {
    message: u64,
}

struct UIInputTx {
    inner: Mutex<chan::Sender<String>>,
}

type Reactor = nakamoto_net_poll::Reactor<net::TcpStream>;

fn main() {
    let (node_tx, node_rx) = chan::unbounded();
    let (ui_tx, ui_rx) = chan::unbounded();
    let (output_tx, output_rx) = chan::unbounded();
    let (loading_tx, loading_rx) = chan::unbounded();

    let client = Client::<Reactor>::new().unwrap();
    // client.handle().
    let handle = client.handle();
    let block_rec = handle.blocks();
    let client_event_recv = handle.events();
    // let ui_handle = handle.clone();
    let network = client::Network::Testnet;

    // let addr = SocketAddr::new(net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 48334);
    let connect: Vec<SocketAddr> = vec![];
    let cfg = Config {
        network,
        connect,
        listen: vec![], // Don't listen for incoming connections.
        ..Config::default()
    };

    let t3 = thread::spawn(move || client.load(cfg, loading_tx)?.run());

    tauri::async_runtime::spawn(async move {
        //TODO offline mode
        if let Ok(x) = loading_rx.recv() {
            println!("Loaded {:?}", x);
        }
        if let Err(err) =
            Wallet::new(handle.clone(), network).run(block_rec, &node_tx, client_event_recv, ui_rx)
        {
            println!("FATAL ERR {}", err);
            std::process::exit(1);
        };
    });

    tauri::Builder::default()
        .manage(UIInputTx {
            inner: Mutex::new(ui_tx),
        })
        // .manage(NodeInputTx {
        //     inner: Mutex::new(out_tx),
        // })
        .invoke_handler(tauri::generate_handler![ui_request])
        .setup(|app| {
            tauri::async_runtime::spawn(async move { chan_process(node_rx, output_tx) });
            let app_handle = app.handle();
            tauri::async_runtime::spawn(async move {
                // let state = app_handle.state::<NodeInputTx>().clone();
                loop {
                    if let Ok(output) = output_rx.recv() {
                        node_event_emitter(output, &app_handle);
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    t3.join().unwrap().unwrap();
}

/// emits node events
fn node_event_emitter<R: tauri::Runtime>(message: String, manager: &impl Manager<R>) {
    manager.emit_all("net_event", message).unwrap();
}

#[tauri::command]
fn ui_request(message: String, state: tauri::State<'_, UIInputTx>) {
    if let Ok(ui_tx) = state.inner.lock() {
        _ = ui_tx.send(message);
    }
}

fn chan_process(
    input_rx: chan::Receiver<String>,
    output_tx: chan::Sender<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    loop {
        if let Ok(input) = input_rx.recv() {
            let output = input;
            output_tx.send(output)?;
        }
    }
}
