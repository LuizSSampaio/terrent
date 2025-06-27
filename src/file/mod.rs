mod bencode;

#[derive(Debug)]
pub struct TorrentFile {
    annouce: String,
    info_hash: [u8; 20],
    piece_hashes: Vec<[u8; 20]>,
    piece_length: usize,
    length: usize,
    name: String,
}

impl TorrentFile {}
