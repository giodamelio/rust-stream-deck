use crate::util::{Requestable, RequestableThread, Setup};
use crossbeam::channel::{Receiver, Sender};
use elgato_streamdeck::info::Kind;
use elgato_streamdeck::StreamDeck;
use eyre::{bail, eyre, Result};
use hidapi::HidApi;
use image::DynamicImage;
use std::fmt::{Debug, Formatter};
use std::thread::JoinHandle;

struct RawDeck(StreamDeck);

impl Setup for RawDeck {
    type State = RawDeck;
    type Args = (Kind, String);

    fn setup((kind, serial): Self::Args) -> Self::State {
        let hid = HidApi::new().unwrap();
        let device = StreamDeck::connect(&hid, kind, &serial).unwrap();
        RawDeck(device)
    }
}
impl Requestable for RawDeck {
    type Request = DeckRequest;
    type Response = DeckResponse;

    fn handle(&self, request: Self::Request) -> eyre::Result<Self::Response> {
        match request {
            Self::Request::Ping => Ok(Self::Response::Pong),
            Self::Request::SetBrightness(val) => {
                self.0.set_brightness(val)?;
                Ok(Self::Response::Success)
            }
        }
    }
}

pub struct Deck {
    pub kind: Kind,
    pub product: String,
    pub serial_number: String,
    pub firmware_version: String,

    serial: String,
    raw_deck: Option<RequestableThread<(Kind, String), RawDeck, DeckRequest, DeckResponse>>,
    // serial: String,
    // command_handle: Option<(Sender<DeckRequest>, Receiver<DeckResponse>)>,
    // device: Option<Arc<RwLock<StreamDeck>>>,
    // Handle input events
    // input_listeners: Vec<Sender<StreamDeckInput>>,
    // input_thread: Option<thread::JoinHandle<()>>,
}

#[derive(Debug, Clone)]
pub enum Status {
    Disconnected,
    Connected,
}

impl Debug for Deck {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (serial_number={}, firmware_version={})",
            self.product, self.serial_number, self.firmware_version
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeckRequest {
    Ping,
    SetBrightness(u8),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeckResponse {
    Pong,
    Success,
}

impl Deck {
    pub fn list_devices() -> Result<Vec<Deck>> {
        // Get a list of connected devices
        let mut hid = HidApi::new()?;
        elgato_streamdeck::refresh_device_list(&mut hid)?;
        let devices: Vec<(Kind, String)> = elgato_streamdeck::list_devices(&hid);

        // Connect to each of them, and extract a few bits of data
        let mut decks: Vec<Deck> = vec![];
        for (kind, serial) in devices {
            let device = StreamDeck::connect(&hid, kind, &serial)?;

            decks.push(Deck {
                kind,
                product: device.product()?,
                serial_number: device.serial_number()?,
                firmware_version: device.firmware_version()?,
                serial,
                raw_deck: None,
            });
        }

        Ok(decks)
    }

    pub fn start(&mut self) -> Result<()> {
        let mut thread = RequestableThread::new((self.kind, self.serial.clone()));
        thread.start()?;
        self.raw_deck = Some(thread);

        Ok(())
    }

    pub fn exit(self) -> Result<()> {
        self.raw_deck
            .ok_or(eyre!("Could not get raw_deck reference"))?
            .exit()
    }

    pub fn ping(&self) -> Result<DeckResponse> {
        let deck = self.raw_deck.as_ref();
        deck.unwrap().request(DeckRequest::Ping)
    }

    pub fn set_brightness(&self, percent: u8) -> Result<DeckResponse> {
        let deck = self.raw_deck.as_ref();
        deck.unwrap().request(DeckRequest::SetBrightness(percent))
    }

    // pub fn wait(&self) -> Result<()> {
    //     match &self.handle {
    //         None => Ok(()),
    //         Some(handle) => handle.join().map_err(|_| eyre!("Could not join thread")),
    //     }
    // }

    // pub fn connect(&mut self) -> Result<()> {
    //     let hid = HidApi::new()?;
    //     let device = StreamDeck::connect(&hid, self.kind, &self.serial)?;
    //     let (request_tx, request_rx) = crossbeam::channel::unbounded::<DeckRequest>();
    //     let (response_tx, response_rx) = crossbeam::channel::unbounded::<DeckResponse>();
    //
    //     self.command_handle = Some((request_tx, response_rx));
    //     self.status = Status::Connected;
    //     self.event_loop(device, request_rx, response_tx)?;
    //
    //     Ok(())
    // }

    // pub fn disconnect(&mut self) -> Result<()> {
    //     self.status = Status::Disconnected;
    //
    //     if let Some((sender, receiver)) = &self.command_handle {
    //         sender.send(DeckRequest::Exit)?;
    //         return match receiver.recv() {
    //             Ok(DeckResponse::Exiting) => Ok(()),
    //             other => bail!("Error shutting down thread: {:?}", other),
    //         };
    //     }
    //
    //     Ok(())
    // }
    //
    // pub fn status(&self) -> Status {
    //     match &self.device {
    //         None => Status::Disconnected,
    //         Some(_dev) => Status::Connected,
    //     }
    // }
    //
    // pub fn set_button_image(&self, index: u8, image: DynamicImage) -> Result<()> {
    //     if let Some(device) = &self.device {
    //         let device = device
    //             .write()
    //             .map_err(|_| eyre!("Could not get reference to device"))?;
    //
    //         (*device)
    //             .set_button_image(index, image)
    //             .wrap_err("Problem setting button image")
    //     } else {
    //         bail!("Device not connected")
    //     }
    // }

    // fn event_loop(
    //     &self,
    //     device: StreamDeck,
    //     requester: Receiver<DeckRequest>,
    //     responder: Sender<DeckResponse>,
    // ) -> Result<()> {
    //     crossbeam::thread::scope(|s| {
    //         s.spawn(move |_| loop {
    //             // Try to handle a message
    //             if let Ok(request) = requester.try_recv() {
    //                 match request {
    //                     DeckRequest::Exit => {
    //                         responder.send(DeckResponse::Exiting).unwrap();
    //                         return;
    //                     }
    //                 }
    //             }
    //
    //             // Read an input event if possible
    //             let event = device.read_input(Some(Duration::ZERO));
    //             match event {
    //                 Err(_) => continue,
    //                 Ok(event) => {
    //                     if event.is_empty() {
    //                         continue;
    //                     }
    //                     dbg!(event);
    //                 }
    //             };
    //         });
    //     })
    //     .map_err(|e| eyre!("Problem creating event loop: {:?}", e))
    // }

    //
    // pub fn set_lcd_image(&self, image: DynamicImage) -> Result<()> {
    //     self.device
    //         .write_lcd(0, 0, &ImageRect::from_image(image)?)
    //         .wrap_err("Problem setting lcd image")
    // }
    //
    // pub fn add_listener(&mut self) -> Receiver<StreamDeckInput> {
    //     let (tx, rx) = crossbeam::channel::unbounded();
    //     self.input_listeners.push(tx);
    //     rx
    // }
    //
    // pub fn input_listen(&mut self) -> Result<()> {
    //     if self.input_thread.is_some() {
    //         return Ok(());
    //     }
    //
    //     self.input_thread = Some(thread::spawn(move || loop {
    //         let input = self.device.read_input(None).unwrap();
    //         if input.is_empty() {
    //             continue;
    //         }
    //         for rx in &self.input_listeners {
    //             rx.send(input.clone()).unwrap();
    //         }
    //     }));
    //
    //     Ok(())
    // }
}
