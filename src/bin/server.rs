use sockets::{
    FIELD_LENGTHS,
    bits::{BitReader, BitWriter, FieldName, PacketSchema},
};
use std::net::UdpSocket;

fn main() {
    let skt = UdpSocket::bind("127.0.0.1:34254").expect("Couldn't bind");
    let pkt_schema = PacketSchema::build(FIELD_LENGTHS).unwrap();
    let mut buf = [0; 1024];

    loop {
        let (byts_len, skt_adr) = skt
            .recv_from(&mut buf)
            .expect("Didn't receive data");
        let buf = &buf[..byts_len];

        let mut reader = BitReader::new(&pkt_schema, buf);
        for (field_name, _) in pkt_schema.fields.iter() {
            if reader.isset(field_name) {
                let value = reader.read(field_name);
                println!("From {skt_adr}: {field_name:?} = {value}");
            }
        }

        // Respond to whichever client just sent this
        let mut writer = BitWriter::new(&pkt_schema);
        writer.write(FieldName::IsJumping, 1);
        skt.send_to(&writer.buf(), skt_adr)
            .unwrap_or_else(|_| panic!("Couldn't send data to {}", skt_adr));
    }
}
