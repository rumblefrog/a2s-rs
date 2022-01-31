use std::io::Cursor;
#[cfg(not(feature = "async"))]
use std::net::ToSocketAddrs;

#[cfg(feature = "async")]
use tokio::net::ToSocketAddrs;

use byteorder::{LittleEndian, ReadBytesExt};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::errors::{Error, Result};
use crate::{A2SClient, ReadCString};

const PLAYER_REQUEST: [u8; 5] = [0xff, 0xff, 0xff, 0xff, 0x55];

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Player {
    /// Index of player chunk starting from 0.
    /// This seems to be always 0?
    pub index: u8,

    /// Name of the player.
    pub name: String,

    /// Player's score (usually "frags" or "kills".)
    pub score: i32,

    /// Time (in seconds) player has been connected to the server.
    pub duration: f32,

    /// The Ship additional player info
    pub the_ship: Option<TheShipPlayer>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct TheShipPlayer {
    pub deaths: u32,

    pub money: u32,
}

impl Player {
    pub fn from_cursor(mut data: Cursor<Vec<u8>>, app_id: u16) -> Result<Vec<Self>> {
        if data.read_u8()? != 0x44 {
            return Err(Error::InvalidResponse);
        }

        let player_count = data.read_u8()?;

        let mut players: Vec<Self> = Vec::with_capacity(player_count as usize);

        for _ in 0..player_count {
            players.push(Self {
                index: data.read_u8()?,
                name: data.read_cstring()?,
                score: data.read_i32::<LittleEndian>()?,
                duration: data.read_f32::<LittleEndian>()?,
                the_ship: {
                    if app_id == 2400 {
                        Some(TheShipPlayer {
                            deaths: data.read_u32::<LittleEndian>()?,
                            money: data.read_u32::<LittleEndian>()?,
                        })
                    } else {
                        None
                    }
                },
            })
        }

        Ok(players)
    }
}

impl A2SClient {
    #[cfg(feature = "async")]
    pub async fn players<A: ToSocketAddrs>(&self, addr: A) -> Result<Vec<Player>> {
        let data = self.do_challenge_request(addr, &PLAYER_REQUEST).await?;
        Player::from_cursor(Cursor::new(data), self.app_id)
    }

    #[cfg(not(feature = "async"))]
    pub fn players<A: ToSocketAddrs>(&self, addr: A) -> Result<Vec<Player>> {
        let data = self.do_challenge_request(addr, &PLAYER_REQUEST)?;
        Player::from_cursor(Cursor::new(data), self.app_id)
    }
}
