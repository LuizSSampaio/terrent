use std::fs;

use bendy::decoding::{Error, FromBencode};

#[derive(Debug)]
struct BencodeInfo {
    pieces: Vec<u8>,
    piece_length: usize,
    length: usize,
    name: String,
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

#[derive(Debug)]
pub(in crate::file) struct BencodeTorrent {
    announce: String,
    info: BencodeInfo,
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
}
