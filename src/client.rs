use std::net::{UdpSocket, ToSocketAddrs};
use std::time::Duration;
use std::io::Read;
use std::ops::Deref;

use byteorder::{ByteOrder, LittleEndian};
use bzip2::read::{BzDecoder};
use crc::{crc32};

use crate::errors::{ Result, Error };

const SINGLE_PACKET: i32 = -1;
const MULTI_PACKET: i32 = -2;

struct PacketFragment {
    number: u8,
    payload: Vec<u8>,
}

pub struct A2SClient {
    socket: UdpSocket,
    max_size: usize,
}

impl A2SClient {
    pub fn new(max_size: Option<usize>) -> Result<A2SClient> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;

        socket.set_read_timeout(Some(Duration::new(5, 0)))?;
        socket.set_write_timeout(Some(Duration::new(5, 0)))?;

        Ok(A2SClient{
            socket: socket,
            max_size: max_size.unwrap_or(1400),
        })
    }

    fn send<A: ToSocketAddrs>(&self, payload: &[u8], addr: A) -> Result<Vec<u8>> {
        self.socket.send_to(payload, addr)?;

        // 32 Bit (4 bytes) header to indicate split or singular response
        let mut header = [0; 4];

        self.socket.recv(&mut header)?;

        let header = LittleEndian::read_i32(&header);

        if header == SINGLE_PACKET {
            let mut data = vec![0u8; self.max_size];

            let read = self.socket.recv(&mut data)?;

            data.truncate(read);

            Ok(data)
        } 

        else if header == MULTI_PACKET {
            // ID - long (4 bytes)
            // Total - byte (1 byte)
            // Number - byte (1 byte)
            // Size - short (2 bytes)

            let mut data = [0u8, 8];

            self.socket.recv(&mut data)?;

            let id = LittleEndian::read_i32(&data[0..4]);
            let total_packets: usize = data[4] as usize;
            let switching_size: usize = LittleEndian::read_i16(&data[6..8]) as usize;

            let mut packets: Vec<PacketFragment> = Vec::with_capacity(total_packets);

            loop {
                let mut data = vec![0u8; switching_size];

                let read = self.socket.recv(&mut data)?;

                if read < data.len() {
                    data.truncate(read);
                }

                // Skip ID field (4 bytes 0..4)
                let packet_id = LittleEndian::read_i32(&data[4..8]);

                if packet_id != id {
                    return Err(Error::MismatchID);
                }

                packets.push(PacketFragment{
                    number: data[10],
                    payload: Vec::from(&data[12..]),
                });

                if packets.len() == total_packets {
                    break;
                }
            }

            packets.sort_by_key(|p| p.number);

            let mut aggregation = Vec::with_capacity(total_packets * self.max_size);

            for p in packets {
                aggregation.extend(p.payload);
            }

            if id as u32 & 0x80000000 != 0 {
                let decompressed_size = LittleEndian::read_i32(&data[0..4]);
                let checksum = LittleEndian::read_i32(&data[4..8]);

                if decompressed_size > (1024*1024) {
                    return Err(Error::InvalidBz2Size);
                }

                let mut decompressed = Vec::with_capacity(total_packets * self.max_size);

                BzDecoder::new(aggregation.deref()).read(&mut decompressed)?;

                if crc32::checksum_ieee(&decompressed) != checksum as u32 {
                    return Err(Error::CheckSumMismatch);
                }

                Ok(decompressed)
            } 
            
            else {
                Ok(aggregation)
            }
        }

        else {
            Err(Error::InvalidResponse)
        }
    }
}
