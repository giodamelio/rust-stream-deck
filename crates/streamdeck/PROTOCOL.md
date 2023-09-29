# Notes on StreamDeck protocol

I only have a StreamDeck Plus, so this is all based on that. There are some notes added here from other sources, but I have not personally tested any of them.

## Feature Reports

 - `0x01`-`0x02`: Unused? These just return errors when read
 - `0x03`: Unknown/Serial
   - Plus: unknown data
   - Others: apparently this is the serial number on some devices
 - `0x04`: Version string of something. Format doesn't fit firmware or companion software format. Not sure what it is. Example from my Plus: `22.0.0.0`
 - `0x05`: [Firmware version](#plus-firmware-version). Example from my Plus: `2.0.3.2`
 - `0x06`: [Serial Number](#plus-serial-number). Example from my Plus: `A00WA308106HWP`
 - `0x07`: [Last firmware version](#plus-firmware-version)? This is just smaller than the current firmware version, does the device support redundant A/B firmware? Example from my Plus: `2.0.3.1`
 - `0x08`-`0x0C` Unknown data returned
 - `0x0D`-`0xFF`: Unused? These just return errors when read

### Details

Unless otherwise noted, `end` is the first 0x00 byte.

#### Plus Firmware Version

| Byte Range      | Description                                                      |
|-----------------|------------------------------------------------------------------|
| `0x00`          | Feature report id, either `0x05` for current, or `0x07` for last |
| `0x01`          | Length of firmware version, including a single NUL byte          |
| `0x02` - `0x05` | Unknown junk data                                                |
| `0x06` - `end`  | Firmware version as ASCII string                                 |

#### Plus Serial Number

| Byte Range     | Description                      |
|----------------|----------------------------------|
| `0x00`         | Feature report id `0x06`         |
| `0x01`         | Length of serial number in bytes |
| `0x02` - `end` | Serial number as ASCII string    |

## Thanks

This would have been much harder if not for all the preexisting info on the web. Thanks everyone!

 - https://gist.github.com/cliffrowley/d18a9c4569537b195f2b1eb6c68469e0
 - https://github.com/streamduck-org/elgato-streamdeck
 - https://github.com/abcminiuser/python-elgato-streamdeck
 - https://github.com/Julusian/node-elgato-stream-deck