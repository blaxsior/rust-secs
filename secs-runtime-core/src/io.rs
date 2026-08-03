use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ByteDataSourceError {
    /// 열려 있지 않은 source에 read/write/close를 요청함
    #[error("data source is not open")]
    NotOpen,
    /// 연결 시도 실패
    #[error("failed to open data source")]
    OpenFailed,
    /// 연결 종료 시도 실패
    #[error("failed to close data source")]
    CloseFailed,
    /// 상대방이 정상적으로 연결을 종료했거나, 장치가 분리됨
    #[error("data source disconnected")]
    Disconnected,
    /// read/write가 일시적으로 준비되지 않음
    #[error("data source is not ready")]
    WouldBlock,
    /// 설정된 I/O 대기 시간이 초과됨
    #[error("data source I/O timed out")]
    TimedOut,
    /// read 동작 실패
    #[error("failed to read from data source")]
    ReadFailed,
    /// write 동작 실패
    #[error("failed to write to data source")]
    WriteFailed,
}

impl ByteDataSourceError {
    pub fn is_disconnected(self) -> bool {
        matches!(self, Self::Disconnected)
    }

    pub fn is_temporary(self) -> bool {
        matches!(self, Self::WouldBlock | Self::TimedOut)
    }

    pub fn is_read_error(self) -> bool {
        matches!(
            self,
            Self::NotOpen
                | Self::Disconnected
                | Self::WouldBlock
                | Self::TimedOut
                | Self::ReadFailed
        )
    }

    pub fn is_write_error(self) -> bool {
        matches!(
            self,
            Self::NotOpen
                | Self::Disconnected
                | Self::WouldBlock
                | Self::TimedOut
                | Self::WriteFailed
        )
    }
}

pub trait ByteDataSource {
    fn open(&mut self) -> Result<(), ByteDataSourceError>;

    fn close(&mut self) -> Result<(), ByteDataSourceError>;

    fn is_open(&self) -> bool;

    /// Reads bytes without blocking the runtime tick indefinitely.
    ///
    /// Implementations should prefer non-blocking I/O and return `WouldBlock` when no data is
    /// currently available. If the underlying API only supports blocking I/O, it must use a short
    /// timeout and return `TimedOut` instead of waiting forever.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ByteDataSourceError>;

    /// Writes bytes without blocking the runtime tick indefinitely.
    ///
    /// Implementations should prefer non-blocking I/O and return `WouldBlock` when the output side
    /// is temporarily unavailable. If the underlying API only supports blocking I/O, it must use a
    /// short timeout and return `TimedOut` instead of waiting forever.
    fn write(&mut self, bytes: &[u8]) -> Result<(), ByteDataSourceError>;
}
