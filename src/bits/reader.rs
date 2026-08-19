use tap::Pipe;

use crate::{
    bits::{
        FieldName, PacketSchema,
        shared::{BufferProgress, field_meta_or_panic},
    },
    util::mask,
};

pub struct BitReader<'a> {
    packet_schema: &'a PacketSchema<'a>,
    buf: &'a [u8],
    buf_mask: &'a [u8],
    bits_acc: usize,
}

impl<'a> BitReader<'a> {
    pub fn new(packet_schema: &'a PacketSchema, buf: &'a [u8]) -> Self {
        let (buf_mask, buf) = Self::split_buf(packet_schema.fields, buf);
        BitReader { packet_schema, buf, buf_mask, bits_acc: 0 }
    }

    fn split_buf(
        field_schemas: &'a [(FieldName, usize)],
        buf: &'a [u8],
    ) -> (&'a [u8], &'a [u8]) {
        field_schemas
            .len()
            .div_ceil(8)
            .pipe(|len| buf.split_at(len))
    }

    pub fn read(&mut self, field_name: &FieldName) -> u32 {
        let (_, field_len) = field_meta_or_panic(self.packet_schema.fields, field_name);

        BufferProgress::calc(self.bits_acc, &field_len).pipe(|prog| {
            if prog.ovf_len > 0 {
                self.read_ovf(&prog)
            } else {
                self.bits_acc += field_len;
                ((self.buf[prog.ind] >> prog.ofs) as u32) & mask(&field_len)
            }
        })
    }

    fn read_ovf(&mut self, prog: &BufferProgress) -> u32 {
        let rem = ((self.buf[prog.ind] >> prog.ofs) as u32) & mask(&prog.rem_len);
        let ovf = if prog.ovf_len > 8 { 8 } else { prog.ovf_len }
            .pipe(|len| (self.buf[prog.ind + 1] as u32) & mask(&len));

        let merge = (ovf << prog.rem_len) | rem;

        let field_len = prog.rem_len + prog.ovf_len;

        if prog.ovf_len > 8 {
            let read_len = prog.rem_len + 8;

            self.bits_acc += read_len;

            let prog =
                (field_len - read_len).pipe(|len| BufferProgress::calc(self.bits_acc, &len));

            (self.read_ovf(&prog) << read_len) | merge
        } else {
            self.bits_acc += field_len;
            merge
        }
    }

    pub fn isset(&self, field_name: &FieldName) -> bool {
        let (field_ind, _) = field_meta_or_panic(self.packet_schema.fields, field_name);
        let mask_ind = field_ind / 8;
        let mask_ofs = field_ind % 8;

        (self.buf_mask[mask_ind] >> mask_ofs) & 1 == 1
    }
}

#[cfg(test)]
mod tests {
    use tap::{Pipe, Tap};

    use crate::bits::{
        BitReader, BitWriter, FieldName, PacketSchema, shared::field_meta_or_panic,
    };

    fn t_reads_match(
        field_schemas: &[(FieldName, usize)],
        field_values: &[(FieldName, u32)],
    ) -> Result<(), String> {
        let schema = PacketSchema::build(field_schemas)?;
        let mut writer = BitWriter::new(&schema);

        let mut bits_written_test = String::from("");
        for (field_name, field_val) in field_values.iter() {
            writer.write(field_name.clone(), *field_val);

            let _ = field_meta_or_panic(field_schemas, field_name)
                .pipe(|(_, field_len)| format!("{field_val:0field_len$b}"))
                .tap(|field_bits| {
                    for bit in field_bits.chars() {
                        bits_written_test.insert(0, bit)
                    }
                });
        }
        log::debug!("bits_written (test): {bits_written_test}");

        let buf = writer.buf();
        let mut reader = BitReader::new(&schema, &buf);

        let field_reads: Vec<(FieldName, u32)> = field_values
            .iter()
            .map(|(field_name, _)| ((*field_name).clone(), reader.read(field_name)))
            .collect();

        let all_reads_match = field_values
            .iter()
            .enumerate()
            .all(|(ind, (_, field_val))| {
                let (field_read_name, field_read_value) = &field_reads[ind];
                let matches = *field_val == *field_read_value;
                if !matches {
                    log::error!("{:?}: read ({field_read_value})", field_read_name);
                    log::error!("{:?}: orig ({field_val})", field_read_name);
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
        let field_schemas =
            [(FieldName::Health, 3), (FieldName::PlayerCount, 4), (FieldName::IsJumping, 1)];

        let field_values =
            [(FieldName::Health, 5), (FieldName::PlayerCount, 9), (FieldName::IsJumping, 1)];

        t_reads_match(&field_schemas, &field_values)
    }

    #[test]
    fn read_overflow() -> Result<(), String> {
        let field_schemas = [
            (FieldName::Health, 3),
            (FieldName::RocketsCount, 3),
            (FieldName::PlayerCount, 4),
            (FieldName::KeysCount, 3),
            (FieldName::Time24, 5),
            (FieldName::IsFriendly, 1),
        ];

        let field_values = [
            (FieldName::Health, 2),
            (FieldName::RocketsCount, 4),
            (FieldName::PlayerCount, 12),
            (FieldName::KeysCount, 1),
            (FieldName::Time24, 16),
            (FieldName::IsFriendly, 0),
        ];

        t_reads_match(&field_schemas, &field_values)
    }

    #[test]
    fn read_super_overflow_v1() -> Result<(), String> {
        let field_schemas = [
            (FieldName::Health, 3),
            (FieldName::RocketsCount, 3),
            (FieldName::PlayerCount, 4),
            (FieldName::KeysCount, 3),
            (FieldName::Time24, 5),
            (FieldName::IsFriendly, 1),
            (FieldName::Exp, 24),
        ];

        let field_values = [
            (FieldName::Health, 2),
            (FieldName::RocketsCount, 4),
            (FieldName::PlayerCount, 12),
            (FieldName::KeysCount, 1),
            (FieldName::Time24, 16),
            (FieldName::IsFriendly, 1),
            (FieldName::Exp, 9842952),
        ];

        t_reads_match(&field_schemas, &field_values)
    }

    #[test]
    fn read_misc_super_overflow_v2() -> Result<(), String> {
        let field_schemas = [
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
        ];

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

        t_reads_match(&field_schemas, &field_values)
    }

    #[test]
    fn read_schema_partial() -> Result<(), String> {
        crate::util::test::init_test_logger();

        let field_schemas = [
            (FieldName::Health, 3),
            (FieldName::RocketsCount, 3),
            (FieldName::PlayerCount, 4),
            (FieldName::KeysCount, 3),
            (FieldName::Time24, 5),
            (FieldName::IsFriendly, 1),
            (FieldName::Exp, 24),
        ];

        let field_values = [
            (FieldName::Health, 2),
            (FieldName::Time24, 16),
            (FieldName::IsFriendly, 1),
            (FieldName::Exp, 655332),
        ];

        t_reads_match(&field_schemas, &field_values)
    }

    #[test]
    fn read_misc_0() -> Result<(), String> {
        let field_schemas = [(FieldName::IsFriendly, 1)];
        let field_values = [(FieldName::IsFriendly, 0)];

        t_reads_match(&field_schemas, &field_values)
    }

    #[test]
    fn read_misc_u8() -> Result<(), String> {
        let field_schemas = [(FieldName::Boss, 32)];
        let field_values = [(FieldName::Boss, 20)];

        t_reads_match(&field_schemas, &field_values)
    }
}
