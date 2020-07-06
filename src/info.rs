use std::io::{Cursor, ErrorKind};
use std::net::ToSocketAddrs;

#[cfg(feature = "serde")]
use serde::Serialize;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::errors::{Error, Result};
use crate::{A2SClient, ReadCString};

const INFO_REQUEST: [u8; 25] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0x54, 0x53, 0x6F, 0x75, 0x72, 0x63, 0x65, 0x20, 0x45, 0x6E, 0x67, 0x69,
    0x6E, 0x65, 0x20, 0x51, 0x75, 0x65, 0x72, 0x79, 0x00,
];

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Info {
    // Protocol version used by the server.
    pub protocol: u8,

    // Name of the server.
    pub name: String,

    // Map the server has currently loaded.
    pub map: String,

    // Name of the folder containing the game files.
    pub folder: String,

    // Full name of the game.
    pub game: String,

    // Steam Application ID of game.
    pub app_id: u16,

    // Number of players on the server.
    pub players: u8,

    // Maximum number of players the server reports it can hold.
    pub max_players: u8,

    // Number of bots on the server.
    pub bots: u8,

    // Indicates the type of server
    // Rag Doll Kung Fu servers always return 0 for "Server type."
    pub server_type: ServerType,

    // Indicates the operating system of the server
    pub server_os: ServerOS,

    // Indicates whether the server requires a password
    pub visibility: bool,

    // Specifies whether the server uses VAC
    pub vac: bool,

    // These fields only exist in a response if the server is running The Ship
    pub the_ship: Option<TheShip>,

    // Version of the game installed on the server.
    pub version: String,

    // If present, this specifies which additional data fields will be included.
    pub edf: u8,

    pub extended_server_info: ExtendedServerInfo,

    // Available if edf & 0x40 is true
    pub source_tv: Option<SourceTVInfo>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct TheShip {
    // Indicates the game mode
    pub mode: TheShipMode,

    // The number of witnesses necessary to have a player arrested.
    pub witnesses: u8,

    // Time (in seconds) before a player is arrested while being witnessed.
    pub duration: u8,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum TheShipMode {
    Hunt,
    Elimination,
    Duel,
    Deathmatch,
    VIPTeam,
    TeamElimination,
    Unknown,
}

impl From<u8> for TheShipMode {
    fn from(v: u8) -> Self {
        match v {
            0 => Self::Hunt,
            1 => Self::Elimination,
            2 => Self::Duel,
            3 => Self::Deathmatch,
            4 => Self::VIPTeam,
            5 => Self::TeamElimination,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct ExtendedServerInfo {
    // The server's game port number.
    // Available if edf & 0x80 is true
    pub port: Option<u16>,

    // Server's SteamID.
    // Available if edf & 0x10 is true
    pub steam_id: Option<u64>,

    // Tags that describe the game according to the server (for future use.)
    // Available if edf & 0x20 is true
    pub keywords: Option<String>,

    // The server's 64-bit GameID. If this is present, a more accurate AppID is present in the low 24 bits.
    // The earlier AppID could have been truncated as it was forced into 16-bit storage.
    // Avaialble if edf & 0x01 is true
    pub game_id: Option<u64>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct SourceTVInfo {
    // Spectator port number for SourceTV.
    pub port: u16,

    // Name of the spectator server for SourceTV.
    pub name: String,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum ServerType {
    Dedicated,
    NonDedicated,
    SourceTV,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum ServerOS {
    Linux,
    Windows,
    Mac,
}

impl A2SClient {
    pub fn info<A: ToSocketAddrs>(&self, addr: A) -> Result<Info> {
        let data = self.send(&INFO_REQUEST, addr)?;
        let mut data = Cursor::new(data);

        if data.read_u8()? != 0x49u8 {
            return Err(Error::InvalidResponse);
        }

        let app_id: u16;
        let mut flag = 0u8;

        Ok(Info {
            protocol: data.read_u8()?,
            name: data.read_cstring()?,
            map: data.read_cstring()?,
            folder: data.read_cstring()?,
            game: data.read_cstring()?,
            app_id: {
                app_id = data.read_u16::<LittleEndian>()?;

                app_id
            },
            players: data.read_u8()?,
            max_players: data.read_u8()?,
            bots: data.read_u8()?,
            server_type: {
                match data.read_u8()? as char {
                    'd' => ServerType::Dedicated,
                    'i' => ServerType::NonDedicated,
                    'p' => ServerType::SourceTV,
                    _ => return Err(Error::Other("Invalid server type")),
                }
            },
            server_os: {
                match data.read_u8()? as char {
                    'l' => ServerOS::Linux,
                    'w' => ServerOS::Windows,
                    'm' | 'o' => ServerOS::Mac,
                    _ => return Err(Error::Other("Invalid environment")),
                }
            },
            visibility: data.read_u8()? != 0,
            vac: data.read_u8()? != 0,
            the_ship: {
                if app_id == 2400 {
                    Some(TheShip {
                        mode: TheShipMode::from(data.read_u8()?),
                        witnesses: data.read_u8()?,
                        duration: data.read_u8()?,
                    })
                } else {
                    None
                }
            },
            version: data.read_cstring()?,
            edf: {
                match data.read_u8() {
                    Ok(val) => {
                        flag = val;
                    }
                    Err(err) => {
                        if err.kind() != ErrorKind::UnexpectedEof {
                            return Err(Error::Io(err));
                        }
                    }
                }
                flag
            },
            extended_server_info: ExtendedServerInfo {
                port: {
                    if flag & 0x80 != 0 {
                        Some(data.read_u16::<LittleEndian>()?)
                    } else {
                        None
                    }
                },
                steam_id: {
                    if flag & 0x10 != 0 {
                        Some(data.read_u64::<LittleEndian>()?)
                    } else {
                        None
                    }
                },
                keywords: {
                    if flag & 0x20 != 0 {
                        Some(data.read_cstring()?)
                    } else {
                        None
                    }
                },
                game_id: {
                    if flag & 0x01 != 0 {
                        Some(data.read_u64::<LittleEndian>()?)
                    } else {
                        None
                    }
                },
            },
            source_tv: {
                if flag & 0x40 != 0 {
                    Some(SourceTVInfo {
                        port: data.read_u16::<LittleEndian>()?,
                        name: data.read_cstring()?,
                    })
                } else {
                    None
                }
            },
        })
    }
}
