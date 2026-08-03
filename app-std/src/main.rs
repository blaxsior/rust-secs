use std::net::SocketAddr;
use std::thread;
use std::time::Duration;

use secs_common::{ConnectionRole, SystemByteSource};
use secs_ii::item::Secs2Variant;
use secs_ii::{FunctionId, Secs2Message, StreamId};
use secs_model::{ValueData, ValueDictionary, ValueId, ValueSpec};
use secs_runtime::{SecsHandle, SecsRuntime, TimeoutConfig};
use secs_runtime_std::model::{JsonCodec, ValueDataFileRepository, ValueSpecFileRepository, YamlCodec};
use secs_runtime_std::{FileDataStore, StdSecsTimer, TcpServerDataSource};
use secs_transport::transport::SessionId;
use secs_transport::transport::hsms::config::HsmsTransportConfig;
use secs_transport::transport::hsms::protocol::HsmsTransport;

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
        t3_timeout: Duration::from_secs(5),
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

fn build_s1f13_request() -> Secs2Message {
    Secs2Message::new(
        StreamId(1),
        FunctionId(13),
        true,
        Some(Secs2Variant::list(vec![
            Secs2Variant::ascii("hello"),
            Secs2Variant::ascii("world"),
        ])),
    )
}

fn init_values() {
    let spec_store = match FileDataStore::<_, ValueSpec>::new(
        "app-std/config/value-spec.yml",
        YamlCodec,
    ) {
        Ok(store) => store,
        Err(error) => {
            log::error!("failed to open value spec store: {:?}", error);
            return;
        }
    };
    let data_store = match FileDataStore::<_, ValueData>::new(
        "app-std/data/value-data.yml",
        YamlCodec,
    ) {
        Ok(store) => store,
        Err(error) => {
            log::error!("failed to open value data store: {:?}", error);
            return;
        }
    };

    let spec_repository = ValueSpecFileRepository::new(spec_store);
    let data_repository = ValueDataFileRepository::new(data_store);
    let mut values = match ValueDictionary::with_store(spec_repository, data_repository) {
        Ok(values) => values,
        Err(error) => {
            log::error!("failed to initialize value dictionary: {:?}", error);
            return;
        }
    };

    if let Ok(v) = values.read(&ValueId::from("MLDN")) {
        log::info!("mldn init = {:?}", v);
    }
    
    if let Ok(v) = values.read(&ValueId::from("SOFTREV")) {
        log::info!("softrev init  = {:?}", v);
    }

    if let Err(error) = values.write(&ValueId::from("MLDN"), Secs2Variant::ascii("TESTMODELNO")) {
        log::error!("failed to write MLDN: {:?}", error);
    }

    if let Err(error) = values.write(&ValueId::from("SOFTREV"), Secs2Variant::ascii("0.1.0")) {
        log::error!("failed to write SOFTREV: {:?}", error);
    }
}

async fn process_incoming(handle: SecsHandle) {
    loop {
        match handle.recv().await {
            Ok(inbound) => {
                let key = inbound.transaction_key;
                let message = inbound.payload;

                log::debug!(
                    "received message: S{}F{}, need_reply={}",
                    message.stream.0,
                    message.function.0,
                    message.need_reply
                );

                if let Some(reply) = build_s1f14_reply(&message) {
                    log::debug!("send response: S1F14");
                    handle.reply(key, reply);
                }
            }
            Err(error) => log::error!("recv failed: {:?}", error),
        }
    }
}

async fn request_establish_communication(handle: SecsHandle) {
    match handle.request(build_s1f13_request()).await {
        Ok(data) => log::info!(
            "request reply S{}F{} W={}",
            data.stream.0,
            data.function.0,
            data.need_reply
        ),
        Err(error) => log::error!("request failed: {:?}", error),
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    init_values();

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
    let timer = StdSecsTimer::new();
    let mut runtime = SecsRuntime::new(
        transport,
        timer,
        SystemByteSource::new(),
        timeout_config(&config),
    );
    let handle = runtime.handle();

    log::debug!("starting HSMS active transport: {}", local_addr);
    if let Err(error) = runtime.start() {
        log::error!("failed to start runtime: {:?}", error);
        return;
    }

    let runtime_thread = thread::spawn(move || {
        loop {
            if let Err(error) = runtime.tick() {
                log::error!("runtime tick failed: {:?}", error);
                if matches!(
                    error,
                    secs_runtime::SecsRuntimeError::Transport(
                        secs_runtime_core::MachineError::DataSourceError(
                            secs_runtime_core::ByteDataSourceError::NotOpen
                        )
                    )
                ) {
                    thread::sleep(Duration::from_millis(200));
                }
            }

            thread::sleep(Duration::from_millis(10));
        }
    });

    let incoming_handle = handle.clone();
    let _incoming_thread = thread::spawn(move || {
        futures::executor::block_on(process_incoming(incoming_handle));
    });

    let request_handle = handle.clone();
    let _request_thread = thread::spawn(move || {
        thread::sleep(Duration::from_secs(20));
        futures::executor::block_on(request_establish_communication(request_handle));
    });

    let _ = runtime_thread.join();
}
