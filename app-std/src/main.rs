use std::collections::HashMap;
use std::net::SocketAddr;
use std::thread;
use std::time::{Duration, Instant};

use secs_common::{ConnectionRole, SecsTimeoutUnit, SystemByteSource, TimeoutId, TimeoutTicket};
use secs_ii::item::Secs2Variant;
use secs_ii::{FunctionId, Secs2Message, StreamId};
use secs_runtime::{HandlerError, SecsRuntime, SecsService, ServiceContext, TimeoutConfig};
use secs_runtime_core::{RuntimeTimer, Timer};
use secs_runtime_std::TcpServerDataSource;
use secs_transport::transport::SessionId;
use secs_transport::transport::hsms::config::HsmsTransportConfig;
use secs_transport::transport::hsms::protocol::HsmsTransport;

struct AppTimer {
    deadlines: HashMap<TimeoutId, (Instant, TimeoutTicket)>,
    next_id: u64,
}

impl AppTimer {
    fn new() -> Self {
        Self {
            deadlines: HashMap::new(),
            next_id: 1,
        }
    }
}

impl Timer for AppTimer {
    type Error = ();
    type Duration = Duration;
    type Handle = TimeoutId;

    fn start_after(&mut self, duration: Self::Duration) -> Result<Self::Handle, Self::Error> {
        let id = TimeoutId(self.next_id);
        self.next_id = self.next_id.wrapping_add(1);
        self.deadlines.insert(
            id,
            (
                Instant::now() + duration,
                TimeoutTicket {
                    id,
                    timeout: SecsTimeoutUnit::T8,
                },
            ),
        );
        Ok(id)
    }

    fn cancel(&mut self, handle: Self::Handle) -> Result<(), Self::Error> {
        self.deadlines.remove(&handle);
        Ok(())
    }
}

impl RuntimeTimer for AppTimer {
    fn start_secs_timeout(
        &mut self,
        ticket: TimeoutTicket,
        duration: Self::Duration,
    ) -> Result<Self::Handle, Self::Error> {
        self.deadlines
            .insert(ticket.id, (Instant::now() + duration, ticket));
        Ok(ticket.id)
    }

    fn cancel_secs_timeout(&mut self, handle: Self::Handle) -> Result<(), Self::Error> {
        self.cancel(handle)
    }

    fn poll_secs_timeout(&mut self) -> Result<Option<TimeoutTicket>, Self::Error> {
        let now = Instant::now();
        let expired = self
            .deadlines
            .iter()
            .find_map(|(id, (deadline, _))| (*deadline <= now).then_some(*id));

        Ok(expired.and_then(|id| self.deadlines.remove(&id).map(|(_, ticket)| ticket)))
    }
}

fn timeout_config(config: &HsmsTransportConfig) -> TimeoutConfig<Duration> {
    TimeoutConfig {
        t1: Duration::from_secs(1),
        t2: Duration::from_secs(1),
        t3: config.t3_timeout,
        t4: Duration::from_secs(1),
        t5: config.t5_timeout,
        t6: config.t6_timeout,
        t7: config.t7_timeout,
        t8: config.t8_timeout,
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

fn build_s1f14_reply(request: &Secs2Message) -> Option<Secs2Message> {
    if request.stream != StreamId(1) || request.function != FunctionId(13) {
        return None;
    }

    Some(Secs2Message::new(
        StreamId(1),
        FunctionId(14),
        false,
        Some(Secs2Variant::list(vec![
            Secs2Variant::ascii("TESTLINE1"),
            Secs2Variant::ascii("TESTLINE2"),
        ])),
    ))
}

struct EstablishCommunicationService;

impl SecsService for EstablishCommunicationService {
    fn serve(&mut self, ctx: &mut ServiceContext) -> Result<(), HandlerError> {
        let Some(message) = ctx.recv() else {
            return Ok(());
        };

        log::debug!(
            "received routed message: S{}F{}, need_reply={}",
            message.stream.0,
            message.function.0,
            message.need_reply
        );

        if let Some(reply) = build_s1f14_reply(&message) {
            log::debug!("send reply: S1F14");
            ctx.send(reply)?;
        }

        Ok(())
    }
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
    let transport = HsmsTransport::new(&config, Box::new(source), SystemByteSource::new());
    let timer = AppTimer::new();
    let mut runtime = SecsRuntime::new(
        transport,
        timer,
        SystemByteSource::new(),
        timeout_config(&config),
    );
    runtime.register_service(StreamId(1), FunctionId(13), EstablishCommunicationService);

    log::debug!("starting HSMS active transport: {}", local_addr);
    if let Err(error) = runtime.start() {
        log::error!("failed to start runtime: {:?}", error);
        return;
    }

    loop {
        if let Err(error) = runtime.tick() {
            log::error!("runtime tick failed: {:?}", error);
            break;
        }

        while let Some(message) = runtime.poll_inbox() {
            log::debug!(
                "received unrouted message: S{}F{}, need_reply={}",
                message.stream.0,
                message.function.0,
                message.need_reply
            );
        }

        thread::sleep(Duration::from_millis(10));
    }
}
