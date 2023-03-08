pub mod errors;
pub mod info;
pub mod players;
pub mod rules;

use std::io::{Cursor, Read, Write};
#[cfg(not(feature = "async"))]
use std::net::{ToSocketAddrs, UdpSocket};
use std::ops::Deref;
use std::time::Duration;

#[cfg(feature = "async")]
use tokio::net::{ToSocketAddrs, UdpSocket};
#[cfg(feature = "async")]
use tokio::time;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use bzip2::read::BzDecoder;
use crc::crc32;

use crate::errors::{Error, Result};

const SINGLE_PACKET: i32 = -1;
const MULTI_PACKET: i32 = -2;

// Offsets
const OFS_HEADER: usize             =  0;
const OFS_SP_PAYLOAD: usize         =  4;
const OFS_MP_ID: usize              =  4;
const OFS_MP_SS_TOTAL: usize        =  8;
const OFS_MP_SS_NUMBER: usize       =  9;
const OFS_MP_SS_SIZE: usize         = 10;
const OFS_MP_SS_BZ2_SIZE: usize     = 12;
const OFS_MP_SS_BZ2_CRC: usize      = 16;
const OFS_MP_SS_PAYLOAD: usize      = OFS_MP_SS_BZ2_SIZE;
const OFS_MP_SS_PAYLOAD_BZ2: usize  = OFS_MP_SS_BZ2_CRC + 4;

macro_rules! read_buffer_offset {
    ($buf:expr, $offset:expr, i8) => (
        $buf[$offset].into()
    );
    ($buf:expr, $offset:expr, u8) => (
        $buf[$offset].into()
    );
    ($buf:expr, $offset:expr, i16) => (
        i16::from_le_bytes([$buf[$offset], $buf[$offset + 1]])
    );
    ($buf:expr, $offset:expr, u16) => (
        u16::from_le_bytes([$buf[$offset], $buf[$offset + 1]])
    );
    ($buf:expr, $offset:expr, i32) => (
        i32::from_le_bytes([$buf[$offset], $buf[$offset + 1], $buf[$offset + 2], $buf[$offset + 3]])
    );
    ($buf:expr, $offset:expr, u32) => (
        u32::from_le_bytes([$buf[$offset], $buf[$offset + 1], $buf[$offset + 2], $buf[$offset + 3]])
    );
    ($buf:expr, $offset:expr, i64) => (
        i64::from_le_bytes([$buf[$offset], $buf[$offset + 1], $buf[$offset + 2], $buf[$offset + 3],
                            $buf[$offset + 4], $buf[$offset + 5], $buf[$offset + 6], $buf[$offset + 7]])
    );
    ($buf:expr, $offset:expr, u64) => (
        u64::from_le_bytes([$buf[$offset], $buf[$offset + 1], $buf[$offset + 2], $buf[$offset + 3],
                            $buf[$offset + 4], $buf[$offset + 5], $buf[$offset + 6], $buf[$offset + 7]])
    )
}

#[derive(Debug)]
struct PacketFragment {
    number: u8,
    payload: Vec<u8>,
}

pub struct A2SClient {
    socket: UdpSocket,
    #[cfg(feature = "async")]
    timeout: Duration,
    max_size: usize,
    app_id: u16,
}

#[cfg(feature = "async")]
macro_rules! future_timeout {
    ($timeout:expr, $future:expr) => {
        match time::timeout($timeout, $future).await {
            Ok(value) => value,
            Err(_) => return Err(Error::ErrTimeout),
        }
    };
}

impl A2SClient {
    #[cfg(not(feature = "async"))]
    pub fn new() -> Result<A2SClient> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        let timeout = Duration::new(5, 0);

        socket.set_read_timeout(Some(timeout))?;
        socket.set_write_timeout(Some(timeout))?;

        Ok(A2SClient {
            socket,
            max_size: 1400,
            app_id: 0,
        })
    }

    #[cfg(feature = "async")]
    pub async fn new() -> Result<A2SClient> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;

        Ok(A2SClient {
            socket: socket,
            timeout: Duration::new(5, 0),
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

    #[cfg(feature = "async")]
    async fn send<A: ToSocketAddrs>(&self, payload: &[u8], addr: A) -> Result<Vec<u8>> {
        future_timeout!(self.timeout, self.socket.send_to(payload, addr))?;

        let mut data = vec![0; self.max_size];

        let read = future_timeout!(self.timeout, self.socket.recv(&mut data))?;
        data.truncate(read);

        let header = read_buffer_offset!(&data, OFS_HEADER, i32);

        if header == SINGLE_PACKET {
            Ok(data[OFS_SP_PAYLOAD..].to_vec())
        } else if header == MULTI_PACKET {
            // ID - long (4 bytes)
            // Total - byte (1 byte)
            // Number - byte (1 byte)
            // Size - short (2 bytes)

            let id = read_buffer_offset!(&data, OFS_MP_ID, i32);
            let total_packets: usize = data[OFS_MP_SS_TOTAL].into();
            let switching_size: usize = read_buffer_offset!(&data, OFS_MP_SS_SIZE, u16).into();

            // Sanity check
            if (switching_size > self.max_size) || (total_packets > 32) {
                return Err(Error::InvalidResponse)
            }

            let mut packets: Vec<PacketFragment> = Vec::with_capacity(0);
            packets.try_reserve(total_packets)?;
            packets.push(PacketFragment {
                number: data[OFS_MP_SS_NUMBER],
                // The first packet seems to include a single packet header (0xFFFFFFFF) for some
                // reason, so we'd rather skip that (hence +4)
                payload: Vec::from(&data[OFS_MP_SS_PAYLOAD+4..]),
            });

            loop {
                let mut data: Vec<u8> = Vec::with_capacity(0);
                data.try_reserve(switching_size)?;
                data.resize(switching_size, 0);

                let read = future_timeout!(self.timeout, self.socket.recv(&mut data))?;
                data.truncate(read);

                if data.len() <= 9 {
                    Err(Error::InvalidResponse)?
                }

                let packet_id = read_buffer_offset!(&data, OFS_MP_ID, i32);

                if packet_id != id {
                    return Err(Error::MismatchID);
                }

                if id as u32 & 0x80000000 == 0 {
                    // Uncompressed packet
                    packets.push(PacketFragment {
                        number: data[OFS_MP_SS_NUMBER],
                        payload: Vec::from(&data[OFS_MP_SS_PAYLOAD..]),
                    });
                }
                else {
                    // BZip2 compressed packet
                    packets.push(PacketFragment {
                        number: data[OFS_MP_SS_NUMBER],
                        payload: Vec::from(&data[OFS_MP_SS_PAYLOAD_BZ2..]),
                    });
                }

                if packets.len() == total_packets {
                    break;
                }
            }

            packets.sort_by_key(|p| p.number);

            let mut aggregation = Vec::with_capacity(0);
            aggregation.try_reserve(total_packets * self.max_size)?;

            for p in packets {
                aggregation.extend(p.payload);
            }

            if id as u32 & 0x80000000 != 0 {
                let decompressed_size = read_buffer_offset!(&data, OFS_MP_SS_BZ2_SIZE, u32);
                let checksum = read_buffer_offset!(&data, OFS_MP_SS_BZ2_CRC, u32);

                if decompressed_size > (1024 * 1024) {
                    return Err(Error::InvalidBz2Size);
                }

                let mut decompressed = Vec::with_capacity(0);
                decompressed.try_reserve(decompressed_size as usize)?;
                decompressed.resize(decompressed_size as usize, 0);

                BzDecoder::new(aggregation.deref()).read_exact(&mut decompressed)?;

                if crc32::checksum_ieee(&decompressed) != checksum {
                    return Err(Error::CheckSumMismatch);
                }

                Ok(decompressed)
            } else {
                Ok(aggregation)
            }
        } else {
            Err(Error::InvalidResponse)
        }
    }

    #[cfg(feature = "async")]
    async fn do_challenge_request<A: ToSocketAddrs>(
        &self,
        addr: A,
        header: &[u8],
    ) -> Result<Vec<u8>> {
        let packet = Vec::with_capacity(9);
        let mut packet = Cursor::new(packet);

        packet.write_all(header)?;
        packet.write_i32::<LittleEndian>(-1)?;

        let data = self.send(packet.get_ref(), &addr).await?;
        let mut data = Cursor::new(data);

        let header = data.read_u8()?;
        if header != 'A' as u8 {
            return Err(Error::InvalidResponse);
        }

        let challenge = data.read_i32::<LittleEndian>()?;

        packet.set_position(5);
        packet.write_i32::<LittleEndian>(challenge)?;
        let data = self.send(packet.get_ref(), &addr).await?;

        Ok(data)
    }

    #[cfg(not(feature = "async"))]
    fn send<A: ToSocketAddrs>(&self, payload: &[u8], addr: A) -> Result<Vec<u8>> {
        self.socket.send_to(payload, addr)?;

        let mut data = vec![0; self.max_size];

        let read = self.socket.recv(&mut data)?;
        data.truncate(read);

        let header = read_buffer_offset!(&data, OFS_HEADER, i32);

        if header == SINGLE_PACKET {
            Ok(data[OFS_SP_PAYLOAD..].to_vec())
        } else if header == MULTI_PACKET {
            // ID - long (4 bytes)
            // Total - byte (1 byte)
            // Number - byte (1 byte)
            // Size - short (2 bytes)

            let id = read_buffer_offset!(&data, OFS_MP_ID, i32);
            let total_packets: usize = data[OFS_MP_SS_TOTAL].into();
            let switching_size: usize = read_buffer_offset!(&data, OFS_MP_SS_SIZE, u16).into();

            // Sanity check
            if (switching_size > self.max_size) || (total_packets > 32) {
                return Err(Error::InvalidResponse)
            }

            let mut packets: Vec<PacketFragment> = Vec::with_capacity(0);
            packets.try_reserve(total_packets)?;
            packets.push(PacketFragment {
                number: data[OFS_MP_SS_NUMBER],
                // The first packet seems to include a single packet header (0xFFFFFFFF) for some
                // reason, so we'd rather skip that (hence +4)
                payload: Vec::from(&data[OFS_MP_SS_PAYLOAD+4..]),
            });

            loop {
                let mut data: Vec<u8> = Vec::with_capacity(0);
                data.try_reserve(switching_size)?;
                data.resize(switching_size, 0);

                let read = self.socket.recv(&mut data)?;
                data.truncate(read);

                if data.len() <= 9 {
                    Err(Error::InvalidResponse)?
                }

                let packet_id = read_buffer_offset!(&data, OFS_MP_ID, i32);

                if packet_id != id {
                    return Err(Error::MismatchID);
                }

                if id as u32 & 0x80000000 == 0 {
                    // Uncompressed packet
                    packets.push(PacketFragment {
                        number: data[OFS_MP_SS_NUMBER],
                        payload: Vec::from(&data[OFS_MP_SS_PAYLOAD..]),
                    });
                }
                else {
                    // BZip2 compressed packet
                    packets.push(PacketFragment {
                        number: data[OFS_MP_SS_NUMBER],
                        payload: Vec::from(&data[OFS_MP_SS_PAYLOAD_BZ2..]),
                    });
                }

                if packets.len() == total_packets {
                    break;
                }
            }

            packets.sort_by_key(|p| p.number);

            let mut aggregation = Vec::with_capacity(0);
            aggregation.try_reserve(total_packets * self.max_size)?;

            for p in packets {
                aggregation.extend(p.payload);
            }

            if id as u32 & 0x80000000 != 0 {
                let decompressed_size = read_buffer_offset!(&data, OFS_MP_SS_BZ2_SIZE, u32);
                let checksum = read_buffer_offset!(&data, OFS_MP_SS_BZ2_CRC, u32);

                if decompressed_size > (1024 * 1024) {
                    return Err(Error::InvalidBz2Size);
                }

                let mut decompressed = Vec::with_capacity(0);
                decompressed.try_reserve(decompressed_size as usize)?;
                decompressed.resize(decompressed_size as usize, 0);

                BzDecoder::new(aggregation.deref()).read_exact(&mut decompressed)?;

                if crc32::checksum_ieee(&decompressed) != checksum {
                    return Err(Error::CheckSumMismatch);
                }

                Ok(decompressed)
            } else {
                Ok(aggregation)
            }
        } else {
            Err(Error::InvalidResponse)
        }
    }

    #[cfg(not(feature = "async"))]
    fn do_challenge_request<A: ToSocketAddrs>(&self, addr: A, header: &[u8]) -> Result<Vec<u8>> {
        let packet = Vec::with_capacity(9);
        let mut packet = Cursor::new(packet);

        packet.write_all(header)?;
        packet.write_i32::<LittleEndian>(-1)?;

        let data = self.send(packet.get_ref(), &addr)?;
        let mut data = Cursor::new(data);

        let header = data.read_u8()?;
        if header != b'A' {
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
        let end = self.get_ref().len() as u64;
        let mut buf = [0; 1];
        let mut str_vec = Vec::with_capacity(256);
        while self.position() < end {
            self.read_exact(&mut buf)?;
            if buf[0] == 0 {
                break;
            } else {
                str_vec.push(buf[0]);
            }
        }
        Ok(String::from_utf8_lossy(&str_vec[..]).into_owned())
    }
}
