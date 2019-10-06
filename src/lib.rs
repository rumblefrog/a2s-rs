pub mod info;
pub mod players;
pub mod errors;

use std::net::{UdpSocket, ToSocketAddrs};
use std::time::Duration;
use std::io::{Read, Cursor, Write};
use std::ops::Deref;

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt, WriteBytesExt};
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
    app_id: u16,
}

impl A2SClient {
    pub fn new() -> Result<A2SClient> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;

        socket.set_read_timeout(Some(Duration::new(5, 0)))?;
        socket.set_write_timeout(Some(Duration::new(5, 0)))?;

        Ok(A2SClient{
            socket: socket,
            max_size: 1400,
            app_id: 0,
        })
    }

    pub fn max_size(&mut self, size: usize) -> &mut Self {
        self.max_size = size;
        self
    }

    pub fn app_id(&mut self, app_id: u16) -> &mut Self {
        self.app_id = app_id;
        self
    }

    fn send<A: ToSocketAddrs>(&self, payload: &[u8], addr: A) -> Result<Vec<u8>> {
        self.socket.send_to(payload, addr)?;
        
        let mut data = vec![0; self.max_size];

        let read = self.socket.recv(&mut data)?;

        let header = LittleEndian::read_i32(&data);

        if header == SINGLE_PACKET {
            data.remove(0); data.remove(0); data.remove(0); data.remove(0);

            data.truncate(read);

            Ok(data)
        } 

        else if header == MULTI_PACKET {
            // ID - long (4 bytes)
            // Total - byte (1 byte)
            // Number - byte (1 byte)
            // Size - short (2 bytes)

            let id = LittleEndian::read_i32(&data[4..8]);
            let total_packets: usize = data[4] as usize;
            let switching_size: usize = LittleEndian::read_i16(&data[8..10]) as usize;

            let mut packets: Vec<PacketFragment> = Vec::with_capacity(total_packets);

            loop {
                let mut data = vec![0u8; switching_size];

                let read = self.socket.recv(&mut data)?;

                if read < data.len() {
                    data.truncate(read);
                }

                // Skip header field (4 bytes 0..4)
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

            aggregation.remove(0); aggregation.remove(0); aggregation.remove(0); aggregation.remove(0);

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

    fn do_challenge_request<A: ToSocketAddrs>(&self, addr: A, header: &[u8]) -> Result<Vec<u8>> {
        let packet = Vec::with_capacity(9);
        let mut packet = Cursor::new(packet);

        packet.write_all(header)?;
        packet.write_i32::<LittleEndian>(-1)?;

        let data = self.send(packet.get_ref(), &addr)?;
        let mut data = Cursor::new(data);

        let header = data.read_u8()?;
        if header != 'A' as u8 {
            return Err(Error::InvalidResponse);
        }

        let challenge = data.read_i32::<LittleEndian>()?;

        packet.set_position(5);
        packet.write_i32::<LittleEndian>(challenge)?;
        let data = self.send(packet.get_ref(), &addr)?;

        Ok(data)
    }
}

trait ReadCString {
    fn read_cstring(&mut self) -> Result<String>;
}

impl ReadCString for Cursor<Vec<u8>> {
    fn read_cstring(&mut self) -> Result<String> {
        let mut buf = [0; 1];
        let mut str_vec = Vec::with_capacity(256);
        loop {
            self.read(&mut buf)?;
            if buf[0] == 0 { break; } else { str_vec.push(buf[0]); }
        }
        Ok(String::from_utf8_lossy(&str_vec[..]).into_owned())
    }
}
