use std::fs;

use bendy::decoding::{Error, FromBencode};
use sha1::{Digest, Sha1};

#[derive(Debug)]
pub(in crate::file) struct BencodeInfo {
    pub pieces: Vec<u8>,
    pub piece_length: usize,
    pub length: usize,
    pub name: String,
}

impl FromBencode for BencodeInfo {
    fn decode_bencode_object(object: bendy::decoding::Object) -> Result<Self, Error> {
        let mut pieces: Option<Vec<u8>> = None;
        let mut piece_length: Option<usize> = None;
        let mut length: Option<usize> = None;
        let mut name: Option<String> = None;

        let mut dict = object.try_into_dictionary()?;

        while let Some((key, value)) = dict.next_pair()? {
            match key {
                b"pieces" => pieces = Some(value.try_into_bytes()?.to_vec()),
                b"piece length" => {
                    let len_str = value.try_into_integer()?;
                    piece_length = Some(len_str.parse::<usize>()?);
                }
                b"length" => {
                    let len_str = value.try_into_integer()?;
                    length = Some(len_str.parse::<usize>()?);
                }
                b"name" => {
                    let name_bytes = value.try_into_bytes()?;
                    name = Some(String::from_utf8(name_bytes.to_vec())?)
                }
                _ => {}
            }
        }

        Ok(Self {
            pieces: pieces.ok_or_else(|| bendy::decoding::Error::missing_field("pieces"))?,
            piece_length: piece_length
                .ok_or_else(|| bendy::decoding::Error::missing_field("piece length"))?,
            length: length.ok_or_else(|| bendy::decoding::Error::missing_field("length"))?,
            name: name.ok_or_else(|| bendy::decoding::Error::missing_field("name"))?,
        })
    }
}

impl BencodeInfo {
    pub fn hash(&self) -> anyhow::Result<[u8; 20]> {
        let mut buf = Vec::<u8>::new();
        buf.push(b'd');

        Self::encode_int_field(&mut buf, "length", self.length);

        Self::encode_str_field(&mut buf, "name", &self.name);

        Self::encode_int_field(&mut buf, "piece length", self.piece_length);

        Self::encode_bytes_field(&mut buf, "pieces", &self.pieces);

        buf.push(b'e');

        let digest: [u8; 20] = Sha1::digest(&buf).into();
        Ok(digest)
    }

    pub fn split_pieces_hashes(&self) -> anyhow::Result<Vec<[u8; 20]>> {
        const HASH_LEN: usize = 20;
        if self.pieces.len() % HASH_LEN != 0 {
            anyhow::bail!(
                "Received malformed pieces: length {} is not a multiple of 20",
                self.pieces.len()
            );
        }

        let mut out = Vec::with_capacity(self.pieces.len() / HASH_LEN);
        for chunk in self.pieces.chunks(HASH_LEN) {
            let mut arr = [0u8; HASH_LEN];
            arr.copy_from_slice(chunk);
            out.push(arr);
        }
        Ok(out)
    }

    fn encode_str_field(buf: &mut Vec<u8>, key: &str, value: &str) {
        let val_bytes = value.as_bytes();
        buf.extend_from_slice(format!("{}:{}", key.len(), key).as_bytes());
        buf.extend_from_slice(format!("{}:", val_bytes.len()).as_bytes());
        buf.extend_from_slice(val_bytes);
    }

    fn encode_int_field(buf: &mut Vec<u8>, key: &str, value: usize) {
        buf.extend_from_slice(format!("{}:{}", key.len(), key).as_bytes());
        buf.push(b'i');
        buf.extend_from_slice(value.to_string().as_bytes());
        buf.push(b'e');
    }

    fn encode_bytes_field(buf: &mut Vec<u8>, key: &str, bytes: &[u8]) {
        buf.extend_from_slice(format!("{}:{}", key.len(), key).as_bytes());
        buf.extend_from_slice(format!("{}:", bytes.len()).as_bytes());
        buf.extend_from_slice(bytes);
    }
}

#[derive(Debug)]
pub(in crate::file) struct BencodeTorrent {
    pub announce: String,
    pub info: BencodeInfo,
}

impl FromBencode for BencodeTorrent {
    fn decode_bencode_object(object: bendy::decoding::Object) -> Result<Self, Error> {
        let mut announce: Option<String> = None;
        let mut info: Option<BencodeInfo> = None;

        let mut dict = object.try_into_dictionary()?;

        while let Some((key, value)) = dict.next_pair()? {
            match key {
                b"announce" => {
                    let announce_bytes = value.try_into_bytes()?;
                    announce = Some(String::from_utf8(announce_bytes.to_vec())?)
                }
                b"info" => {
                    info = Some(BencodeInfo::decode_bencode_object(value)?);
                }
                _ => {}
            }
        }

        Ok(Self {
            announce: announce.ok_or_else(|| bendy::decoding::Error::missing_field("announce"))?,
            info: info.ok_or_else(|| bendy::decoding::Error::missing_field("info"))?,
        })
    }
}

impl BencodeTorrent {
    pub fn open(file_path: &str) -> anyhow::Result<Self> {
        let content = fs::read(file_path)?;
        let torrent = Self::from_bencode(&content)
            .map_err(|e| anyhow::anyhow!("Failed to decode bencode: {}", e))?;

        Ok(torrent)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    // ---------- helpers ----------
    const INFO_OK: &[u8] =
        b"d6:lengthi12345e4:name4:test12:piece lengthi262144e6:pieces20:abcdefghijklmnopqrste";

    const TORRENT_OK: &[u8] =
        b"d8:announce14:http://tracker4:infod6:lengthi12345e4:name4:test12:piece \
                                 lengthi262144e6:pieces20:abcdefghijklmnopqrstee";

    const INFO_MULTI_PIECES: &[u8] =
        b"d6:lengthi12345e4:name4:test12:piece lengthi262144e6:pieces40:abcdefghijklmnopqrst12345678901234567890e";

    const INFO_BAD_PIECES: &[u8] =
        b"d6:lengthi12345e4:name4:test12:piece lengthi262144e6:pieces19:abcdefghijklmnopqrse";

    #[test]
    fn decode_info_success() {
        let info = BencodeInfo::from_bencode(INFO_OK).expect("should decode");
        assert_eq!(info.length, 12_345);
        assert_eq!(info.piece_length, 262_144);
        assert_eq!(info.name, "test");
        assert_eq!(info.pieces.len(), 20);
        assert_eq!(&info.pieces, b"abcdefghijklmnopqrst");
    }

    #[test]
    fn decode_info_missing_pieces_fails() {
        let bad = b"d6:lengthi12e4:name4:test12:piece lengthi4ee";
        let err = BencodeInfo::from_bencode(bad);
        assert!(
            err.is_err(),
            "decoder must fail if a mandatory field is absent"
        );
    }

    #[test]
    fn decode_torrent_success() {
        let torrent = BencodeTorrent::from_bencode(TORRENT_OK).expect("decode torrent");
        assert_eq!(torrent.announce, "http://tracker");
        assert_eq!(torrent.info.name, "test"); // `info` propagated
        assert_eq!(torrent.info.length, 12_345);
        assert_eq!(torrent.info.pieces.len(), 20);
    }

    #[test]
    fn open_torrent_from_file() {
        // Write fixture to temp file
        let tmpdir = std::env::temp_dir();
        let file_path = tmpdir.join("unit_test.torrent");
        {
            let mut f = std::fs::File::create(&file_path).unwrap();
            f.write_all(TORRENT_OK).unwrap();
        }

        let torrent = BencodeTorrent::open(file_path.to_str().unwrap())
            .expect("open and decode through helper");
        assert_eq!(torrent.announce, "http://tracker");
    }

    #[test]
    fn hash_success() {
        let info = BencodeInfo::from_bencode(INFO_OK).expect("should decode");
        let hash = info.hash().expect("should compute hash");

        // Hash should be exactly 20 bytes
        assert_eq!(hash.len(), 20);

        // Hash should be deterministic - same input produces same output
        let hash2 = info.hash().expect("should compute hash again");
        assert_eq!(hash, hash2);
    }

    #[test]
    fn hash_different_for_different_info() {
        let info1 = BencodeInfo::from_bencode(INFO_OK).expect("should decode");
        let info2 = BencodeInfo::from_bencode(INFO_MULTI_PIECES).expect("should decode");

        let hash1 = info1.hash().expect("should compute hash1");
        let hash2 = info2.hash().expect("should compute hash2");

        // Different inputs should produce different hashes
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn split_pieces_hashes_single_piece() {
        let info = BencodeInfo::from_bencode(INFO_OK).expect("should decode");
        let hashes = info.split_pieces_hashes().expect("should split pieces");

        assert_eq!(hashes.len(), 1);
        assert_eq!(hashes[0], *b"abcdefghijklmnopqrst");
    }

    #[test]
    fn split_pieces_hashes_multiple_pieces() {
        let info = BencodeInfo::from_bencode(INFO_MULTI_PIECES).expect("should decode");
        let hashes = info.split_pieces_hashes().expect("should split pieces");

        assert_eq!(hashes.len(), 2);
        assert_eq!(hashes[0], *b"abcdefghijklmnopqrst");
        assert_eq!(hashes[1], *b"12345678901234567890");
    }

    #[test]
    fn split_pieces_hashes_empty_pieces() {
        let info = BencodeInfo {
            pieces: Vec::new(),
            piece_length: 262144,
            length: 0,
            name: "empty".to_string(),
        };

        let hashes = info
            .split_pieces_hashes()
            .expect("should handle empty pieces");
        assert_eq!(hashes.len(), 0);
    }

    #[test]
    fn split_pieces_hashes_malformed_length_fails() {
        let info = BencodeInfo::from_bencode(INFO_BAD_PIECES).expect("should decode");
        let result = info.split_pieces_hashes();

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not a multiple of 20"));
        assert!(err_msg.contains("19"));
    }

    #[test]
    fn split_pieces_hashes_odd_length_fails() {
        let info = BencodeInfo {
            pieces: vec![1, 2, 3, 4, 5], // 5 bytes, not multiple of 20
            piece_length: 262144,
            length: 12345,
            name: "test".to_string(),
        };

        let result = info.split_pieces_hashes();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not a multiple of 20"));
        assert!(err_msg.contains("5"));
    }

    #[test]
    fn split_pieces_hashes_large_number_of_pieces() {
        // Create pieces for 100 hashes (100 * 20 = 2000 bytes)
        let mut pieces = Vec::with_capacity(2000);
        for i in 0..100 {
            // Fill each 20-byte chunk with the same byte value
            let byte_val = (i % 256) as u8;
            pieces.extend(std::iter::repeat_n(byte_val, 20));
        }

        let info = BencodeInfo {
            pieces,
            piece_length: 262144,
            length: 12345,
            name: "test".to_string(),
        };

        let hashes = info
            .split_pieces_hashes()
            .expect("should split many pieces");
        assert_eq!(hashes.len(), 100);

        // Verify each hash is filled with the expected byte value
        for (i, hash) in hashes.iter().enumerate() {
            let expected_byte = (i % 256) as u8;
            assert!(
                hash.iter().all(|&b| b == expected_byte),
                "Hash {i} should be filled with byte {expected_byte}"
            );
        }
    }
}
