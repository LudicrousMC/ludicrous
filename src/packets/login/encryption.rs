use super::super::{
    write_varint, Packet, PacketMode, PacketStatic, Player, PlayerReadConn, PlayerStream,
    PlayerWriteConn, ENCRYPTION_DATA,
};
use openssl::symm::{Cipher, Crypter, Mode};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct Encryption {
    encryption_res: (Option<Crypter>, Option<Crypter>),
    verify_token: [u8; 16],
}

impl Encryption {
    pub fn new() -> Self {
        Encryption {
            encryption_res: (None, None),
            verify_token: [0; 16],
        }
    }
}

impl PacketStatic for Encryption {
    const CLIENTBOUND_ID: i32 = 0x01;
    const PACKET_MODE: PacketMode = PacketMode::SendThenReceive;
}

#[async_trait::async_trait]
impl Packet for Encryption {
    fn mode(&self) -> PacketMode {
        Self::PACKET_MODE
    }

    async fn handle(&mut self, conn: &mut PlayerStream) {
        self.send(&mut conn.write).await;
        self.receive(&mut conn.read).await;
        conn.write.encryptor = self.encryption_res.0.take();
        conn.read.decryptor = self.encryption_res.1.take();
    }

    async fn send(&mut self, write_conn: &mut PlayerWriteConn) {
        let public_key = ENCRYPTION_DATA.get().unwrap().public_key.clone();
        (0..16).for_each(|i| self.verify_token[i] = rand::random());
        // Encryption Request Packet (S -> C)
        let mut encryption_packet = write_varint(Self::CLIENTBOUND_ID);
        encryption_packet.extend(write_varint(0x00)); // Empty protobuf string
        encryption_packet.extend(write_varint(public_key.len() as i32));
        encryption_packet.extend(public_key.clone());
        encryption_packet.extend(write_varint(self.verify_token.len() as i32));
        encryption_packet.extend(self.verify_token);
        encryption_packet.push(0x00);
        write_conn.write_packet(encryption_packet).await;
    }

    async fn receive(&mut self, read_conn: &mut PlayerReadConn) {
        // Encryption Response Packet (C -> S)
        let _encryption_response_len = read_conn.read_varint().await;
        let _encryption_response_id = read_conn.read_varint().await;
        let shared_secret_len = read_conn.read_varint().await;
        let mut shared_secret = vec![0u8; shared_secret_len as usize];
        read_conn
            .socket_read
            .read_exact(&mut shared_secret)
            .await
            .unwrap();
        let verify_token_len = read_conn.read_varint().await;
        let mut verify_token = vec![0u8; verify_token_len as usize];
        read_conn
            .socket_read
            .read_exact(&mut verify_token)
            .await
            .unwrap();

        let shared_secret = ENCRYPTION_DATA
            .get()
            .unwrap()
            .private_key
            .decrypt(rsa::Pkcs1v15Encrypt, &shared_secret)
            .unwrap();
        let verify_token = ENCRYPTION_DATA
            .get()
            .unwrap()
            .private_key
            .decrypt(rsa::Pkcs1v15Encrypt, &verify_token)
            .unwrap();
        let mut encryptor = Crypter::new(
            Cipher::aes_128_cfb8(),
            Mode::Encrypt,
            &shared_secret,
            Some(&shared_secret),
        )
        .unwrap();
        encryptor.pad(false);
        let mut decryptor = Crypter::new(
            Cipher::aes_128_cfb8(),
            Mode::Decrypt,
            &shared_secret,
            Some(&shared_secret),
        )
        .unwrap();
        decryptor.pad(false);

        self.encryption_res = (Some(encryptor), Some(decryptor));
    }
}
