use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Peer {
    ip: IpAddr,
    port: u16,
}

impl Peer {
    pub fn unmarshal(peers_bin: &[u8]) -> anyhow::Result<Vec<Peer>> {
        const PEER_SIZE: usize = 6;
        const PEER_IP_SIZE: usize = 4;

        if peers_bin.len() % PEER_SIZE != 0 {
            anyhow::bail!(
                "Received malformed peers list (length not divisible by {})",
                PEER_SIZE
            );
        }

        let num_peers = peers_bin.len() / PEER_SIZE;
        let mut peers = Vec::with_capacity(num_peers);

        for i in 0..num_peers {
            let offset = i * PEER_SIZE;

            let ip_bytes = &peers_bin[offset..offset + PEER_IP_SIZE];
            let ip = IpAddr::V4(Ipv4Addr::new(
                ip_bytes[0],
                ip_bytes[1],
                ip_bytes[2],
                ip_bytes[3],
            ));

            let port_bytes = &peers_bin[offset + PEER_IP_SIZE..offset + PEER_SIZE];
            let port = u16::from_be_bytes([port_bytes[0], port_bytes[1]]);

            peers.push(Self { ip, port });
        }

        Ok(peers)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    // Empty slice – should return an empty Vec
    const PEERS_EMPTY: &[u8] = &[];

    // One peer  : 1.2.3.4:6881   (6881 = 0x1AE1)
    const PEERS_SINGLE: &[u8] = &[1, 2, 3, 4, 0x1A, 0xE1];

    // Two peers : 1.1.1.1:80  |  8.8.8.8:6881
    const PEERS_TWO: &[u8] = &[
        1, 1, 1, 1, 0x00, 0x50, // 1.1.1.1:80
        8, 8, 8, 8, 0x1A, 0xE1, // 8.8.8.8:6881
    ];

    // Malformed slice – 5 bytes (not divisible by 6)
    const PEERS_BAD_LEN: &[u8] = &[127, 0, 0, 1, 0x1A];

    #[test]
    fn unmarshall_empty_slice() {
        let peers = Peer::unmarshal(PEERS_EMPTY).expect("empty slice must decode");
        assert_eq!(peers.len(), 0);
    }

    #[test]
    fn unmarshall_single_peer_success() {
        let peers = Peer::unmarshal(PEERS_SINGLE).expect("single peer must decode");
        assert_eq!(peers.len(), 1);

        let p = &peers[0];
        assert_eq!(p.ip, IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)));
        assert_eq!(p.port, 6881);
    }

    #[test]
    fn unmarshall_multiple_peers_success() {
        let peers = Peer::unmarshal(PEERS_TWO).expect("two peers must decode");
        assert_eq!(peers.len(), 2);

        assert_eq!(peers[0].ip, IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)));
        assert_eq!(peers[0].port, 80);

        assert_eq!(peers[1].ip, IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)));
        assert_eq!(peers[1].port, 6881);
    }

    #[test]
    fn unmarshall_malformed_length_fails() {
        let err = Peer::unmarshal(PEERS_BAD_LEN);
        assert!(
            err.is_err(),
            "decoder must reject non-multiple-of-6 lengths"
        );

        // Optional: inspect error message content
        let msg = err.unwrap_err().to_string();
        assert!(
            msg.to_lowercase().contains("malformed") || msg.to_lowercase().contains("divisible"),
            "error message should hint at malformed length, got: {msg}"
        );
    }
}
