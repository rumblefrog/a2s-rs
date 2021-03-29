use std::convert::TryFrom;
use std::io::{Cursor, ErrorKind};
#[cfg(not(feature = "async"))]
use std::net::ToSocketAddrs;

#[cfg(feature = "async")]
use tokio::net::ToSocketAddrs;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::errors::{Error, Result};
use crate::{A2SClient, ReadCString};

const INFO_REQUEST: [u8; 25] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0x54, 0x53, 0x6F, 0x75, 0x72, 0x63, 0x65, 0x20, 0x45, 0x6E, 0x67, 0x69,
    0x6E, 0x65, 0x20, 0x51, 0x75, 0x65, 0x72, 0x79, 0x00,
];

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
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
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct TheShip {
    // Indicates the game mode
    pub mode: TheShipMode,

    // The number of witnesses necessary to have a player arrested.
    pub witnesses: u8,

    // Time (in seconds) before a player is arrested while being witnessed.
    pub duration: u8,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
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
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
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
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct SourceTVInfo {
    // Spectator port number for SourceTV.
    pub port: u16,

    // Name of the spectator server for SourceTV.
    pub name: String,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum ServerType {
    Dedicated,
    NonDedicated,
    SourceTV,
}

impl TryFrom<u8> for ServerType {
    type Error = Error;
    fn try_from(val: u8) -> Result<Self> {
        match val {
            b'd' => Ok(Self::Dedicated),
            b'i' => Ok(Self::NonDedicated),
            b'p' => Ok(Self::SourceTV),
            _ => Err(Self::Error::Other("Invalid server type")),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum ServerOS {
    Linux,
    Windows,
    Mac,
}

impl TryFrom<u8> for ServerOS {
    type Error = Error;

    fn try_from(val: u8) -> Result<Self> {
        match val {
            b'l' => Ok(Self::Linux),
            b'w' => Ok(Self::Windows),
            b'm' | b'o' => Ok(Self::Mac),
            _ => Err(Self::Error::Other("Invalid environment")),
        }
    }
}

impl A2SClient {
    fn read_info_data(&self, mut data: Cursor<Vec<u8>>) -> Result<Info> {
        if data.read_u8()? != 0x49u8 {
            return Err(Error::InvalidResponse);
        }

        let protocol = data.read_u8()?;
        let name = data.read_cstring()?;
        let map = data.read_cstring()?;
        let folder = data.read_cstring()?;
        let game = data.read_cstring()?;
        let app_id = data.read_u16::<LittleEndian>()?;
        let players = data.read_u8()?;
        let max_players = data.read_u8()?;
        let bots = data.read_u8()?;
        let server_type = ServerType::try_from(data.read_u8()?)?;
        let server_os = ServerOS::try_from(data.read_u8()?)?;
        let visibility = data.read_u8()? != 0;
        let vac = data.read_u8()? != 0;
        let the_ship = if app_id == 2400 {
            Some(TheShip {
                mode: TheShipMode::from(data.read_u8()?),
                witnesses: data.read_u8()?,
                duration: data.read_u8()?,
            })
        } else {
            None
        };
        let version = data.read_cstring()?;
        let edf = match data.read_u8() {
            Ok(val) => val,
            Err(err) => {
                if err.kind() != ErrorKind::UnexpectedEof {
                    return Err(Error::Io(err));
                } else {
                    0
                }
            }
        };
        let extended_server_info = ExtendedServerInfo {
            port: if edf & 0x80 != 0 {
                Some(data.read_u16::<LittleEndian>()?)
            } else {
                None
            },
            steam_id: if edf & 0x10 != 0 {
                Some(data.read_u64::<LittleEndian>()?)
            } else {
                None
            },
            keywords: if edf & 0x20 != 0 {
                Some(data.read_cstring()?)
            } else {
                None
            },
            game_id: if edf & 0x01 != 0 {
                Some(data.read_u64::<LittleEndian>()?)
            } else {
                None
            },
        };
        let source_tv = if edf & 0x40 != 0 {
            Some(SourceTVInfo {
                port: data.read_u16::<LittleEndian>()?,
                name: data.read_cstring()?,
            })
        } else {
            None
        };

        Ok(Info {
            protocol,
            name,
            map,
            folder,
            game,
            app_id,
            players,
            max_players,
            bots,
            server_type,
            server_os,
            visibility,
            vac,
            the_ship,
            version,
            edf,
            extended_server_info,
            source_tv,
        })
    }

    #[cfg(feature = "async")]
    pub async fn info<A: ToSocketAddrs>(&self, addr: A) -> Result<Info> {
        let data = self.send(&INFO_REQUEST, addr).await?;
        self.read_info_data(Cursor::new(data))
    }

    #[cfg(not(feature = "async"))]
    pub fn info<A: ToSocketAddrs>(&self, addr: A) -> Result<Info> {
        let data = self.send(&INFO_REQUEST, addr)?;
        self.read_info_data(Cursor::new(data))
    }
}
