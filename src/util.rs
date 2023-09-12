use crossbeam::channel::{Receiver, Sender};
use eyre::{bail, eyre, WrapErr};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::thread;
use std::thread::JoinHandle;

pub struct RequestableThread<A, W, REQ, RES>
where
    A: Clone,
    W: Requestable + Setup,
    REQ: Debug + Clone + Send,
    RES: Debug + Clone + Send,
{
    args: A,
    request_chan: (Sender<Request<REQ>>, Receiver<Request<REQ>>),
    response_chan: (Sender<Response<RES>>, Receiver<Response<RES>>),
    wrapped: PhantomData<W>,
}

impl<A, W, REQ, RES> RequestableThread<A, W, REQ, RES>
where
    A: Clone + Send + 'static,
    W: Requestable<Request = REQ, Response = RES> + Setup<State = W, Args = A>,
    REQ: Debug + Clone + Send + PartialEq + 'static,
    RES: Debug + Clone + Send + PartialEq + 'static,
{
    pub fn new(args: A) -> Self {
        Self {
            args,
            request_chan: crossbeam::channel::unbounded(),
            response_chan: crossbeam::channel::unbounded(),
            wrapped: PhantomData,
        }
    }

    pub fn start(&mut self) -> eyre::Result<JoinHandle<()>> {
        let requester = self.request_chan.1.clone();
        let responder = self.response_chan.0.clone();
        let args = self.args.clone();

        let handle = thread::spawn(move || {
            let wrapped = W::setup(args);
            loop {
                let request = match requester.recv() {
                    Ok(r) => r,
                    Err(_) => return,
                };
                match request {
                    Request::Exit => {
                        responder.send(Response::Exiting).expect("Respond error");
                        return;
                    }
                    Request::Custom(r) => {
                        match wrapped.handle(r) {
                            Ok(res) => responder
                                .send(Response::Custom(res))
                                .expect("Respond error"),
                            Err(_err) => responder.send(Response::Error).expect("Handling error"),
                        };
                    }
                }
            }
        });

        Ok(handle)
    }

    pub fn request(&self, request: REQ) -> eyre::Result<RES> {
        self.request_chan
            .0
            .send(Request::Custom(request))
            .map_err(|_| eyre!("Could not send request"))?;

        self.response_chan
            .1
            .recv()
            .map(|res| match res {
                Response::Custom(r) => r,
                _ => panic!("Should never get here"),
            })
            .wrap_err("Error receiving response")
    }

    pub fn exit(&self) -> eyre::Result<()> {
        self.request_chan
            .0
            .send(Request::Exit)
            .map_err(|_| eyre!("Could not send request"))?;

        debug_assert_eq!(
            self.response_chan
                .1
                .recv()
                .wrap_err("Error receiving response")?,
            Response::Exiting
        );

        Ok(())
    }
}

pub trait Requestable {
    type Request: PartialEq;
    type Response: PartialEq;

    fn handle(&self, request: Self::Request) -> eyre::Result<Self::Response>;
}

pub trait Setup {
    type State;
    type Args;

    fn setup(args: Self::Args) -> Self::State;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Request<R> {
    Custom(R),
    Exit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Response<R> {
    Custom(R),
    Error,
    Exiting,
}
