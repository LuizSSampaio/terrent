use anyhow::{Context, Ok};
use bencode::BencodeTorrent;
use percent_encoding::{NON_ALPHANUMERIC, percent_encode};
use url::{Url, form_urlencoded};

mod bencode;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TorrentFile {
    announce: String,
    info_hash: [u8; 20],
    piece_hashes: Vec<[u8; 20]>,
    piece_length: usize,
    length: usize,
    name: String,
}

impl TorrentFile {
    pub fn open(file_path: &str) -> anyhow::Result<Self> {
        let bencode = BencodeTorrent::open(file_path)?;

        Self::try_from(bencode)
    }

    pub fn build_tracker_url(&self, peer_id: [u8; 20], port: u16) -> anyhow::Result<String> {
        let mut url = Url::parse(&self.announce)
            .with_context(|| format!("Invalid announce URL: {}", self.announce))?;

        let info_hash_enc = percent_encode(&self.info_hash, NON_ALPHANUMERIC).to_string();
        let peer_id_enc = percent_encode(&peer_id, NON_ALPHANUMERIC).to_string();

        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("info_hash", &info_hash_enc)
            .append_pair("peer_id", &peer_id_enc)
            .append_pair("port", &port.to_string())
            .append_pair("uploaded", "0")
            .append_pair("downloaded", "0")
            .append_pair("compact", "1")
            .append_pair("left", &self.length.to_string())
            .finish();

        url.set_query(Some(&query));
        Ok(url.to_string())
    }
}

impl TryFrom<BencodeTorrent> for TorrentFile {
    fn try_from(value: BencodeTorrent) -> anyhow::Result<Self> {
        Ok(Self {
            announce: value.announce,
            info_hash: value.info.hash()?,
            piece_hashes: value.info.split_pieces_hashes()?,
            piece_length: value.info.piece_length,
            length: value.info.length,
            name: value.info.name,
        })
    }

    type Error = anyhow::Error;
}

#[cfg(test)]
mod tests {
    use super::*;
    use bencode::BencodeTorrent;
    use bendy::decoding::FromBencode;
    use std::io::Write;

    // ---- helpers ----
    const TORRENT_OK: &[u8] =
        b"d8:announce14:http://tracker4:infod6:lengthi12345e4:name4:test12:piece \
        lengthi262144e6:pieces20:abcdefghijklmnopqrstee";

    const TORRENT_HTTPS: &[u8] =
        b"d8:announce26:https://secure.tracker.com4:infod6:lengthi54321e4:name9:test-file12:piece \
        lengthi524288e6:pieces40:abcdefghijklmnopqrst12345678901234567890ee";

    fn create_test_torrent_file() -> TorrentFile {
        TorrentFile {
            announce: "http://tracker.example.com/announce".to_string(),
            info_hash: [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ],
            piece_hashes: vec![[
                21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
            ]],
            piece_length: 262144,
            length: 1000000,
            name: "example.txt".to_string(),
        }
    }

    #[test]
    fn open_torrent_success() {
        // Write fixture to temp file
        let tmpdir = std::env::temp_dir();
        let file_path = tmpdir.join("test_torrent.torrent");
        {
            let mut f = std::fs::File::create(&file_path).unwrap();
            f.write_all(TORRENT_OK).unwrap();
        }

        let torrent = TorrentFile::open(file_path.to_str().unwrap())
            .expect("should open and convert torrent file");

        assert_eq!(torrent.announce, "http://tracker");
        assert_eq!(torrent.length, 12345);
        assert_eq!(torrent.piece_length, 262144);
        assert_eq!(torrent.name, "test");
        assert_eq!(torrent.piece_hashes.len(), 1);
        assert_eq!(torrent.info_hash.len(), 20);
    }

    #[test]
    fn open_torrent_file_not_found() {
        let result = TorrentFile::open("nonexistent_file.torrent");
        assert!(result.is_err(), "should fail when file doesn't exist");
    }

    #[test]
    fn open_torrent_invalid_bencode() {
        // Write invalid content to temp file
        let tmpdir = std::env::temp_dir();
        let file_path = tmpdir.join("invalid_torrent.torrent");
        {
            let mut f = std::fs::File::create(&file_path).unwrap();
            f.write_all(b"invalid bencode content").unwrap();
        }

        let result = TorrentFile::open(file_path.to_str().unwrap());
        assert!(result.is_err(), "should fail with invalid bencode content");
    }

    #[test]
    fn build_tracker_url_success() {
        let torrent = create_test_torrent_file();
        let peer_id = [65; 20]; // "AAAAAAAAAAAAAAAAAAAA"
        let port = 6881;

        let url = torrent
            .build_tracker_url(peer_id, port)
            .expect("should build tracker URL");

        // Verify the base URL
        assert!(url.starts_with("http://tracker.example.com/announce?"));

        // Verify required parameters are present
        assert!(url.contains("info_hash="));
        assert!(url.contains("peer_id="));
        assert!(url.contains("port=6881"));
        assert!(url.contains("uploaded=0"));
        assert!(url.contains("downloaded=0"));
        assert!(url.contains("compact=1"));
        assert!(url.contains("left=1000000"));
    }

    #[test]
    fn build_tracker_url_https() {
        let torrent = BencodeTorrent::from_bencode(TORRENT_HTTPS).expect("should decode torrent");
        let torrent_file = TorrentFile::try_from(torrent).expect("should convert to TorrentFile");

        let peer_id = [66; 20]; // "BBBBBBBBBBBBBBBBBBBB"
        let port = 8080;

        let url = torrent_file
            .build_tracker_url(peer_id, port)
            .expect("should build HTTPS tracker URL");

        assert!(url.starts_with("https://secure.tracker.com/?"));
        assert!(url.contains("port=8080"));
    }

    #[test]
    fn build_tracker_url_different_ports() {
        let torrent = create_test_torrent_file();
        let peer_id = [67; 20];

        // Test various port numbers
        for port in [1, 80, 443, 6881, 65535] {
            let url = torrent
                .build_tracker_url(peer_id, port)
                .expect("should build URL with different ports");
            assert!(url.contains(&format!("port={port}")));
        }
    }

    #[test]
    fn build_tracker_url_different_peer_ids() {
        let torrent = create_test_torrent_file();
        let port = 6881;

        // Test different peer IDs
        let peer_ids = [
            [0; 20],   // All zeros
            [255; 20], // All max values
            [
                65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84,
            ], // Mixed
        ];

        for peer_id in peer_ids {
            let url = torrent
                .build_tracker_url(peer_id, port)
                .expect("should build URL with different peer IDs");
            assert!(url.contains("peer_id="));
        }
    }

    #[test]
    fn build_tracker_url_encoding() {
        let mut torrent = create_test_torrent_file();
        // Use info_hash with bytes that need URL encoding
        torrent.info_hash = [
            0, 32, 127, 128, 255, 10, 13, 37, 38, 63, 35, 43, 61, 47, 92, 34, 39, 60, 62, 124,
        ];

        let peer_id = [
            0, 32, 127, 128, 255, 10, 13, 37, 38, 63, 35, 43, 61, 47, 92, 34, 39, 60, 62, 124,
        ];
        let port = 6881;

        let url = torrent
            .build_tracker_url(peer_id, port)
            .expect("should build URL with encoding");

        // Should contain percent-encoded values
        assert!(url.contains("%"));
        // Should not contain raw special characters that need encoding
        assert!(!url.contains(" "));
        assert!(!url.contains("\n"));
        assert!(!url.contains("\r"));
    }

    #[test]
    fn build_tracker_url_invalid_announce_fails() {
        let mut torrent = create_test_torrent_file();
        torrent.announce = "not-a-valid-url".to_string();

        let peer_id = [68; 20];
        let port = 6881;

        let result = torrent.build_tracker_url(peer_id, port);
        assert!(result.is_err(), "should fail with invalid announce URL");

        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Invalid announce URL"));
        assert!(err_msg.contains("not-a-valid-url"));
    }

    #[test]
    fn try_from_bencode_torrent_success() {
        let bencode = BencodeTorrent::from_bencode(TORRENT_OK).expect("should decode bencode");
        let torrent = TorrentFile::try_from(bencode).expect("should convert from BencodeTorrent");

        assert_eq!(torrent.announce, "http://tracker");
        assert_eq!(torrent.length, 12345);
        assert_eq!(torrent.piece_length, 262144);
        assert_eq!(torrent.name, "test");
        assert_eq!(torrent.piece_hashes.len(), 1);
        assert_eq!(torrent.info_hash.len(), 20);
    }
}
