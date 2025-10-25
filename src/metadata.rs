#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Metadata {
    pub name: String,
    pub piece_length: u64,
    pub pieces: Vec<[u8; 20]>,
    pub private: Option<usize>,

    pub announce: Vec<String>,

    created_by: Option<String>,
    creation_date: Option<u64>,
    comment: Option<String>,
    encoding: Option<String>,
}
