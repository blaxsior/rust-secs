use alloc::vec::Vec;
use secs_ii::{Secs2Message, convert::secs2::serialize::Encode, item::Secs2Variant};

use crate::{
    core::SecsMessage,
    transport::{
        error::SecsMessageConvertError,
        secs1::block::{Secs1Block, Secs1BlockHeader},
    },
};

pub fn decode(mut blocks: Vec<Secs1Block>) -> Result<SecsMessage, SecsMessageConvertError> {
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

    let device_id = header.device_id;
    let system_byte = header.system_byte;
    let rbit = header.rbit;

    let stream = header.stream;
    let function = header.function;
    let need_reply = header.need_reply();

    let raw_bytes: Vec<u8> = blocks.iter().flat_map(|it| it.to_bytes()).collect();
    let secs_value = Secs2Variant::try_from(raw_bytes.as_slice())
        .map_err(|e| SecsMessageConvertError::DecodeFailed(e))?;

    let payload = Secs2Message::new(stream, function, need_reply, secs_value);
    let msg = SecsMessage::new(device_id, system_byte, rbit, payload);

    Ok(msg)
}

pub fn encode(msg: &SecsMessage) -> Result<Vec<Secs1Block>, SecsMessageConvertError> {
    let payload = &msg.payload;
    let stream = payload.stream;
    let function = payload.function;
    let need_reply = payload.need_reply;

    let mut raw_data = Vec::new();
    if let Err(err) = payload.body.encode(&mut raw_data) {
        return Err(SecsMessageConvertError::EncodeFailed(err));
    }

    let blocks = raw_data
        .chunks(244)
        .enumerate()
        .map(|(i, chunk)| {
            let is_last = (i + 1) * 244 >= raw_data.len();

            // 헤더 구성
            let header = Secs1BlockHeader {
                device_id: msg.device_id,
                rbit: msg.rbit,
                stream: stream,
                function: function,
                wbit: need_reply,
                ebit: is_last,
                block_no: (i + 1) as u16,
                system_byte: msg.system_byte,
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
    use secs_ii::{FunctionId, Secs2Message, StreamId, item::Secs2Variant};

    use crate::{
        core::SecsMessage,
        transport::{DeviceId, Rbit, SystemByte, secs1::convert::encode},
    };

    /// primary + need recv 데이터를 요청받은 경우
    #[test]
    fn test_recv_primary_need_reply() {
        let device_id = DeviceId(1016);
        let system_byte = SystemByte(3030);
        let rbit = Rbit(false);

        let payload = Secs2Message::new(
            StreamId(1),
            FunctionId(3),
            true,
            Secs2Variant::list(vec![
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
            ]),
        );
        let msg = SecsMessage::new(device_id, system_byte, rbit, payload);
        // host -> eqp 가정

        let expected_data = vec![
            0x01, 0x0A, 0xB1, 0x04, 0x00, 0x00, 0x03, 0xE9, 0xB1, 0x04, 0x00, 0x00, 0x03, 0xEA,
            0xB1, 0x04, 0x00, 0x00, 0x03, 0xEB, 0xB1, 0x04, 0x00, 0x00, 0x03, 0xEC, 0xB1, 0x04,
            0x00, 0x00, 0x03, 0xED, 0xB1, 0x04, 0x00, 0x00, 0x03, 0xEE, 0xB1, 0x04, 0x00, 0x00,
            0x03, 0xEF, 0xB1, 0x04, 0x00, 0x00, 0x03, 0xF0, 0xB1, 0x04, 0x00, 0x00, 0x03, 0xF1,
            0xB1, 0x04, 0x00, 0x00, 0x03, 0xF2,
        ];

        let blocks = encode(&msg).expect("message encode failed");

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

    // #[test]
    // fn test_send() {}
}
