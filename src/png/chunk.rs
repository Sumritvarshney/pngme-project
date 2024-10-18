mod chunk_type;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::convert::TryInto;
use std::result::Result;
use std::io::Error;
use crate::chunk_type::ChunkType;
use crc::{Crc, CRC_32_ISO_HDLC};


const CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

pub struct Chunk {
    length: u32,
    chunk_type: ChunkType,
    data: Vec<u8>,
    crc: u32,
}

impl TryFrom<&[u8]> for Chunk {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut iter = value.iter().copied();
        let first4: [u8; 4] = iter.by_ref()
            .take(4).collect::<Vec<u8>>().as_slice().try_into().map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid length"))?;
        let length = u32::from_be_bytes(first4);

        let chunk_type_bytes: [u8; 4] = iter.by_ref()
            .take(4).collect::<Vec<u8>>().as_slice().try_into().map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid length"))?;
        let chunk_type = ChunkType::try_from(chunk_type_bytes).map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid chunk type"))?;

        let data: Vec<u8> = iter.by_ref()
            .take(length as usize).collect();

        let crc_bytes = iter.by_ref()
            .take(4).collect::<Vec<u8>>().as_slice().try_into().map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid length"))?;
        let crc = u32::from_be_bytes(crc_bytes);

        let calculated_crc = CRC.checksum(&Chunk::get_bytes_for_crc(&chunk_type, &data));
        if calculated_crc != crc {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid crc"))
        } else {
            Ok(Chunk {
                length,
                chunk_type,
                data,
                crc,
            })
        }
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.data_as_string().unwrap_or_else(|_| String::new()))
    }
}

impl Chunk {
    pub const DATA_LENGTH_BYTES: usize = 4;
    pub const CHUNK_TYPE_BYTES: usize = 4;
    pub const CRC_BYTES: usize = 4;

    /// Total size of the of the metadata making up a chunk
    pub const METADATA_BYTES: usize =
        Chunk::DATA_LENGTH_BYTES + Chunk::CHUNK_TYPE_BYTES + Chunk::CRC_BYTES;
    pub fn get_bytes_for_crc(chunk_type: &ChunkType, data: &[u8]) -> Vec<u8> {
        let mut container = Vec::new();
        container.extend(chunk_type.bytes());
        container.extend(data);
        container
    }

    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Chunk {
        Chunk {
            length: data.len() as u32,
            crc: CRC.checksum(&Self::get_bytes_for_crc(&chunk_type, &data)),
            chunk_type,
            data,
        }
    }

    pub fn length(&self) -> u32 {
        self.length
    }

    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn crc(&self) -> u32 {
        self.crc
    }

    pub fn data_as_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.data.clone())
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut container = Vec::new();
        container.extend(&self.length.to_be_bytes());
        container.extend(self.chunk_type.bytes());
        container.extend(&self.data);
        container.extend(&self.crc.to_be_bytes());
        container
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();
        
        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!".as_bytes().to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();
        
        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();
        
        let _chunk_string = format!("{}", chunk);
    }
}
