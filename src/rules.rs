use std::io::Cursor;
use std::net::ToSocketAddrs;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{A2SClient, ReadCString};
use crate::errors::{Error, Result};

const RULES_REQUEST: [u8; 5] = [0xFF, 0xFF, 0xFF, 0xFF, 0x56];

pub struct Rules {
    pub count: u16,
    pub rules: Vec<Rule>,
}

pub struct Rule {
    pub name: String,
    pub value: String,
}

impl A2SClient {
    pub fn rules<A: ToSocketAddrs>(&self, addr: A) -> Result<Rules> {
        let data = self.do_challenge_request(addr, &RULES_REQUEST)?;
        
        let mut data = Cursor::new(data);

        if data.read_u8()? != 0x45 {
            return Err(Error::InvalidResponse);
        }

        let count = data.read_u16::<LittleEndian>()?;

        let mut rules: Vec<Rule> = Vec::with_capacity(count as usize);

        for _ in 0..count {
            rules.push(Rule {
                name: data.read_cstring()?,
                value: data.read_cstring()?,
            })
        }

        Ok(Rules {
            count: count,
            rules: rules,
        })
    }
}