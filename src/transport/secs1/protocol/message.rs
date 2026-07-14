use crate::core::SecsMessage;
use crate::transport::DeviceId;
use crate::transport::secs1::protocol::message::transaction::Secs1TransactionEffect;
use crate::transport::secs1::protocol::message::transaction_manager::Secs1TransactionManager;
use crate::{
    transport::{
        ConnectionRole, SecsTimeoutUnit, SystemByte, TransactionKey,
        error::SecsTransportError,
        secs1::{
            block::{Secs1Block, Secs1BlockHeader},
            config::Secs1TransportConfig,
        },
    },
    util::time::{TimeoutManager, TimeoutTicket},
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
    SendComplete { transaction_key: TransactionKey },
    /// 메시지를 성공적으로 수신
    RecvComplete { transaction_key: TransactionKey },

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
    outgoing_msgs: VecDeque<SecsMessage>,
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

    /// 연결 모드 -> active or passive
    role: ConnectionRole,

    /// 다음에 사용할 트랜잭션 ID (source 주도 기준)
    next_system_byte: SystemByte,
    /// 트랜잭션 관리 구조체
    transaction_manager: Secs1TransactionManager,
}

/// 상대방 요청에 응답하기 위한 트랜잭션 식별 키
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct ReplyMsgKey(StreamId, FunctionId);

impl Secs1MessageMachine {
    pub fn new(config: &Secs1TransportConfig) -> Self {
        Self {
            outgoing_blocks: VecDeque::new(),
            outgoing_msgs: VecDeque::new(),
            outgoing_timeouts: VecDeque::new(),
            outgoing_events: VecDeque::new(),
            timeout_manager: TimeoutManager::new(),
            device_id: config.device_id,
            reply_map: BTreeMap::new(),
            role: config.local_role,
            next_system_byte: SystemByte(0),
            transaction_manager: Secs1TransactionManager::new(),
        }
    }
    /// 외부 시스템에 전송할 메시지를 담는다.
    fn emit_event(&mut self, output: Secs1MessageEvent) {
        self.outgoing_events.push_back(output);
    }

    /// timeout을 시작한다.
    fn start_timeout(&mut self, timeout: SecsTimeoutUnit) {
        let ticket = self.timeout_manager.issue(timeout);
        self.outgoing_timeouts.push_back(ticket);
    }

    /// timeout을 취소한다.
    fn cancel_timeout(&mut self, timeout: SecsTimeoutUnit) {
        self.timeout_manager.cancel(timeout);
    }

    /// block을 쓴다
    fn write_block(&mut self, block: Secs1Block) {
        self.outgoing_blocks.push_back(block);
    }

    /// SystemByte를 만든다
    fn generate_system_byte(&mut self) -> SystemByte {
        let current = self.next_system_byte;
        self.next_system_byte = current.next();
        return current;
    }

    fn take_reply_transaction_key(
        &mut self,
        stream: StreamId,
        function: FunctionId,
    ) -> Option<TransactionKey> {
        self.reply_map.remove(&ReplyMsgKey(stream, function))
    }

    /// header로부터 transaction_key를 획득
    fn get_transaction_key(&self, header: &Secs1BlockHeader) -> TransactionKey {
        let direction = header.rbit;
        let role = self.role;
        let system_byte = header.system_byte;

        TransactionKey::from(role, direction, system_byte)
    }

    fn process_send(&mut self, msg: SecsMessage) {
        // 1.  SxFy, y&1 == 0인 경우 -> 기존 프로세스 참조
        let stream = msg.payload.stream;
        let function = msg.payload.function;

        if function.is_primary() {
            let system_byte = self.generate_system_byte();

            // 요청 -> 트랜잭션을 새롭게 생성
            let transaction_key = TransactionKey::from(self.role, msg.rbit.into(), system_byte);

            let transaction = match self.transaction_manager.create_send(&transaction_key, msg) {
                Some(t) => t,
                None => {
                    self.emit_event(Secs1MessageEvent::ErrorOccured(
                        SecsTransportError::NoSuchTransaction(transaction_key),
                    ));
                    return;
                }
            };
            if let Some(block) = transaction.poll_send() {
                self.write_block(block);
            }
        } else {
            // 상대 request 받고 대응되는 메시지 보내는 상황
            let transaction_key = match self.take_reply_transaction_key(stream, function) {
                Some(key) => key,
                None => {
                    self.emit_event(Secs1MessageEvent::ErrorOccured(
                        SecsTransportError::NoMatchReplyTransaction(stream, function),
                    ));
                    return;
                }
            };

            let transaction = match self.transaction_manager.find(&transaction_key) {
                Some(t) => t,
                None => {
                    self.emit_event(Secs1MessageEvent::ErrorOccured(
                        SecsTransportError::NoSuchTransaction(transaction_key),
                    ));
                    return;
                }
            };

            transaction.handle_reply(msg);
            if let Some(block) = transaction.poll_send() {
                self.write_block(block);
            }
        }
    }

    fn process_receive(&mut self, block: Secs1Block) {
        // Routing ERROR: 내가 다루는 deviceId가 아님 -> 에러 알리고 무시
        let transaction_key = self.get_transaction_key(&block.header);

        if !self.is_known_device(&block.header.device_id) {
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

        transaction.handle_receive(block);
        let effects = transaction.poll_effects();
        self.handle_effects(effects, &transaction_key);
    }

    fn handle_effects(
        &mut self,
        effects: Vec<Secs1TransactionEffect>,
        transaction_key: &TransactionKey,
    ) {
        for effect in effects {
            match effect {
                Secs1TransactionEffect::StartTimeout(secs_timeout_unit) => {
                    self.start_timeout(secs_timeout_unit);
                }
                Secs1TransactionEffect::ClearTimeout(secs_timeout_unit) => {
                    self.cancel_timeout(secs_timeout_unit);
                }
                Secs1TransactionEffect::ErrorOccured(error) => {
                    self.emit_event(Secs1MessageEvent::ErrorOccured(error));
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
        // 상위 측으로 이벤트 알림
        self.emit_event(Secs1MessageEvent::ErrorOccured(
            SecsTransportError::UnknownDeviceId(block.header.device_id),
        ));

        let transaction_key = self.get_transaction_key(&block.header);
        // 트랜잭션으로 등록되어 있었던 경우라면 트랜잭션도 취소 처리
        self.transaction_manager.remove(&transaction_key);
    }

    fn process_timeout(&mut self, ticket: TimeoutTicket) {
        // 1. _now에서 타임아웃 유닛 추출 (구조체에 맞게 메서드 호출)
        let is_timeout_valid = self.timeout_manager.fire(&ticket);
        if !is_timeout_valid {
            // 이미 취소된 타임아웃인 경우 -> 무시
            return;
        }
        let timeout = ticket.timeout;
        let transaction_key = timeout.to_transaction_key();

        // 타임아웃 발생 -> 해당 transaction abort = 제거 처리 후 이벤트 발생
        if self.transaction_manager.find(&transaction_key).is_some() {
            self.transaction_manager.remove(&transaction_key);
            self.emit_event(Secs1MessageEvent::MessageTimeout {
                transaction_key,
                timeout_unit: ticket.timeout,
            });
        };

        // timeout 발생 -> 대상 트랜잭션 abort 처리
    }

    // 외부 signal에 대해 아이템을 처리한다.
    fn process_signal(&mut self, signal: Secs1MessageSignal) {
        // send 처리를 위함
        let header = match signal {
            Secs1MessageSignal::BlockSendSuccess { header } => header,
            Secs1MessageSignal::BlockSendFailed { header } => header,
        };

        let transaction_key = self.get_transaction_key(&header);
        let Some(transaction) = self.transaction_manager.find(&transaction_key) else {
            return;
        };

        transaction.handle_signal(signal);

        let block = transaction.poll_send();
        let effects = transaction.poll_effects();
        
        if let Some(block) = block {
            self.write_block(block);
        }

        self.handle_effects(effects, &transaction_key);
    }

    /// device id가 알려진 장치인지 체크
    fn is_known_device(&self, device_id: &DeviceId) -> bool {
        self.device_id == *device_id
    }
}

impl Protocol<Secs1Block, SecsMessage, Secs1MessageSignal> for Secs1MessageMachine {
    type Rout = SecsMessage;
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

    fn handle_write(&mut self, msg: SecsMessage) -> Result<(), Self::Error> {
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
