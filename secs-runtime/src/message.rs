use crate::core::{
    ByteDataSource, MachineError, MachineEvent, MachineSignal, MessageMachine, RuntimeError,
    RuntimeMessage, RuntimeTimer,
};

pub enum MessageRuntimeEvent {
    Machine(MachineEvent),
    Message(RuntimeMessage),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MessageRuntimeTick {
    pub timeout_count: usize,
    pub machine_event_count: usize,
    pub read_bytes: usize,
    pub write_count: usize,
    pub timeout_request_count: usize,
}

impl MessageRuntimeTick {
    pub fn did_work(&self) -> bool {
        self.timeout_count > 0
            || self.machine_event_count > 0
            || self.read_bytes > 0
            || self.write_count > 0
            || self.timeout_request_count > 0
    }
}

pub struct MessageRuntime<D, M, T> {
    datasource: D,
    machine: M,
    timer: T,
}

impl<D, M, T> MessageRuntime<D, M, T> {
    pub fn new(datasource: D, machine: M, timer: T) -> Self {
        Self {
            datasource,
            machine,
            timer,
        }
    }

    pub fn datasource(&self) -> &D {
        &self.datasource
    }

    pub fn datasource_mut(&mut self) -> &mut D {
        &mut self.datasource
    }

    pub fn machine(&self) -> &M {
        &self.machine
    }

    pub fn machine_mut(&mut self) -> &mut M {
        &mut self.machine
    }

    pub fn timer(&self) -> &T {
        &self.timer
    }

    pub fn timer_mut(&mut self) -> &mut T {
        &mut self.timer
    }
}

impl<D, M, T> MessageRuntime<D, M, T>
where
    M: MessageMachine,
{
    pub fn send(&mut self, msg: RuntimeMessage) -> Result<(), MachineError> {
        self.machine.handle_write_message(msg)
    }

    pub fn signal(&mut self, signal: MachineSignal) -> Result<(), MachineError> {
        self.machine.handle_signal(signal)
    }

    pub fn recv(&mut self) -> Option<RuntimeMessage> {
        self.machine.poll_read_message()
    }

    pub fn poll_event(&mut self) -> Option<MessageRuntimeEvent> {
        if let Some(event) = self.machine.poll_event() {
            return Some(MessageRuntimeEvent::Machine(event));
        }

        self.machine
            .poll_read_message()
            .map(MessageRuntimeEvent::Message)
    }
}

impl<D, M, T> MessageRuntime<D, M, T>
where
    D: ByteDataSource,
    M: MessageMachine,
{
    pub fn process_machine_event_once(
        &mut self,
    ) -> Result<bool, RuntimeError<D::Error, MachineError, T::Error>>
    where
        T: RuntimeTimer,
    {
        let Some(event) = self.machine.poll_event() else {
            return Ok(false);
        };

        match event {
            MachineEvent::LinkOpenRequested => {
                self.datasource.open().map_err(RuntimeError::DataSource)?;
                self.machine
                    .handle_signal(MachineSignal::LinkOpened)
                    .map_err(RuntimeError::Machine)?;
            }
            MachineEvent::LinkCloseRequested => {
                self.datasource.close().map_err(RuntimeError::DataSource)?;
                self.machine
                    .handle_signal(MachineSignal::LinkClosed)
                    .map_err(RuntimeError::Machine)?;
            }
        }

        Ok(true)
    }

    fn process_machine_events(
        &mut self,
    ) -> Result<usize, RuntimeError<D::Error, MachineError, T::Error>>
    where
        T: RuntimeTimer,
    {
        let mut count = 0;

        while self.process_machine_event_once()? {
            count += 1;
        }

        Ok(count)
    }

    pub fn read_once(
        &mut self,
        buf: &mut [u8],
    ) -> Result<usize, RuntimeError<D::Error, MachineError, T::Error>>
    where
        T: RuntimeTimer,
    {
        let len = self
            .datasource
            .read(buf)
            .map_err(RuntimeError::DataSource)?;

        if len > 0 {
            self.machine
                .handle_read_bytes(&buf[..len])
                .map_err(RuntimeError::Machine)?;
        }

        Ok(len)
    }

    pub fn flush_writes(&mut self) -> Result<(), RuntimeError<D::Error, MachineError, T::Error>>
    where
        T: RuntimeTimer,
    {
        while let Some(bytes) = self.machine.poll_write_bytes() {
            self.datasource
                .write(&bytes)
                .map_err(RuntimeError::DataSource)?;
        }

        Ok(())
    }

    fn flush_writes_count(
        &mut self,
    ) -> Result<usize, RuntimeError<D::Error, MachineError, T::Error>>
    where
        T: RuntimeTimer,
    {
        let mut count = 0;

        while let Some(bytes) = self.machine.poll_write_bytes() {
            self.datasource
                .write(&bytes)
                .map_err(RuntimeError::DataSource)?;
            count += 1;
        }

        Ok(count)
    }
}

impl<D, M, T> MessageRuntime<D, M, T>
where
    M: MessageMachine,
    T: RuntimeTimer,
{
    pub fn arm_machine_timeouts(&mut self) -> Result<(), T::Error> {
        while let Some(timeout) = self.machine.poll_timeout() {
            let _ = self.timer.start(timeout)?;
        }

        Ok(())
    }

    fn arm_machine_timeouts_count(&mut self) -> Result<usize, T::Error> {
        let mut count = 0;

        while let Some(timeout) = self.machine.poll_timeout() {
            let _ = self.timer.start(timeout)?;
            count += 1;
        }

        Ok(count)
    }

    pub fn process_timer_once(
        &mut self,
    ) -> Result<(), RuntimeError<D::Error, MachineError, T::Error>>
    where
        D: ByteDataSource,
    {
        let Some(ticket) = self.timer.poll_timeout().map_err(RuntimeError::Timer)? else {
            return Ok(());
        };

        self.machine
            .handle_timeout(ticket)
            .map_err(RuntimeError::Machine)
    }

    fn process_timer_events(
        &mut self,
    ) -> Result<usize, RuntimeError<D::Error, MachineError, T::Error>>
    where
        D: ByteDataSource,
    {
        let mut count = 0;

        while let Some(ticket) = self.timer.poll_timeout().map_err(RuntimeError::Timer)? {
            self.machine
                .handle_timeout(ticket)
                .map_err(RuntimeError::Machine)?;
            count += 1;
        }

        Ok(count)
    }
}

impl<D, M, T> MessageRuntime<D, M, T>
where
    D: ByteDataSource,
    M: MessageMachine,
    T: RuntimeTimer,
{
    pub fn tick(
        &mut self,
        read_buf: &mut [u8],
    ) -> Result<MessageRuntimeTick, RuntimeError<D::Error, MachineError, T::Error>> {
        let mut report = MessageRuntimeTick::default();

        report.timeout_count += self.process_timer_events()?;
        report.machine_event_count += self.process_machine_events()?;
        report.timeout_request_count += self
            .arm_machine_timeouts_count()
            .map_err(RuntimeError::Timer)?;

        report.read_bytes += self.read_once(read_buf)?;
        report.machine_event_count += self.process_machine_events()?;
        report.timeout_request_count += self
            .arm_machine_timeouts_count()
            .map_err(RuntimeError::Timer)?;

        report.write_count += self.flush_writes_count()?;
        report.machine_event_count += self.process_machine_events()?;
        report.timeout_request_count += self
            .arm_machine_timeouts_count()
            .map_err(RuntimeError::Timer)?;

        Ok(report)
    }

    pub fn run_until_idle(
        &mut self,
        read_buf: &mut [u8],
        max_ticks: usize,
    ) -> Result<MessageRuntimeTick, RuntimeError<D::Error, MachineError, T::Error>> {
        let mut total = MessageRuntimeTick::default();

        for _ in 0..max_ticks {
            let tick = self.tick(read_buf)?;

            total.timeout_count += tick.timeout_count;
            total.machine_event_count += tick.machine_event_count;
            total.read_bytes += tick.read_bytes;
            total.write_count += tick.write_count;
            total.timeout_request_count += tick.timeout_request_count;

            if !tick.did_work() {
                break;
            }
        }

        Ok(total)
    }
}
