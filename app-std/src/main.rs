use std::collections::HashMap;
use std::net::SocketAddr;
use std::thread;
use std::time::{Duration, Instant};

use secs_common::{ConnectionRole, SecsTimeoutUnit, SystemByteSource, TimeoutId, TimeoutTicket};
use secs_ii::item::Secs2Variant;
use secs_ii::{FunctionId, Secs2Message, StreamId};
use secs_runtime_core::{MessageTransport, RuntimeMessage};
use secs_runtime_std::TcpServerDataSource;
use secs_transport::transport::SessionId;
use secs_transport::transport::hsms::config::HsmsTransportConfig;
use secs_transport::transport::hsms::protocol::HsmsTransport;

struct AppTimer {
    deadlines: HashMap<TimeoutId, (Instant, TimeoutTicket)>,
}

impl AppTimer {
    fn new() -> Self {
        Self {
            deadlines: HashMap::new(),
        }
    }

    fn start(&mut self, ticket: TimeoutTicket, duration: Duration) {
        self.deadlines
            .insert(ticket.id, (Instant::now() + duration, ticket));
    }

    fn poll_expired(&mut self) -> Vec<TimeoutTicket> {
        let now = Instant::now();
        let expired = self
            .deadlines
            .iter()
            .filter_map(|(id, (deadline, _))| (*deadline <= now).then_some(*id))
            .collect::<Vec<_>>();

        expired
            .into_iter()
            .filter_map(|id| self.deadlines.remove(&id).map(|(_, ticket)| ticket))
            .collect()
    }
}

fn timeout_duration(config: &HsmsTransportConfig, unit: SecsTimeoutUnit) -> Duration {
    match unit {
        SecsTimeoutUnit::T3(_) => config.t3_timeout,
        SecsTimeoutUnit::T5 => config.t5_timeout,
        SecsTimeoutUnit::T6 => config.t6_timeout,
        SecsTimeoutUnit::T7 => config.t7_timeout,
        SecsTimeoutUnit::T8 => config.t8_timeout,
        _ => Duration::from_secs(1),
    }
}

fn build_config(remote_addr: SocketAddr) -> HsmsTransportConfig {
    HsmsTransportConfig {
        session_id: SessionId(0),
        connection_mode: ConnectionRole::Passive,
        t3_timeout: Duration::from_secs(45),
        t5_timeout: Duration::from_secs(10),
        t6_timeout: Duration::from_secs(5),
        t7_timeout: Duration::from_secs(10),
        t8_timeout: Duration::from_secs(5),
        local_addr: SocketAddr::from(([0, 0, 0, 0], 0)),
        remote_addr,
    }
}

fn build_s1f14_reply(request: &RuntimeMessage) -> Option<RuntimeMessage> {
    if request.stream() != StreamId(1) || request.function() != FunctionId(13) {
        return None;
    }

    let payload = Secs2Message::new(
        StreamId(1),
        FunctionId(14),
        false,
        Some(Secs2Variant::list(vec![
            Secs2Variant::ascii("TESTLINE1"),
            Secs2Variant::ascii("TESTLINE2"),
        ])),
    );
    Some(RuntimeMessage::new_local(request.system_byte(), payload))
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    // let remote_addr = env::args()
    //     .nth(1)
    //     .unwrap_or_else(|| "127.0.0.1:6000".to_string())
    //     .parse::<SocketAddr>()
    //     .expect("invalid remote address");
    let local_addr = "127.0.0.1:7020"
        .parse::<SocketAddr>()
        .expect("invalid remote addr");

    let config = build_config(local_addr);
    let source = TcpServerDataSource::new(local_addr);
    let mut transport = HsmsTransport::new(&config, Box::new(source), SystemByteSource::new());
    let mut timer = AppTimer::new();

    println!("starting HSMS active transport: {}", local_addr);
    if let Err(error) = transport.start() {
        eprintln!("failed to start transport: {:?}", error);
        return;
    }

    loop {
        if let Err(error) = transport.poll() {
            eprintln!("transport poll failed: {:?}", error);
            break;
        }

        while let Some(ticket) = transport.poll_timeout() {
            let duration = timeout_duration(&config, ticket.timeout);
            println!("start timeout: {:?} for {:?}", ticket, duration);
            timer.start(ticket, duration);
        }

        for ticket in timer.poll_expired() {
            println!("timeout expired: {:?}", ticket);
            if let Err(error) = transport.handle_timeout(ticket) {
                eprintln!("transport timeout handling failed: {:?}", error);
                break;
            }
        }

        while let Some(message) = transport.poll_recv() {
            println!(
                "received message: S{}F{}, system_byte={:?}, need_reply={}",
                message.stream().0,
                message.function().0,
                message.system_byte(),
                message.need_reply()
            );

            if let Some(reply) = build_s1f14_reply(&message) {
                println!("send reply: S1F14, system_byte={:?}", reply.system_byte());
                if let Err(error) = transport.send(reply) {
                    eprintln!("failed to send reply: {:?}", error);
                    break;
                }
            }
        }

        thread::sleep(Duration::from_millis(10));
    }
}
