//! cargo run --bin intent_test -- [server_addr] [hold|burst] [burst_n]
//!
//! hold  (default): join, then 3s of mv_z = -1 at 60Hz, then 2s of zero
//!                  intents.
//!                  Server log: one "consumed" line per tick, seq counting up,
//!                  pos.z falling while held, then flat after the zero
//!                  intents.
//! burst:           join, then `burst_n` (default 8) intents in the same
//!                  instant, 1s of silence, then one zero intent to stop the
//!                  player.
//!                  Server log: `burst_n` consecutive "consumed" lines on
//!                  consecutive ticks (the queue draining at 60Hz). The server
//!                  queue holds at most 8 (QUEUE_MAX): with burst_n > 8 the
//!                  oldest are dropped and the first consumed seq is
//!                  burst_n - 7.
//!
//! The client only sends (and acks the server's reliable replies, so the
//! server stops retrying). Verification happens in the server logs.

use std::{
    env,
    net::UdpSocket,
    thread,
    time::{Duration, Instant},
};

use bitp::{
    bits::{BitReader, BitWriter, DecodeType, PacketSchema},
    logt,
    pkt::{FIELD_LENGTHS, FieldName, PacketKind},
    quantize::intent::{MV, YAW},
    sim::{Edges, Held, PlayerIntentSim},
};

const TICK: Duration = Duration::from_micros(16_667);

fn main() {
    let mut args = env::args().skip(1);
    let server = args.next().unwrap();
    let mode = args.next().unwrap();
    let burst_n: u8 = args.next().and_then(|s| s.parse().ok()).unwrap();

    let skt = UdpSocket::bind("0.0.0.0:0").expect("bind failed");
    let schema = PacketSchema::build(FIELD_LENGTHS).unwrap();

    spawn_recv_thread(&skt, &schema);

    skt.send_to(&join_pkt(&schema), &server)
        .expect("join send failed");
    logt!("join sent, waiting 500ms for registration/spawn");
    thread::sleep(Duration::from_millis(500));

    let mut seq: u8 = 1;
    let mut send = |mv_z: f32| {
        skt.send_to(
            &intent_pkt(
                &schema,
                seq,
                PlayerIntentSim {
                    held: Held {
                        mv_x: 0.0,
                        mv_z,
                        yaw: 0.0,
                        crouch: false,
                    },
                    edges: Edges { jump: false },
                },
            ),
            &server,
        )
        .expect("send failed");
        seq = seq.wrapping_add(1);
    };

    match mode.as_str() {
        "hold" => {
            let mut next = Instant::now();
            for i in 0..5 * 60 {
                send(if i < 3 * 60 { -1.0 } else { 0.0 });

                next += TICK; // drift-free deadline loop
                thread::sleep(next.saturating_duration_since(Instant::now()));
            }
        }
        "burst" => {
            for _ in 0..burst_n {
                send(-1.0);
            }
            // After the queue drains the server keeps repeating the last held
            // intent (mv_z = -1), so the player keeps walking. Stop it
            // explicitly.
            thread::sleep(Duration::from_secs(1));
            send(0.0);
            thread::sleep(Duration::from_millis(300));
        }
        other => logt!("unknown mode '{other}' (use hold|burst)"),
    }

    logt!("done");
}

/// Logs every reliable Data/Single packet the server sends and acks it
/// (otherwise the server's retry loop keeps resending welcome/spawn).
fn spawn_recv_thread(skt: &UdpSocket, schema: &PacketSchema) {
    let skt = skt.try_clone().expect("clone failed");
    let schema = schema.clone();

    thread::spawn(move || {
        let mut buf = [0u8; 1024];
        loop {
            let Ok((skt_buf_len, skt_src)) = skt.recv_from(&mut buf) else {
                return;
            };

            match &buf[..skt_buf_len] {
                [pkt_kind, dec_type, body @ ..]
                    if *pkt_kind == PacketKind::Data as u8
                        && *dec_type == DecodeType::Single as u8 =>
                {
                    if !schema.mask_fits(body) {
                        logt!("Data/Single with bad mask, len={skt_buf_len}");
                        continue;
                    }

                    // Payload is packed in schema order, so DevPacketId can
                    // only be read by walking every set field before it.
                    let mut reader = BitReader::new(&schema, body);
                    let mut pkt_fields = Vec::new();
                    let mut pkt_id = None;
                    for (name, _) in schema.fields.iter() {
                        if !reader.isset(name) {
                            continue;
                        }
                        let val = reader.read(name);
                        if matches!(name, FieldName::DevPacketId) {
                            pkt_id = Some(val as u8);
                        }
                        pkt_fields.push(format!("{name:?}={val}"));
                    }
                    logt!("pkt id={pkt_id:?} fields={}", pkt_fields.join(" "));

                    if let Some(id) = pkt_id {
                        skt.send_to(&bitp::rel::ack::encode(id), skt_src).ok();
                    }
                }
                other => logt!("kind={} len={}", other.first().unwrap_or(&0), skt_buf_len),
            }
        }
    });
}

/// Join = `[PacketKind::Data][DecodeType::Single][mask][payload]`, matching
/// what the server's Data arm strips with `buf.get(2..)`.
fn join_pkt(schema: &PacketSchema) -> Vec<u8> {
    let mut w = BitWriter::new(schema);
    w.write(&FieldName::DevPing, &1);

    let mut pkt = vec![PacketKind::Data as u8, DecodeType::Single as u8];
    pkt.extend(w.buf());
    pkt
}

/// Intent = `[PacketKind::Intent][mask][payload]` (no sid, no packet id).
/// Write order MUST match `IntentPkt::intent()` on the server:
/// seq, mv_x, mv_z, yaw, crouch, jump. All six fields must be set.
fn intent_pkt(schema: &PacketSchema, seq: u8, intent: PlayerIntentSim) -> Vec<u8> {
    let mut w = BitWriter::new(schema);
    w.write(&FieldName::IntentSeq, &(seq as u32));
    w.write(&FieldName::IntentMvX, &MV.encode(intent.held.mv_x));
    w.write(&FieldName::IntentMvZ, &MV.encode(intent.held.mv_z));
    w.write(&FieldName::IntentYaw, &YAW.encode(intent.held.yaw));
    w.write(&FieldName::IntentCrouch, &(intent.held.crouch as u32));
    w.write(&FieldName::IntentJump, &(intent.edges.jump as u32));

    let mut pkt = vec![PacketKind::Intent as u8];
    pkt.extend(w.buf());
    pkt
}
