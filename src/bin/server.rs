use bitp::{
    FIELD_LENGTHS,
    bits::{BitReader, BitWriter, FieldName, PacketSchema},
};
use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
};
use tap::Pipe;

fn main() {
    let skt = UdpSocket::bind("0.0.0.0:34254").expect("Couldn't bind");
    let pkt_schema = PacketSchema::build(FIELD_LENGTHS).unwrap();
    let mut buf = [0; 1024];

    let mut skt_clients = HashMap::<SocketAddr, u32>::new();
    let mut next_sess_id: u32 = 0;

    loop {
        let (skt_src_buflen, skt_src) = skt
            .recv_from(&mut buf)
            .expect("Didn't receive data");

        // Bail early on packets too short to even hold the mask bytes the
        // schema expects.
        // Protects `BitReader::new()`'s `split_at()` from panicking and
        // taking the whole server down over one client's bad packet.
        if skt_src_buflen < pkt_schema.fields.len().div_ceil(8) {
            continue;
        }
        let sess_id = *skt_clients.entry(skt_src).or_insert_with(|| {
            next_sess_id.pipe(|id| {
                next_sess_id += 1;
                id
            })
        });

        let buf = &buf[..skt_src_buflen];
        let mut reader = BitReader::new(&pkt_schema, buf);
        let mut writer = BitWriter::new(&pkt_schema);

        writer.write(&FieldName::SessionId, &sess_id);

        for (field_name, _) in pkt_schema.fields.iter() {
            if !reader.isset(field_name) {
                continue;
            }

            let value = reader.read(field_name);
            writer.write(field_name, &value);
            println!("@{skt_src} #{sess_id}: {field_name:?}={value}");
        }

        // Broadcast to every client besides the current
        for skt_client in skt_clients.keys() {
            if *skt_client != skt_src {
                skt.send_to(&writer.buf(), skt_client).ok();
            }
        }
    }
}
