use crate::transport::secs1::Secs1Message;
use crate::transport::secs1::protocol::message::transaction::{
    Secs1MessageTransaction, Secs1TransactionEffect,
};
use crate::transport::secs1::protocol::message::transaction_manager::Secs1TransactionManager;
use crate::transport::{DeviceId, TimeoutTicket, TransactionOwner, TransferContext};
use crate::{
    transport::{
        SecsTimeoutUnit, TransactionKey,
        error::SecsTransportError,
        secs1::{
            block::{Secs1Block, Secs1BlockHeader},
            config::Secs1TransportConfig,
        },
    },
    util::time::TimeoutManager,
};

use alloc::collections::BTreeMap;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use sansio::Protocol;

use secs_ii::FunctionId;
use secs_ii::StreamId;

pub mod transaction;
pub mod transaction_manager;

/// secs-i message protocol 수행 중 외부에서 주입하는 이벤트(블록 정상 송신 등)
pub enum Secs1MessageSignal {
    /// 대상 블록 전송에 성공함
    BlockSendSuccess { header: Secs1BlockHeader },
    /// 대상 블록 전송을 실패
    BlockSendFailed { header: Secs1BlockHeader },
}

/// secs-i message protocol 수행 중 발생하는 이벤트
pub enum Secs1MessageEvent {
    /// 메시지를 성공적으로 송신
    SendComplete {
        transaction_key: TransactionKey,
    },
    /// 메시지를 성공적으로 수신
    RecvComplete {
        transaction_key: TransactionKey,
    },

    TransactionEnd {
        transaction_key: TransactionKey,
    },

    /// 메시지 송수신 중 타임아웃 발생
    MessageTimeout {
        transaction_key: TransactionKey,
        timeout_unit: SecsTimeoutUnit,
    },
    /// 비정상적인 상태 전이 등 에러가 발생한 경우
    ErrorOccured(SecsTransportError),
}

pub struct Secs1MessageMachine {
    /** 상태 임시 저장 */
    /// block 송신할 데이터를 임시로 보관하는 큐. poll_write
    outgoing_blocks: VecDeque<Secs1Block>,
    /// secs-ii msg 수신한 블록들을 임시로 저장하는 큐. poll_read
    outgoing_msgs: VecDeque<Secs1Message>,
    /// 외부로 타임아웃 시작 알리는 큐. poll_timeout
    outgoing_timeouts: VecDeque<TimeoutTicket>,
    /// 외부 발송할 이벤트 목록을 저장해두는 큐. poll_event
    outgoing_events: VecDeque<Secs1MessageEvent>,
    /// timeout 처리 객체
    timeout_manager: TimeoutManager,
    /// 통신 중인 디바이스의 ID
    device_id: DeviceId,

    /// reply 메시지에 대한 transaction ID - key mapping
    reply_map: BTreeMap<ReplyMsgKey, TransactionKey>,

    /// 트랜잭션 관리 구조체
    transaction_manager: Secs1TransactionManager,
}

/// 상대방 요청에 응답하기 위한 트랜잭션 식별 키
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct ReplyMsgKey(StreamId, FunctionId);

impl Secs1MessageMachine {
    pub fn new(config: &Secs1TransportConfig) -> Self {
        log::debug!("[message] create machine");
        Self {
            outgoing_blocks: VecDeque::new(),
            outgoing_msgs: VecDeque::new(),
            outgoing_timeouts: VecDeque::new(),
            outgoing_events: VecDeque::new(),
            timeout_manager: TimeoutManager::new(),
            device_id: config.device_id,
            reply_map: BTreeMap::new(),
            transaction_manager: Secs1TransactionManager::new(),
        }
    }

    /// 외부 시스템에 전송할 메시지를 담는다.
    fn emit_event(&mut self, output: Secs1MessageEvent) {
        self.outgoing_events.push_back(output);
    }

    /// timeout을 시작한다.
    fn start_timeout(&mut self, timeout: SecsTimeoutUnit) {
        log::debug!("[message] start timeout {:?}", timeout);
        let ticket = self.timeout_manager.issue(timeout);
        self.outgoing_timeouts.push_back(ticket);
    }

    /// timeout을 취소한다.
    fn cancel_timeout(&mut self, timeout: SecsTimeoutUnit) {
        log::debug!("[message] cancel timeout {:?}", timeout);
        self.timeout_manager.cancel(timeout);
    }

    /// block을 쓴다
    fn write_block(&mut self, block: Secs1Block) {
        log::debug!("[message] queue block {:?}", block.header);
        self.outgoing_blocks.push_back(block);
    }

    fn take_reply_transaction_key(
        &mut self,
        stream: StreamId,
        function: FunctionId,
    ) -> Option<TransactionKey> {
        self.reply_map.remove(&ReplyMsgKey(stream, function))
    }

    /// header로부터 recv transaction_key를 획득
    fn get_transaction_key(
        &self,
        context: TransferContext,
        header: &Secs1BlockHeader,
    ) -> TransactionKey {
        let is_primary = header.function.is_primary();
        let system_byte = header.system_byte;

        TransactionKey::from(context, is_primary, system_byte)
    }

    /// transaction이 만든 출력물을 한 번에 꺼낸다.
    fn take_transaction_outputs(
        transaction: &mut Secs1MessageTransaction,
    ) -> (
        Vec<Secs1Message>,
        Vec<Secs1Block>,
        Vec<Secs1TransactionEffect>,
    ) {
        let mut reads = Vec::new();
        while let Some(msg) = transaction.poll_read() {
            reads.push(msg);
        }

        let mut writes = Vec::new();
        while let Some(block) = transaction.poll_write() {
            writes.push(block);
        }

        let mut effects = Vec::new();
        while let Some(effect) = transaction.poll_event() {
            effects.push(effect);
        }
        (reads, writes, effects)
    }

    /// transaction 출력물을 엔진 큐에 반영한다.
    fn handle_transaction_outputs(
        &mut self,
        outputs: (
            Vec<Secs1Message>,
            Vec<Secs1Block>,
            Vec<Secs1TransactionEffect>,
        ),
        transaction_key: &TransactionKey,
    ) {
        let (reads, writes, effects) = outputs;

        for msg in reads {
            log::debug!("[message] queue message {:?}", msg.header.system_byte);
            self.outgoing_msgs.push_back(msg);
        }

        for block in writes {
            self.write_block(block);
        }

        self.handle_effects(effects, transaction_key);
    }

    fn process_send(&mut self, msg: Secs1Message) {
        log::debug!(
            "[message] process send: stream={:?} function={:?} need_reply={} system_byte={:?}",
            msg.header.stream,
            msg.header.function,
            msg.header.wbit.need_reply(),
            msg.header.system_byte
        );
        // 1.  SxFy, y&1 == 0인 경우 -> 기존 프로세스 참조
        let stream = msg.header.stream;
        let function = msg.header.function;
        let system_byte = msg.header.system_byte;

        if function.is_primary() {
            // 요청 -> 트랜잭션을 새롭게 생성
            let transaction_key = TransactionKey::new(TransactionOwner::Local, system_byte);

            let transaction = match self.transaction_manager.create_send(&transaction_key, msg) {
                Some(t) => t,
                None => {
                    log::warn!(
                        "[message] failed to create send transaction: {:?}",
                        transaction_key
                    );
                    self.emit_event(Secs1MessageEvent::ErrorOccured(
                        SecsTransportError::NoSuchTransaction(transaction_key),
                    ));
                    return;
                }
            };
            let outputs = Self::take_transaction_outputs(transaction);
            self.handle_transaction_outputs(outputs, &transaction_key);
        } else {
            // 상대 request 받고 대응되는 메시지 보내는 상황
            let transaction_key = match self.take_reply_transaction_key(stream, function) {
                Some(key) => key,
                None => {
                    log::warn!(
                        "[message] no reply transaction for stream={:?}, function={:?}",
                        stream,
                        function
                    );
                    self.emit_event(Secs1MessageEvent::ErrorOccured(
                        SecsTransportError::NoMatchReplyTransaction(stream, function),
                    ));
                    return;
                }
            };

            let transaction = match self.transaction_manager.find(&transaction_key) {
                Some(t) => t,
                None => {
                    log::warn!(
                        "[message] send transaction not found: {:?}",
                        transaction_key
                    );
                    self.emit_event(Secs1MessageEvent::ErrorOccured(
                        SecsTransportError::NoSuchTransaction(transaction_key),
                    ));
                    return;
                }
            };

            transaction.handle_write(msg).unwrap();

            let outputs = Self::take_transaction_outputs(transaction);
            self.handle_transaction_outputs(outputs, &transaction_key);
        }
    }

    fn process_receive(&mut self, block: Secs1Block) {
        log::debug!("[message] process receive: {:?}", block.header);
        // Routing ERROR: 내가 다루는 deviceId가 아님 -> 에러 알리고 무시
        let transaction_key = self.get_transaction_key(TransferContext::Recv, &block.header);

        if !self.is_known_device(&block.header.device_id) {
            log::warn!("[message] unknown device id: {:?}", block.header.device_id);
            self.handle_unknown_device(block);
            return;
        }

        // 기존 트랜잭션이 있나? 없으면 생성 후 대응
        let transaction = match self.transaction_manager.find(&transaction_key) {
            Some(t) => t,
            None => match self.transaction_manager.create_recv(&transaction_key) {
                Some(t) => t,
                None => unreachable!("unreachable code"),
            },
        };

        transaction.handle_read(block).unwrap();
        let outputs = Self::take_transaction_outputs(transaction);
        self.handle_transaction_outputs(outputs, &transaction_key);
    }

    fn handle_effects(
        &mut self,
        effects: Vec<Secs1TransactionEffect>,
        transaction_key: &TransactionKey,
    ) {
        for effect in effects {
            log::debug!("[message] handle effect: {:?}", effect);
            match effect {
                Secs1TransactionEffect::StartTimeout(secs_timeout_unit) => {
                    self.start_timeout(secs_timeout_unit);
                }
                Secs1TransactionEffect::ClearTimeout(secs_timeout_unit) => {
                    self.cancel_timeout(secs_timeout_unit);
                }
                Secs1TransactionEffect::ErrorOccured(error) => {
                    if let SecsTransportError::Timeout(timeout_unit) = error {
                        self.emit_event(Secs1MessageEvent::MessageTimeout {
                            transaction_key: *transaction_key,
                            timeout_unit,
                        });
                    } else {
                        self.emit_event(Secs1MessageEvent::ErrorOccured(error));
                    }
                }
                Secs1TransactionEffect::RecvComplete => {
                    self.emit_event(Secs1MessageEvent::RecvComplete {
                        transaction_key: *transaction_key,
                    })
                }
                Secs1TransactionEffect::SendComplete => {
                    self.emit_event(Secs1MessageEvent::SendComplete {
                        transaction_key: *transaction_key,
                    });
                }
                Secs1TransactionEffect::TransactionEnd => {
                    self.transaction_manager.remove(transaction_key);
                }
                Secs1TransactionEffect::ReplyRequired(stream, function, key) => {
                    let reply_key = ReplyMsgKey(stream, function);
                    self.reply_map.insert(reply_key, key);
                }
            }
        }
    }

    /// receive 중 unknown device 발견 시 대응 처리
    fn handle_unknown_device(&mut self, block: Secs1Block) {
        log::warn!(
            "[message] handle unknown device: {:?}",
            block.header.device_id
        );
        // 상위 측으로 이벤트 알림
        self.emit_event(Secs1MessageEvent::ErrorOccured(
            SecsTransportError::UnknownDeviceId(block.header.device_id),
        ));

        let transaction_key = self.get_transaction_key(TransferContext::Recv, &block.header);
        // 트랜잭션으로 등록되어 있었던 경우라면 트랜잭션도 취소 처리
        self.transaction_manager.remove(&transaction_key);
    }

    fn process_timeout(&mut self, ticket: TimeoutTicket) {
        log::debug!("[message] process timeout: {:?}", ticket.timeout);
        // 1. _now에서 타임아웃 유닛 추출 (구조체에 맞게 메서드 호출)
        let is_timeout_valid = self.timeout_manager.fire(&ticket);
        if !is_timeout_valid {
            // 이미 취소된 타임아웃인 경우 -> 무시
            return;
        }
        let timeout = ticket.timeout;
        let transaction_key = timeout.to_transaction_key();

        let Some(transaction) = self.transaction_manager.find(&transaction_key) else {
            return;
        };

        transaction.handle_timeout(ticket.timeout).unwrap();
        let outputs = Self::take_transaction_outputs(transaction);
        self.handle_transaction_outputs(outputs, &transaction_key);
    }

    // 외부 signal에 대해 아이템을 처리한다.
    fn process_signal(&mut self, signal: Secs1MessageSignal) {
        log::debug!("[message] process signal");
        // send 처리를 위함
        let header = match signal {
            Secs1MessageSignal::BlockSendSuccess { header } => header,
            Secs1MessageSignal::BlockSendFailed { header } => header,
        };

        // 내가 보낸 것에 대한 트랜잭션 키를 얻어 옴
        let transaction_key = self.get_transaction_key(TransferContext::Send, &header);
        let Some(transaction) = self.transaction_manager.find(&transaction_key) else {
            log::debug!("[message] no transaction for signal: {:?}", transaction_key);
            return;
        };

        let _ = transaction.handle_event(signal);
        let outputs = Self::take_transaction_outputs(transaction);
        self.handle_transaction_outputs(outputs, &transaction_key);
    }

    /// device id가 알려진 장치인지 체크
    fn is_known_device(&self, device_id: &DeviceId) -> bool {
        self.device_id == *device_id
    }
}

impl Protocol<Secs1Block, Secs1Message, Secs1MessageSignal> for Secs1MessageMachine {
    type Rout = Secs1Message;
    type Wout = Secs1Block;
    type Eout = Secs1MessageEvent;
    type Error = SecsTransportError;
    type Time = TimeoutTicket;

    fn handle_read(&mut self, msg: Secs1Block) -> Result<(), Self::Error> {
        self.process_receive(msg);
        // self.run();
        Ok(())
    }

    fn poll_read(&mut self) -> Option<Self::Rout> {
        self.outgoing_msgs.pop_front()
    }

    fn handle_write(&mut self, msg: Secs1Message) -> Result<(), Self::Error> {
        self.process_send(msg);
        // self.run();
        Ok(())
    }

    fn poll_write(&mut self) -> Option<Self::Wout> {
        self.outgoing_blocks.pop_front()
    }

    /// timeout 발생 시 처리
    fn handle_timeout(&mut self, timeticket: Self::Time) -> Result<(), Self::Error> {
        self.process_timeout(timeticket);
        Ok(())
    }

    fn poll_timeout(&mut self) -> Option<Self::Time> {
        self.outgoing_timeouts.pop_front()
    }

    fn handle_event(&mut self, signal: Secs1MessageSignal) -> Result<(), Self::Error> {
        self.process_signal(signal);
        // self.run();
        Ok(())
    }

    /// 외부 시스템에서 전송할 메시지를 가져간다.
    fn poll_event(&mut self) -> Option<Self::Eout> {
        self.outgoing_events.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::transport::secs1::{Secs1Message, Secs1MessageHeader};
    use crate::transport::{
        ConnectionRole, DeviceId, Rbit, SystemByte, TransactionKey, TransactionOwner, Wbit,
        secs1::{config::Secs1TransportConfig, convert::encode},
    };
    use core::time::Duration;
    use sansio::Protocol;
    use secs_ii::{FunctionId, StreamId, item::Secs2Variant};

    fn build_primary_message(
        device_id: DeviceId,
        system_byte: SystemByte,
        rbit: Rbit,
        need_reply: bool,
    ) -> Secs1Message {
        let stream = StreamId(1);
        let function = FunctionId(3);

        let header = Secs1MessageHeader {
            device_id,
            rbit,
            stream,
            function,
            wbit: Wbit(need_reply),
            system_byte,
        };

        let body = Secs2Variant::list(vec![Secs2Variant::uint4(3001), Secs2Variant::uint4(3002)]);

        Secs1Message::new(header, Some(body))
    }

    fn build_secondary_message(
        device_id: DeviceId,
        system_byte: SystemByte,
        rbit: Rbit,
        need_reply: bool,
    ) -> Secs1Message {
        let stream = StreamId(1);
        let function = FunctionId(4);
        let header = Secs1MessageHeader {
            device_id,
            rbit,
            stream,
            function,
            wbit: Wbit(need_reply),
            system_byte,
        };

        let body = Secs2Variant::list(vec![Secs2Variant::uint8(1500), Secs2Variant::uint8(1501)]);

        Secs1Message::new(header, Some(body))
    }

    fn build_machine() -> Secs1MessageMachine {
        let config = Secs1TransportConfig {
            device_id: DeviceId(10),
            local_role: ConnectionRole::Active,
            t1_timeout: Duration::from_millis(100),
            t2_timeout: Duration::from_millis(100),
            t3_timeout: Duration::from_millis(100),
            t4_timeout: Duration::from_millis(100),
            t2_rty_limit: 3,
        };
        Secs1MessageMachine::new(&config)
    }

    fn drain_events(machine: &mut Secs1MessageMachine) -> Vec<Secs1MessageEvent> {
        let mut events = Vec::new();
        while let Some(event) = machine.poll_event() {
            events.push(event);
        }
        events
    }

    fn drain_messages(machine: &mut Secs1MessageMachine) -> Vec<Secs1Message> {
        let mut messages = Vec::new();
        while let Some(msg) = machine.poll_read() {
            messages.push(msg);
        }
        messages
    }

    fn drain_blocks(machine: &mut Secs1MessageMachine) -> Vec<Secs1Block> {
        let mut blocks = Vec::new();
        while let Some(block) = machine.poll_write() {
            blocks.push(block);
        }
        blocks
    }

    #[test]
    fn test_active_send_primary_need_reply() {
        let mut machine = build_machine();
        let device_id = machine.device_id;
        let system_byte = SystemByte(101);
        let msg = build_primary_message(device_id, system_byte, Rbit::FORWARD, true);
        let expected_key = TransactionKey::new(TransactionOwner::Local, system_byte);

        machine.handle_write(msg).unwrap();

        let mut blocks = drain_blocks(&mut machine);
        assert_eq!(blocks.len(), 1);
        let block = blocks.remove(0);

        machine
            .handle_event(Secs1MessageSignal::BlockSendSuccess {
                header: block.header,
            })
            .unwrap();

        let events = drain_events(&mut machine);
        assert!(events.iter().any(|event| {
            matches!(
                event,
                Secs1MessageEvent::SendComplete {
                    transaction_key
                } if *transaction_key == expected_key
            )
        }));
        assert_eq!(
            machine.poll_timeout().map(|ticket| ticket.timeout),
            Some(SecsTimeoutUnit::T3(expected_key))
        );

        let reply = build_secondary_message(device_id, system_byte, Rbit::REVERSE, false);
        let reply_block = encode(reply).unwrap().into_iter().next().unwrap();

        machine.handle_read(reply_block).unwrap();

        let messages = drain_messages(&mut machine);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].header.system_byte, system_byte);

        let events = drain_events(&mut machine);
        assert!(events.iter().any(|event| {
            matches!(
                event,
                Secs1MessageEvent::RecvComplete {
                    transaction_key
                } if *transaction_key == expected_key
            )
        }));
        assert!(machine.poll_timeout().is_none());
    }

    #[test]
    fn test_active_send_primary_no_reply() {
        let mut machine = build_machine();
        let device_id = machine.device_id;
        let system_byte = SystemByte(102);
        let msg = build_primary_message(device_id, system_byte, Rbit::FORWARD, false);
        let expected_key = TransactionKey::new(TransactionOwner::Local, system_byte);

        machine.handle_write(msg).unwrap();
        let mut blocks = drain_blocks(&mut machine);
        assert_eq!(blocks.len(), 1);
        let block = blocks.remove(0);

        machine
            .handle_event(Secs1MessageSignal::BlockSendSuccess {
                header: block.header,
            })
            .unwrap();

        let events = drain_events(&mut machine);
        assert!(events.iter().any(|event| {
            matches!(
                event,
                Secs1MessageEvent::SendComplete {
                    transaction_key
                } if *transaction_key == expected_key
            )
        }));
        assert!(machine.poll_timeout().is_none());
    }

    #[test]
    fn test_active_send_primary_timeout_t3() {
        let mut machine = build_machine();
        let device_id = machine.device_id;
        let system_byte = SystemByte(103);
        let msg = build_primary_message(device_id, system_byte, Rbit::FORWARD, true);
        let expected_key = TransactionKey::new(TransactionOwner::Local, system_byte);

        machine.handle_write(msg).unwrap();
        let mut blocks = drain_blocks(&mut machine);
        let block = blocks.remove(0);
        machine
            .handle_event(Secs1MessageSignal::BlockSendSuccess {
                header: block.header,
            })
            .unwrap();

        let timeout = machine.poll_timeout().unwrap();
        assert!(matches!(
            timeout,
            TimeoutTicket {
                timeout: SecsTimeoutUnit::T3(TransactionKey { .. }),
                ..
            }
        ));

        let events = drain_events(&mut machine);
        assert!(!events.is_empty());
        assert!(events.iter().any(|event| matches!(
            event,
            Secs1MessageEvent::SendComplete {
                transaction_key
            } if *transaction_key == expected_key
        )));

        // timeout 처리
        machine.handle_timeout(timeout).unwrap();
        let events = drain_events(&mut machine);

        // timeout 발생했다고 알림
        assert!(events.iter().any(|it| matches!(
            it,
            Secs1MessageEvent::MessageTimeout {
                transaction_key,
                timeout_unit: SecsTimeoutUnit::T3(_)
            } if *transaction_key == expected_key
        )));
    }

    #[test]
    fn test_active_recv_primary_need_reply() {
        let mut machine = build_machine();
        let device_id = machine.device_id;
        let system_byte = SystemByte(202);
        let primary = build_primary_message(device_id, system_byte, Rbit::REVERSE, true);
        let recv_block = encode(primary).unwrap().into_iter().next().unwrap();
        let expected_key = TransactionKey::new(TransactionOwner::Remote, system_byte);

        machine.handle_read(recv_block).unwrap();
        let _ = drain_messages(&mut machine);
        let events = drain_events(&mut machine);
        assert!(!events.is_empty());
        assert!(events.iter().any(|event| matches!(
            event,
            Secs1MessageEvent::RecvComplete {
                transaction_key
            } if *transaction_key == expected_key
        )));
        assert!(machine.poll_timeout().is_none());

        let reply = build_secondary_message(device_id, system_byte, Rbit::FORWARD, false);
        machine.handle_write(reply).unwrap();

        let mut blocks = drain_blocks(&mut machine);
        assert_eq!(blocks.len(), 1);
        let block = blocks.remove(0);
        machine
            .handle_event(Secs1MessageSignal::BlockSendSuccess {
                header: block.header,
            })
            .unwrap();
        let events = drain_events(&mut machine);
        assert!(events.iter().any(|it| matches!(
            it,
            Secs1MessageEvent::SendComplete {
                transaction_key
            } if *transaction_key == expected_key
        )));
        assert!(machine.poll_timeout().is_none());
    }
}
