use futures_lite::future::Boxed;

use crate::streamdeck;

pub trait App {
    fn name(&self) -> &'static str;
    fn handle(&self, input: streamdeck::Input) -> Boxed<()>;
}
