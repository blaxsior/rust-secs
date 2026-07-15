use alloc::vec::Vec;
use secs_ii::{convert::secs2::serialize::Encode, item::Secs2Variant};

use crate::transport::secs1::{Secs1Message, Secs1MessageHeader};
use crate::transport::{
    error::SecsMessageConvertError,
    secs1::block::{Secs1Block, Secs1BlockHeader},
};

pub fn decode(mut blocks: Vec<Secs1Block>) -> Result<Secs1Message, SecsMessageConvertError> {
    // block이 비어 있는 경우 -> 에러
    if blocks.is_empty() {
        return Err(SecsMessageConvertError::EmptyBlocks);
    }
    // 혹시 모르니 block_no 순서대로 정렬 (데이터 받는 순서 상 문제 없어야 함)
    blocks.sort_by(|a, b| a.header.block_no.cmp(&b.header.block_no));

    // 블록 번호가 순차적인지 검사
    let mut expected = blocks[0].header.block_no;
    for block in &blocks {
        if block.header.block_no != expected {
            return Err(SecsMessageConvertError::SequenceGap(block.header.block_no));
        }
        expected += 1;
    }

    // 마지막 블록이 E-bit가 설정되어 있지 않은 경우 체크
    let header = &blocks
        .last()
        .ok_or(SecsMessageConvertError::EmptyBlocks)?
        .header;

    if !header.is_end() {
        return Err(SecsMessageConvertError::MissingEbit);
    }

    let header = Secs1MessageHeader::from(header);

    let raw_bytes: Vec<u8> = blocks.into_iter().flat_map(|it| it.data).collect();
    let body = Secs2Variant::try_from(raw_bytes.as_slice())
        .map_err(|e| SecsMessageConvertError::DecodeFailed(e))?;

    let msg = Secs1Message::new(header, body);

    Ok(msg)
}

pub fn encode(msg: Secs1Message) -> Result<Vec<Secs1Block>, SecsMessageConvertError> {
    let body = msg.body;
    let msg_header = msg.header;

    let mut raw_data = Vec::new();
    if let Err(err) = body.encode(&mut raw_data) {
        return Err(SecsMessageConvertError::EncodeFailed(err));
    }

    let blocks = raw_data
        .chunks(244)
        .enumerate()
        .map(|(i, chunk)| {
            let is_last = (i + 1) * 244 >= raw_data.len();

            // 헤더 구성
            let header = Secs1BlockHeader {
                device_id: msg_header.device_id,
                rbit: msg_header.rbit,
                stream: msg_header.stream,
                function: msg_header.function,
                wbit: msg_header.wbit,
                ebit: is_last,
                block_no: (i + 1) as u16,
                system_byte: msg_header.system_byte,
                // 기타 헤더 필드 설정...
            };

            Secs1Block {
                header,
                data: chunk.to_vec(), // data에 맞게 vec로 복사
            }
        })
        .collect::<Vec<Secs1Block>>();

    Ok(blocks)
}

#[cfg(test)]
mod tests {
    use crate::transport::secs1::{Secs1Message, Secs1MessageHeader};
    use crate::transport::{
        DeviceId, Rbit, SystemByte, Wbit,
        secs1::{
            block::{Secs1Block, Secs1BlockHeader},
            convert::{decode, encode},
        },
    };
    use secs_ii::{FunctionId, StreamId, convert::secs2::serialize::Encode, item::Secs2Variant};

    /// primary + need recv 데이터를 요청받은 경우
    #[test]
    fn test_encode_recv_primary_need_reply() {
        let stream = StreamId(1);
        let function = FunctionId(3);
        let device_id = DeviceId(1016);
        let system_byte = SystemByte(3030);
        let rbit = Rbit(false);
        let wbit = Wbit(true);

        let header = Secs1MessageHeader {
            stream,
            function,
            device_id,
            system_byte,
            rbit,
            wbit
        };

        let body = Secs2Variant::list(vec![
            Secs2Variant::uint4(1001),
            Secs2Variant::uint4(1002),
            Secs2Variant::uint4(1003),
            Secs2Variant::uint4(1004),
            Secs2Variant::uint4(1005),
            Secs2Variant::uint4(1006),
            Secs2Variant::uint4(1007),
            Secs2Variant::uint4(1008),
            Secs2Variant::uint4(1009),
            Secs2Variant::uint4(1010),
        ]);

        let msg = Secs1Message::new(header, body);
        // host -> eqp 가정

        let expected_data = vec![
            0x01, 0x0A, 0xB1, 0x04, 0x00, 0x00, 0x03, 0xE9, 0xB1, 0x04, 0x00, 0x00, 0x03, 0xEA,
            0xB1, 0x04, 0x00, 0x00, 0x03, 0xEB, 0xB1, 0x04, 0x00, 0x00, 0x03, 0xEC, 0xB1, 0x04,
            0x00, 0x00, 0x03, 0xED, 0xB1, 0x04, 0x00, 0x00, 0x03, 0xEE, 0xB1, 0x04, 0x00, 0x00,
            0x03, 0xEF, 0xB1, 0x04, 0x00, 0x00, 0x03, 0xF0, 0xB1, 0x04, 0x00, 0x00, 0x03, 0xF1,
            0xB1, 0x04, 0x00, 0x00, 0x03, 0xF2,
        ];

        let blocks = encode(msg).expect("message encode failed");

        assert_eq!(blocks.len(), 1);
        let block = blocks.get(0).unwrap();
        let header = &block.header;

        assert_eq!(header.device_id, device_id);
        assert_eq!(header.system_byte, system_byte);
        assert_eq!(header.block_no, 1);
        assert_eq!(header.stream, StreamId(1));
        assert_eq!(header.function, FunctionId(3));
        assert_eq!(header.rbit, Rbit(false));
        assert_eq!(header.ebit, true);
        assert_eq!(block.data, expected_data);
    }

    /// multi block이 정상적으로 분리되는지 테스트
    #[test]
    fn test_encode_multi_block_msg() {
        let device_id = DeviceId(1016);
        let system_byte = SystemByte(3030);
        let rbit = Rbit(false);

        let body = Secs2Variant::list(vec![
            Secs2Variant::uint8(1001),
            Secs2Variant::uint8(1002),
            Secs2Variant::uint8(1003),
            Secs2Variant::uint8(1004),
            Secs2Variant::uint8(1005),
            Secs2Variant::uint8(1006),
            Secs2Variant::uint8(1007),
            Secs2Variant::uint8(1008),
            Secs2Variant::uint8(1009),
            Secs2Variant::uint8(1010),
            Secs2Variant::uint8(2001),
            Secs2Variant::uint8(2002),
            Secs2Variant::uint8(2003),
            Secs2Variant::uint8(2004),
            Secs2Variant::uint8(2005),
            Secs2Variant::uint8(2006),
            Secs2Variant::uint8(2007),
            Secs2Variant::uint8(2008),
            Secs2Variant::uint8(2009),
            Secs2Variant::uint8(2010),
            Secs2Variant::uint8(3001),
            Secs2Variant::uint8(3002),
            Secs2Variant::uint8(3003),
            Secs2Variant::uint8(3004),
            Secs2Variant::uint8(3005),
            Secs2Variant::uint8(3006),
            Secs2Variant::uint8(3007),
            Secs2Variant::uint8(3008),
            Secs2Variant::uint8(3009),
            Secs2Variant::uint8(3010),
        ]);

        let expected_data1 = [
            0x01, 0x1E, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xE9, 0xA1, 0x08,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xEA, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x03, 0xEB, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xEC,
            0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xED, 0xA1, 0x08, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x03, 0xEE, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x03, 0xEF, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xF0, 0xA1, 0x08,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xF1, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x03, 0xF2, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0xD1,
            0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0xD2, 0xA1, 0x08, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x07, 0xD3, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x07, 0xD4, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0xD5, 0xA1, 0x08,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0xD6, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x07, 0xD7, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0xD8,
            0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0xD9, 0xA1, 0x08, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x07, 0xDA, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x0B, 0xB9, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0xBA, 0xA1, 0x08,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0xBB, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x0B, 0xBC, 0xA1, 0x08,
        ];

        let expected_data2 = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0xBD, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x0B, 0xBE, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0xBF,
            0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0xC0, 0xA1, 0x08, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x0B, 0xC1, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x0B, 0xC2,
        ];

        let header = Secs1MessageHeader {
            device_id,
            rbit,
            wbit: Wbit(true),
            stream: StreamId(1),
            function: FunctionId(4),
            system_byte,
        };
        let msg = Secs1Message::new(header, body);

        let blocks = encode(msg).expect("message encode failed");

        assert_eq!(blocks.len(), 2);
        let block = &blocks[0];
        assert_eq!(block.header.device_id, device_id);
        assert_eq!(block.header.system_byte, system_byte);
        assert_eq!(block.header.block_no, 1);
        assert_eq!(block.header.stream, StreamId(1));
        assert_eq!(block.header.function, FunctionId(4));
        assert_eq!(block.header.rbit, Rbit(false));
        assert_eq!(block.header.ebit, false);
        assert_eq!(block.data, expected_data1);

        let block = &blocks[1];
        assert_eq!(block.header.device_id, device_id);
        assert_eq!(block.header.system_byte, system_byte);
        assert_eq!(block.header.block_no, 2);
        assert_eq!(block.header.stream, StreamId(1));
        assert_eq!(block.header.function, FunctionId(4));
        assert_eq!(block.header.rbit, Rbit(false));
        assert_eq!(block.header.ebit, true);
        assert_eq!(block.data, expected_data2);
    }

    #[test]
    fn test_decode_multi_block_msg() {
        let data1 = [
            0x01, 0x1E, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xE9, 0xA1, 0x08,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xEA, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x03, 0xEB, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xEC,
            0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xED, 0xA1, 0x08, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x03, 0xEE, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x03, 0xEF, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xF0, 0xA1, 0x08,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xF1, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x03, 0xF2, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0xD1,
            0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0xD2, 0xA1, 0x08, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x07, 0xD3, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x07, 0xD4, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0xD5, 0xA1, 0x08,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0xD6, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x07, 0xD7, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0xD8,
            0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0xD9, 0xA1, 0x08, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x07, 0xDA, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x0B, 0xB9, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0xBA, 0xA1, 0x08,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0xBB, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x0B, 0xBC, 0xA1, 0x08,
        ];

        let data2 = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0xBD, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x0B, 0xBE, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0xBF,
            0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0xC0, 0xA1, 0x08, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x0B, 0xC1, 0xA1, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x0B, 0xC2,
        ];

        let device_id = DeviceId(1016);
        let system_byte = SystemByte(3030);
        let rbit = Rbit(false);
        let expected_body = Secs2Variant::list(vec![
            Secs2Variant::uint8(1001),
            Secs2Variant::uint8(1002),
            Secs2Variant::uint8(1003),
            Secs2Variant::uint8(1004),
            Secs2Variant::uint8(1005),
            Secs2Variant::uint8(1006),
            Secs2Variant::uint8(1007),
            Secs2Variant::uint8(1008),
            Secs2Variant::uint8(1009),
            Secs2Variant::uint8(1010),
            Secs2Variant::uint8(2001),
            Secs2Variant::uint8(2002),
            Secs2Variant::uint8(2003),
            Secs2Variant::uint8(2004),
            Secs2Variant::uint8(2005),
            Secs2Variant::uint8(2006),
            Secs2Variant::uint8(2007),
            Secs2Variant::uint8(2008),
            Secs2Variant::uint8(2009),
            Secs2Variant::uint8(2010),
            Secs2Variant::uint8(3001),
            Secs2Variant::uint8(3002),
            Secs2Variant::uint8(3003),
            Secs2Variant::uint8(3004),
            Secs2Variant::uint8(3005),
            Secs2Variant::uint8(3006),
            Secs2Variant::uint8(3007),
            Secs2Variant::uint8(3008),
            Secs2Variant::uint8(3009),
            Secs2Variant::uint8(3010),
        ]);

        let expected = Secs1Message::new(
            Secs1MessageHeader {
                device_id,
                rbit,
                wbit: Wbit(false),
                stream: StreamId(1),
                function: FunctionId(4),
                system_byte,
            },
            expected_body,
        );

        // expected_data1 / expected_data2는 encode 테스트와 동일

        let blocks = vec![
            Secs1Block {
                header: Secs1BlockHeader {
                    device_id,
                    system_byte,
                    block_no: 1,
                    wbit: Wbit(false),
                    stream: StreamId(1),
                    function: FunctionId(4),
                    rbit,
                    ebit: false,
                },
                data: data1.to_vec(),
            },
            Secs1Block {
                header: Secs1BlockHeader {
                    device_id,
                    system_byte,
                    block_no: 2,
                    wbit: Wbit(false),
                    stream: StreamId(1),
                    function: FunctionId(4),
                    rbit,
                    ebit: true,
                },
                data: data2.to_vec(),
            },
        ];

        let actual = decode(blocks).expect("message decode failed");

        assert_eq!(expected.header.device_id, actual.header.device_id);
        assert_eq!(expected.header.rbit, actual.header.rbit);
        assert_eq!(expected.header.system_byte, actual.header.system_byte);

        let mut expected_bytes = Vec::new();
        expected.body.encode(&mut expected_bytes).unwrap();
        let mut actual_bytes = Vec::new();
        actual.body.encode(&mut actual_bytes).unwrap();
        assert_eq!(expected_bytes, actual_bytes);
    }
}
