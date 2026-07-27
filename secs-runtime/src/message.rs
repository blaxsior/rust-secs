// use crate::core::{
//     ByteDataSource, MachineError, MachineEvent, MachineSignal, MessageTransport, RuntimeError,
//     RuntimeMessage, RuntimeTimer,
// };

// pub enum MessageRuntimeEvent {
//     Machine(MachineEvent),
//     Message(RuntimeMessage),
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
// pub struct MessageRuntimeTick {
//     pub timeout_count: usize,
//     pub machine_event_count: usize,
//     pub read_bytes: usize,
//     pub timeout_request_count: usize,
// }

// impl MessageRuntimeTick {
//     pub fn did_work(&self) -> bool {
//         self.timeout_count > 0
//             || self.machine_event_count > 0
//             || self.read_bytes > 0
//             || self.timeout_request_count > 0
//     }
// }

// pub struct MessageRuntime<D, M, T> {
//     datasource: D,
//     transport: M,
//     timer: T,
// }

// impl<D, M, T> MessageRuntime<D, M, T> {
//     pub fn new(datasource: D, transport: M, timer: T) -> Self {
//         Self {
//             datasource,
//             transport,
//             timer,
//         }
//     }

//     pub fn datasource(&self) -> &D {
//         &self.datasource
//     }

//     pub fn datasource_mut(&mut self) -> &mut D {
//         &mut self.datasource
//     }

//     pub fn machine(&self) -> &M {
//         &self.transport
//     }

//     pub fn machine_mut(&mut self) -> &mut M {
//         &mut self.transport
//     }

//     pub fn timer(&self) -> &T {
//         &self.timer
//     }

//     pub fn timer_mut(&mut self) -> &mut T {
//         &mut self.timer
//     }
// }

// impl<D, M, T> MessageRuntime<D, M, T>
// where
//     M: MessageTransport,
//     T: RuntimeTimer,
// {
//     pub fn send(&mut self, msg: RuntimeMessage) -> Result<(), MachineError> {
//         self.transport.write(msg)
//     }

//     pub fn recv(&mut self) -> Option<RuntimeMessage> {
//         self.transport.read()
//     }

//         fn handle_timeout(&mut self, ticket: TimeoutTicket) -> Result<(), crate::error::MachineError>;
//     fn poll_timeout(&mut self) -> Option<TimeoutTicket>;
// }

// impl<D, M, T> MessageRuntime<D, M, T>
// where
//     D: ByteDataSource,
//     M: MessageTransport,
//     T: RuntimeTimer,
// {
//     pub fn process_machine_event_once(
//         &mut self,
//     ) -> Result<bool, RuntimeError<D::Error, MachineError, T::Error>>
//     where
//         T: RuntimeTimer,
//     {
//         let Some(event) = self.transport.poll_event() else {
//             return Ok(false);
//         };

//         match event {
//             MachineEvent::LinkOpenRequested => {
//                 self.datasource.open().map_err(RuntimeError::DataSource)?;
//                 self.transport
//                     .handle_signal(MachineSignal::LinkOpened)
//                     .map_err(RuntimeError::Machine)?;
//             }
//             MachineEvent::LinkCloseRequested => {
//                 self.datasource.close().map_err(RuntimeError::DataSource)?;
//                 self.transport
//                     .handle_signal(MachineSignal::LinkClosed)
//                     .map_err(RuntimeError::Machine)?;
//             }
//         }

//         Ok(true)
//     }

//     fn process_machine_events(
//         &mut self,
//     ) -> Result<usize, RuntimeError<D::Error, MachineError, T::Error>>
//     where
//         T: RuntimeTimer,
//     {
//         let mut count = 0;

//         while self.process_machine_event_once()? {
//             count += 1;
//         }

//         Ok(count)
//     }

//     pub fn arm_machine_timeouts(&mut self) -> Result<(), T::Error> {
//         while let Some(ticket) = self.transport.poll_timeout() {
//             let _ = self.timer.start_secs_timeout(ticket)?;
//         }

//         Ok(())
//     }

//     fn arm_machine_timeouts_count(&mut self) -> Result<usize, T::Error> {
//         let mut count = 0;

//         while let Some(ticket) = self.transport.poll_timeout() {
//             let _ = self.timer.start_secs_timeout(ticket)?;
//             count += 1;
//         }

//         Ok(count)
//     }

//     pub fn process_timer_once(
//         &mut self,
//     ) -> Result<(), RuntimeError<D::Error, MachineError, T::Error>>
//     where
//         D: ByteDataSource,
//     {
//         let Some(ticket) = self
//             .timer
//             .poll_secs_timeout()
//             .map_err(RuntimeError::Timer)?
//         else {
//             return Ok(());
//         };

//         self.transport
//             .handle_timeout(ticket)
//             .map_err(RuntimeError::Machine)
//     }

//     fn process_timer_events(
//         &mut self,
//     ) -> Result<usize, RuntimeError<D::Error, MachineError, T::Error>>
//     where
//         D: ByteDataSource,
//     {
//         let mut count = 0;

//         while let Some(ticket) = self
//             .timer
//             .poll_secs_timeout()
//             .map_err(RuntimeError::Timer)?
//         {
//             self.transport
//                 .handle_timeout(ticket)
//                 .map_err(RuntimeError::Machine)?;
//             count += 1;
//         }

//         Ok(count)
//     }
// }

// impl<D, M, T> MessageRuntime<D, M, T>
// where
//     D: ByteDataSource,
//     M: MessageTransport,
//     T: RuntimeTimer,
// {
//     pub fn tick(
//         &mut self,
//         read_buf: &mut [u8],
//     ) -> Result<MessageRuntimeTick, RuntimeError<D::Error, MachineError, T::Error>> {
//         let mut report = MessageRuntimeTick::default();

//         report.timeout_count += self.process_timer_events()?;
//         report.machine_event_count += self.process_machine_events()?;
//         report.timeout_request_count += self
//             .arm_machine_timeouts_count()
//             .map_err(RuntimeError::Timer)?;

//         let _ = read_buf;
//         report.machine_event_count += self.process_machine_events()?;
//         report.timeout_request_count += self
//             .arm_machine_timeouts_count()
//             .map_err(RuntimeError::Timer)?;

//         report.machine_event_count += self.process_machine_events()?;
//         report.timeout_request_count += self
//             .arm_machine_timeouts_count()
//             .map_err(RuntimeError::Timer)?;

//         Ok(report)
//     }

//     pub fn run_until_idle(
//         &mut self,
//         read_buf: &mut [u8],
//         max_ticks: usize,
//     ) -> Result<MessageRuntimeTick, RuntimeError<D::Error, MachineError, T::Error>> {
//         let mut total = MessageRuntimeTick::default();

//         for _ in 0..max_ticks {
//             let tick = self.tick(read_buf)?;

//             total.timeout_count += tick.timeout_count;
//             total.machine_event_count += tick.machine_event_count;
//             total.read_bytes += tick.read_bytes;
//             total.timeout_request_count += tick.timeout_request_count;

//             if !tick.did_work() {
//                 break;
//             }
//         }

//         Ok(total)
//     }
// }
