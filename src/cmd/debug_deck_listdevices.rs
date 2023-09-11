use eyre::Result;

use crate::deck::Deck;

pub fn run() -> Result<()> {
    let decks = Deck::list_devices()?;

    println!("{} StreamDeck devices are connected\n", decks.len());

    // Connect to each device to get it's info
    for deck in decks {
        println!("Device Model: {}", deck.product);
        println!("├── Kind: {:?}", deck.kind);
        println!("├── Serial Number: {}", deck.serial_number);
        println!("└── Firmware Version: {}", deck.firmware_version);
    }

    Ok(())
}
