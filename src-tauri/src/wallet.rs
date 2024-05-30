use crate::error;
use crate::logger;
use chan::Sender;
use crossbeam_channel::{self as chan, Receiver};

use microserde::{Deserialize, Serialize};
use nakamoto_client as client;
use nakamoto_client::{traits::Handle, Network};
use nakamoto_common::block::{Block, BlockHash, Height};
use std::ops::ControlFlow;
use std::ops::ControlFlow::*;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug)]
struct NodeState {
    height: Height,
    message: String,
}

pub struct Wallet<H> {
    client: H,
    network: Network,
}

impl<H: Handle> Wallet<H> {
    pub fn new(client: H, network: client::Network) -> Self {
        Self { client, network }
    }
    /// Run the wallet loop until it exits.
    pub fn run(
        &mut self,
        block_rx: Receiver<(Block, u64)>,
        node_client_tx: &Sender<String>,
        events: chan::Receiver<client::Event>,
        ui_rx: chan::Receiver<String>,
    ) -> Result<(), error::Error> {
        logger::init(log::Level::Debug).expect("initializing logger for the first time");
        // Running...
        loop {
            chan::select! {
                recv(events) -> event => {
                    let event = event?;
                    if let Break(()) = self.handle_client_event(event,&node_client_tx)? {
                        break;
                    }
                }
                recv(ui_rx) -> ui_rx => {
                    let ui_rx = ui_rx?;
                    if let Break(()) = self.handle_ui_request(ui_rx,&node_client_tx)? {
                        break;
                    }
                }
                recv(block_rx) -> block_rx => {
                    let block_rx = block_rx?;
                    if let Break(()) = self.block_sub(block_rx,&node_client_tx)? {
                        break;
                    }
                }
            }
        }
        Ok(())
    }
    fn block_sub(
        &mut self,
        block: (Block, u64),
        node_client_tx: &Sender<String>,
    ) -> Result<ControlFlow<()>, error::Error> {
        !todo!();
        Ok(ControlFlow::Continue(()))
    }
    fn handle_ui_request(
        &mut self,
        ui_event: String,
        node_client_tx: &Sender<String>,
    ) -> Result<ControlFlow<()>, error::Error> {
        _ = node_client_tx;

        match ui_event.as_str() {
            "get-block-2" => {
                _ = self.client.get_tip();
                _ = self.client.get_block(
                    // block 2
                    &BlockHash::from_str(
                        "000000006c02c8ea6e4ff69651f7fcde348fb9d557a06e6957b65552002a7820",
                    )
                    .unwrap(),
                );
            }
            _ => {}
        }
        Ok(ControlFlow::Continue(()))
    }
    fn handle_client_event(
        &mut self,
        event: client::Event,
        node_client_tx: &Sender<String>,
    ) -> Result<ControlFlow<()>, error::Error> {
        match event {
            client::Event::Ready { .. } => {
                println!("node ready",);
            }

            _ => {
                node_client_tx
                    .send(event.to_string())
                    .map_err(|e| e.to_string())
                    .expect("msg");
            }
        }
        Ok(ControlFlow::Continue(()))
    }
    fn handle_peer_height(&mut self, height: Height) -> u64 {
        height
    }
}
