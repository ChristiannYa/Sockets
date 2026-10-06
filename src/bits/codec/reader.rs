use crate::{
    bits::{PacketSchema, utils::mask},
    pkt::FieldName,
};

pub struct BitReader<'s, 'b> {
    packet_schema: &'s PacketSchema,
    buf: &'b [u8],
    buf_mask: &'b [u8],
    bits_acc: usize,
}

impl<'s, 'b> BitReader<'s, 'b> {
    pub fn new(pkt_schema: &'s PacketSchema, buf: &'b [u8]) -> Self {
        let (buf_mask, buf) = buf.split_at(pkt_schema.mask_len());
        BitReader {
            packet_schema: pkt_schema,
            buf,
            buf_mask,
            bits_acc: 0,
        }
    }

    pub fn read(&mut self, field_name: &FieldName) -> u32 {
        let field = self.packet_schema.info_of(field_name);
        let mut val: u32 = 0;
        let mut bits_read = 0;

        while bits_read < field.len {
            let ind = self.bits_acc / 8;
            let ofs = self.bits_acc % 8;
            let take = (8 - ofs).min(field.len - bits_read);

            let chunk = ((self.buf[ind] >> ofs) as u32) & mask(&take);
            val |= chunk << bits_read;

            bits_read += take;
            self.bits_acc += take;
        }

        val
    }

    pub fn isset(&self, field_name: &FieldName) -> bool {
        let field = self.packet_schema.info_of(field_name);
        let mask_ind = field.ind / 8;
        let mask_ofs = field.ind % 8;

        (self.buf_mask[mask_ind] >> mask_ofs) & 1 == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bits::{BitReader, BitWriter, schema::PacketSchema};
    use tap::{Pipe, Tap};

    fn t_reads_match(
        schema: &PacketSchema,
        field_values: &[(FieldName, u32)],
    ) -> Result<(), String> {
        let mut writer = BitWriter::new(schema);

        let mut bits_written_test = String::from("");
        for (field_name, field_val) in field_values.iter() {
            writer.write(field_name, field_val);

            let _ = schema
                .fields
                .pipe(|_| {
                    format!(
                        "{field_val:0field_len$b}",
                        field_len = schema.info_of(field_name).len
                    )
                })
                .tap(|field_bits| {
                    for bit in field_bits.chars() {
                        bits_written_test.insert(0, bit)
                    }
                });
        }
        log::debug!("bits_written (test): {bits_written_test}");

        let buf = writer.buf();
        let mut reader = BitReader::new(schema, &buf);

        let field_reads: Vec<(FieldName, u32)> = field_values
            .iter()
            .map(|(field_name, _)| {
                ((*field_name).clone(), reader.read(field_name))
            })
            .collect();

        let all_reads_match =
            field_values
                .iter()
                .enumerate()
                .all(|(ind, (_, field_val))| {
                    let (field_read_name, field_read_value) = &field_reads[ind];
                    let matches = *field_val == *field_read_value;
                    if !matches {
                        log::error!(
                            "{:?}: read ({field_read_value})",
                            field_read_name
                        );
                        log::error!(
                            "{:?}: orig ({field_val})",
                            field_read_name
                        );
                    } else {
                        log::debug!(
                            "{:?}: read ({field_read_value}) == orig ({})",
                            field_read_name,
                            field_val
                        );
                    }
                    matches
                });

        if all_reads_match {
            Ok(())
        } else {
            Err(String::from("one or more reads do not match"))
        }
    }

    #[test]
    fn read_base() -> Result<(), String> {
        let schema = PacketSchema::build(&[
            (FieldName::Health, 3),
            (FieldName::PlayerCount, 4),
            (FieldName::IsJumping, 1),
        ])
        .unwrap();

        let field_values = [
            (FieldName::Health, 5),
            (FieldName::PlayerCount, 9),
            (FieldName::IsJumping, 1),
        ];

        t_reads_match(&schema, &field_values)
    }

    #[test]
    fn read_overflow() -> Result<(), String> {
        let schema = PacketSchema::build(&[
            (FieldName::Health, 3),
            (FieldName::RocketsCount, 3),
            (FieldName::PlayerCount, 4),
            (FieldName::KeysCount, 3),
            (FieldName::Time24, 5),
            (FieldName::IsFriendly, 1),
        ])
        .unwrap();

        let field_values = [
            (FieldName::Health, 2),
            (FieldName::RocketsCount, 4),
            (FieldName::PlayerCount, 12),
            (FieldName::KeysCount, 1),
            (FieldName::Time24, 16),
            (FieldName::IsFriendly, 0),
        ];

        t_reads_match(&schema, &field_values)
    }

    #[test]
    fn read_super_overflow_v1() -> Result<(), String> {
        let schema = PacketSchema::build(&[
            (FieldName::Health, 3),
            (FieldName::RocketsCount, 3),
            (FieldName::PlayerCount, 4),
            (FieldName::KeysCount, 3),
            (FieldName::Time24, 5),
            (FieldName::IsFriendly, 1),
            (FieldName::Exp, 24),
        ])
        .unwrap();

        let field_values = [
            (FieldName::Health, 2),
            (FieldName::RocketsCount, 4),
            (FieldName::PlayerCount, 12),
            (FieldName::KeysCount, 1),
            (FieldName::Time24, 16),
            (FieldName::IsFriendly, 1),
            (FieldName::Exp, 9842952),
        ];

        t_reads_match(&schema, &field_values)
    }

    #[test]
    fn read_super_overflow_short_tail() -> Result<(), String> {
        // Year (11 bits) starts at bit 6: 2 bits in byte 0, 8 in byte 1, 1 in byte 2
        let schema = PacketSchema::build(&[
            (FieldName::Health, 3),
            (FieldName::RocketsCount, 3),
            (FieldName::Year, 11),
            (FieldName::Age, 7),
            (FieldName::IsFriendly, 1),
        ])
        .unwrap();

        let field_values = [
            (FieldName::Health, 5),
            (FieldName::RocketsCount, 2),
            (FieldName::Year, 1437),
            (FieldName::Age, 100),
            (FieldName::IsFriendly, 1),
        ];

        t_reads_match(&schema, &field_values)
    }

    #[test]
    fn read_misc_super_overflow_v2() -> Result<(), String> {
        let schema = PacketSchema::build(&[
            (FieldName::Health, 3),
            (FieldName::RocketsCount, 3),
            (FieldName::PlayerCount, 4),
            (FieldName::KeysCount, 3),
            (FieldName::Time24, 5),
            (FieldName::IsFriendly, 1),
            (FieldName::Exp, 24),
            (FieldName::Age, 7),
            (FieldName::Year, 13),
            (FieldName::IsPremium, 1),
            (FieldName::Boss, 31),
            (FieldName::IsLucky, 1),
        ])
        .unwrap();

        let field_values = [
            (FieldName::Health, 2),
            (FieldName::RocketsCount, 4),
            (FieldName::PlayerCount, 12),
            (FieldName::KeysCount, 1),
            (FieldName::Time24, 16),
            (FieldName::IsFriendly, 1),
            (FieldName::Exp, 9842952),
            (FieldName::Age, 24),
            (FieldName::Year, 7076),
            (FieldName::IsPremium, 0),
            (FieldName::Boss, 41),
            (FieldName::IsLucky, 1),
        ];

        t_reads_match(&schema, &field_values)
    }

    #[test]
    fn read_misc_spawn_packet_layout() -> Result<(), String> {
        use crate::pkt::FIELD_LENGTHS;
        let schema = PacketSchema::build(FIELD_LENGTHS).unwrap();

        // Same fields, same order as the server's spawn packet
        let field_values = [
            (FieldName::DevSessionId, 0),
            (FieldName::DevPacketId, 0),
            (FieldName::LocationX, 1008),
            (FieldName::LocationZ, 1089),
            (FieldName::ColorH, 45),
            (FieldName::ColorS, 5),
            (FieldName::ColorV, 6),
        ];

        t_reads_match(&schema, &field_values)
    }

    #[test]
    fn read_schema_partial() -> Result<(), String> {
        crate::util::test::init_test_logger();

        let schema = PacketSchema::build(&[
            (FieldName::Health, 3),
            (FieldName::RocketsCount, 3),
            (FieldName::PlayerCount, 4),
            (FieldName::KeysCount, 3),
            (FieldName::Time24, 5),
            (FieldName::IsFriendly, 1),
            (FieldName::Exp, 24),
        ])
        .unwrap();

        let field_values = [
            (FieldName::Health, 2),
            (FieldName::Time24, 16),
            (FieldName::IsFriendly, 1),
            (FieldName::Exp, 655332),
        ];

        t_reads_match(&schema, &field_values)
    }

    #[test]
    fn read_misc_0() -> Result<(), String> {
        let schema =
            PacketSchema::build(&[(FieldName::IsFriendly, 1)]).unwrap();
        let field_values = [(FieldName::IsFriendly, 0)];

        t_reads_match(&schema, &field_values)
    }

    #[test]
    fn read_misc_u8() -> Result<(), String> {
        let schema = PacketSchema::build(&[(FieldName::Boss, 32)]).unwrap();
        let field_values = [(FieldName::Boss, 20)];

        t_reads_match(&schema, &field_values)
    }
}
