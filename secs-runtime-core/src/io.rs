#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteDataSourceError {
    /// 이미 열려 있는 source에 open을 요청함
    AlreadyOpen,
    /// 열려 있지 않은 source에 read/write/close를 요청함
    NotOpen,
    /// 연결 시도 실패
    OpenFailed,
    /// 연결 종료 시도 실패
    CloseFailed,
    /// 상대방이 정상적으로 연결을 종료했거나, 장치가 분리됨
    Disconnected,
    /// read/write가 일시적으로 준비되지 않음
    WouldBlock,
    /// 설정된 I/O 대기 시간이 초과됨
    TimedOut,
    /// read 동작 실패
    ReadFailed,
    /// write 동작 실패
    WriteFailed,
    /// 출력 버퍼가 가득 차서 현재 write 불가
    WriteBufferFull,
    /// 입력 버퍼가 부족해서 현재 read 불가
    ReadBufferTooSmall,
    /// 구현체 고유 에러
    SourceSpecific(u16),
}

impl ByteDataSourceError {
    pub fn is_disconnected(self) -> bool {
        matches!(self, Self::Disconnected)
    }

    pub fn is_temporary(self) -> bool {
        matches!(
            self,
            Self::WouldBlock | Self::TimedOut | Self::WriteBufferFull
        )
    }

    pub fn is_read_error(self) -> bool {
        matches!(
            self,
            Self::NotOpen
                | Self::Disconnected
                | Self::WouldBlock
                | Self::TimedOut
                | Self::ReadFailed
                | Self::ReadBufferTooSmall
                | Self::SourceSpecific(_)
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
                | Self::WriteBufferFull
                | Self::SourceSpecific(_)
        )
    }
}

pub trait ByteDataSource {
    fn open(&mut self) -> Result<(), ByteDataSourceError>;

    fn close(&mut self) -> Result<(), ByteDataSourceError>;

    fn is_open(&self) -> bool;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ByteDataSourceError>;

    fn write(&mut self, bytes: &[u8]) -> Result<(), ByteDataSourceError>;
}
