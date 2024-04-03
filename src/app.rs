use anyhow::Result;
use tokio::{
    sync::{broadcast, mpsc},
    task::JoinHandle,
};

use crate::streamdeck::{Input, StreamDeckPlus};

#[derive(Debug)]
pub struct Apps {
    pub deck: StreamDeckPlus,
    apps: Vec<AppInfo>,
    active_app: Option<usize>,
    active_app_tx: broadcast::Sender<mpsc::UnboundedSender<Input>>,
}

pub type AppResult = JoinHandle<Result<()>>;
pub type AppInfo = (String, AppResult, mpsc::UnboundedSender<Input>);

impl Apps {
    pub fn new(deck: StreamDeckPlus) -> Self {
        let (active_app_tx, _active_app_rx) = broadcast::channel(10);
        Self {
            deck,
            apps: vec![],
            active_app: Some(0),
            active_app_tx,
        }
    }

    pub fn register(
        &mut self,
        name: String,
        handle: AppResult,
        tx: mpsc::UnboundedSender<Input>,
    ) -> Result<()> {
        self.apps.push((name, handle, tx));
        Ok(())
    }

    pub fn activate(&mut self, index: usize) -> Result<()> {
        self.active_app = Some(index);
        let tx = self.apps[index].2.clone();
        self.active_app_tx.send(tx)?;
        Ok(())
    }

    pub fn route(&self) -> Result<()> {
        let deck = self.deck.clone();
        let rx = self.active_app_tx.subscribe();

        tokio::task::Builder::new()
            .name("input router")
            .spawn(router(deck, rx))?;

        Ok(())
    }
}

async fn router(
    deck: StreamDeckPlus,
    mut rx: broadcast::Receiver<mpsc::UnboundedSender<Input>>,
) -> Result<()> {
    let (_handle, mut inputs) = deck.subscribe().unwrap();

    let mut active_channel: Option<mpsc::UnboundedSender<Input>> = None;
    loop {
        tokio::select! {
            Some(input) = inputs.recv() => {
                if let Some(ref chan) = active_channel {
                    chan.send(input)?;
                }
            }
            Ok(new_chan) = rx.recv() => {
                active_channel = Some(new_chan);
            }
        }
    }
}

macro_rules! spawn_app {
    ($apps:ident, $name:literal, $func:ident) => {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<crate::streamdeck::Input>();
        let app = tokio::task::Builder::new()
            .name($name)
            .spawn($func($apps.deck.clone(), rx))?;
        $apps.register($name.into(), app, tx)?;
    };
}

pub(crate) use spawn_app;
