use alloc::vec::Vec;
use secs_ii::{SecsMessage, convert::secs2::serialize::Encode, item::Secs2Variant};

use crate::transport::{
    ConnectionMode, TransactionId, error::SecsMessageConvertError, secs1::{
        block::{Secs1Block, Secs1BlockHeader},
        config::DeviceId,
    }
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

    let stream = header.stream;
    let function = header.function;
    let need_reply = header.need_reply();

    let raw_bytes: Vec<u8> = blocks.iter().flat_map(|it| it.to_bytes()).collect();
    let secs_value = Secs2Variant::try_from(raw_bytes.as_slice())
        .map_err(|e| SecsMessageConvertError::DecodeFailed(e))?;

    Ok(SecsMessage::new(stream, function, need_reply, secs_value))
}

pub fn encode(
    device_id: DeviceId,
    transaction_id: TransactionId,
    connection_mode: ConnectionMode,
    msg: SecsMessage,
) -> Result<Vec<Secs1Block>, SecsMessageConvertError> {
    let stream = msg.stream;
    let function = msg.function;
    let need_reply = msg.need_reply;

    let mut raw_data = Vec::new();
    if let Err(err) = msg.body.encode(&mut raw_data) {
        return Err(SecsMessageConvertError::EncodeFailed(err));
    }

    let blocks = raw_data
        .chunks(244)
        .enumerate()
        .map(|(i, chunk)| {
            let is_last = (i + 1) * 244 >= raw_data.len();

            // 헤더 구성
            let header = Secs1BlockHeader {
                device_id: device_id,
                rbit: connection_mode == ConnectionMode::Passive,
                stream: stream,
                function: function,
                wbit: need_reply,
                ebit: is_last,
                block_no: (i + 1) as u16,
                system_bytes: transaction_id,
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
