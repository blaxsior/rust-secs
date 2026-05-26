use core::time::Duration;
use sans_io_time::Instant;

/// 시간에 따른 timer 만료 여부를 판단하기 위한 구조체
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickTimer {
    limit: Duration,
    start_time: Option<Instant>,
}

impl TickTimer {
    pub const fn new(limit: Duration) -> Self {
        Self {
            limit,
            start_time: None,
        }
    }

    /// 타이머를 시작한다.
    pub fn start(&mut self, now: Instant) {
        self.start_time = Some(now);
    }

    /// 타이머를 정지 및 초기화한다.
    pub fn reset(&mut self) {
        self.start_time = None;
    }

    /// 현재 타이머 활성화 여부를 반환한다.
    pub fn is_active(&self) -> bool {
        self.start_time.is_some()
    }

    /// time을 주입, 현재 타이머의 만료 여부를 체크한다.
    pub fn check_timeout(&mut self, now: Instant) -> bool {
        // timer 활성화되어 있지 않다면 false
        // 만약 start_time이 Some이면 start 변수에 담아서 로직 수행
        let Some(start) = self.start_time else {
            return false;
        };

        // 경과 시간 계산
        let elapsed = now - start;

        if elapsed >= self.limit {
            self.reset(); // 만료 시 자동 리셋
            true // 만료 신호 반환
        } else {
            false
        }
    }
}

/// 현재 시간을 제공하는 trait
pub trait TimeProvider {
    fn get_now(&self) -> Instant;
}

// pub struct SystemTimeProvider {}
// impl TimeProvider for SystemTimeProvider {
//     fn get_now(&self) -> Instant {
//         Instant::from_nanos(0)
//     }
// }
