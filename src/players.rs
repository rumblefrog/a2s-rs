use std::io::Cursor;
use std::net::ToSocketAddrs;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{A2SClient, ReadCString};
use crate::errors::{Error, Result};

const PLAYER_REQUEST: [u8; 5] = [0xff, 0xff, 0xff, 0xff, 0x55];

pub struct Players {
    pub count: u8,

    pub players: Vec<Player>,
}

pub struct Player {
    // Index of player chunk starting from 0.
	// This seems to be always 0?
    pub index: u8,

    // Name of the player.
    pub name: String,

    // Player's score (usually "frags" or "kills".)
    pub score: u32,

    // Time (in seconds) player has been connected to the server.
    pub duration: f32,

    // The Ship additional player info
    pub the_ship: Option<TheShipPlayer>,
}

pub struct TheShipPlayer {
    pub deaths: u32,

    pub money: u32,
}

impl A2SClient {
    pub fn players<A: ToSocketAddrs>(&self, addr: A) -> Result<Players> {
        let data = self.do_challenge_request(addr, &PLAYER_REQUEST)?;

        let mut data = Cursor::new(data);

        if data.read_u8()? != 0x44 {
            return Err(Error::InvalidResponse);
        }

        let player_count = data.read_u8()?;

        let mut players: Vec<Player> = Vec::with_capacity(player_count as usize);

        for _ in 0..player_count {
            players.push(Player{
                index: data.read_u8()?,
                name: data.read_cstring()?,
                score: data.read_u32::<LittleEndian>()?,
                duration: data.read_f32::<LittleEndian>()?,
                the_ship: {
                    if self.app_id == 2400 {
                        Some(TheShipPlayer{
                            deaths: data.read_u32::<LittleEndian>()?,
                            money: data.read_u32::<LittleEndian>()?,
                        })
                    } else {
                        None
                    }
                }
            })
        }

        Ok(Players{
            count: player_count,
            players: players,
        })
    }
}