use bitp::{
    FIELD_LENGTHS,
    bits::{BitReader, BitWriter, PacketSchema},
};
use std::{
    collections::HashSet,
    net::{SocketAddr, UdpSocket},
};

fn main() {
    let skt = UdpSocket::bind("127.0.0.1:34254").expect("Couldn't bind");
    let pkt_schema = PacketSchema::build(FIELD_LENGTHS).unwrap();
    let mut buf = [0; 1024];
    let mut skt_clients = HashSet::<SocketAddr>::new();

    loop {
        let (skt_src_buflen, skt_src) = skt
            .recv_from(&mut buf)
            .expect("Didn't receive data");
        skt_clients.insert(skt_src);

        let buf = &buf[..skt_src_buflen];
        let mut reader = BitReader::new(&pkt_schema, buf);
        let mut writer = BitWriter::new(&pkt_schema);

        for (field_name, _) in pkt_schema.fields.iter() {
            if !reader.isset(field_name) {
                continue;
            }

            let value = reader.read(field_name);
            writer.write(field_name, &value);
            println!("@{skt_src}: {field_name:?}={value}");
        }

        // Broadcast to every client besides the current
        for skt_client in skt_clients.iter() {
            if *skt_client != skt_src {
                skt.send_to(&writer.buf(), skt_client).ok();
            }
        }
    }
}
