use std::io::Cursor;
#[cfg(not(feature = "async"))]
use std::net::ToSocketAddrs;

use byteorder::{LittleEndian, ReadBytesExt};

#[cfg(feature = "async")]
use tokio::net::ToSocketAddrs;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::errors::{Error, Result};
use crate::{A2SClient, ReadCString};

const RULES_REQUEST: [u8; 5] = [0xFF, 0xFF, 0xFF, 0xFF, 0x56];

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Rule {
    /// Name of the rule.
    pub name: String,

    /// Value of the rule.
    pub value: String,
}

impl A2SClient {
    fn read_rule_data(&self, mut data: Cursor<Vec<u8>>) -> Result<Vec<Rule>> {
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

        Ok(rules)
    }

    #[cfg(feature = "async")]
    pub async fn rules<A: ToSocketAddrs>(&self, addr: A) -> Result<Vec<Rule>> {
        let data = self.do_challenge_request(addr, &RULES_REQUEST).await?;
        self.read_rule_data(Cursor::new(data))
    }

    #[cfg(not(feature = "async"))]
    pub fn rules<A: ToSocketAddrs>(&self, addr: A) -> Result<Vec<Rule>> {
        let data = self.do_challenge_request(addr, &RULES_REQUEST)?;
        self.read_rule_data(Cursor::new(data))
    }
}
