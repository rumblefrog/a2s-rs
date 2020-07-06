use std::io::Cursor;
use std::net::ToSocketAddrs;

use byteorder::{LittleEndian, ReadBytesExt};

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::errors::{Error, Result};
use crate::{A2SClient, ReadCString};

const RULES_REQUEST: [u8; 5] = [0xFF, 0xFF, 0xFF, 0xFF, 0x56];

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Rules {
    // Number of rules in the response.
    pub count: u16,

    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Rule {
    // Name of the rule.
    pub name: String,

    // Value of the rule.
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
