use smart_default::SmartDefault;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(SmartDefault)]
pub struct Snowflake {
    #[default(Self::EPOCH_2021)]
    pub epoch: i64,
    pub worker_id: i64,
    pub datacenter_id: i64,
    id: AtomicI64,
}

impl Snowflake {
    pub const EPOCH_2021: i64 = 1_627_588_000_000;
    pub const EPOCH_2022: i64 = 1_640_995_200_000;

    const TIME_SHIFT: i32 = 22;
    const WID_SHIFT: i32 = 17;
    const DID_SHIFT: i32 = 12;
    const SEQ_BITS: i32 = Self::DID_SHIFT;

    /// Get milliseconds duration since the Unix Epoch.
    pub fn now_millis() -> i64 {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(v) => v.as_millis() as i64,
            _ => 0,
        }
    }

    pub fn new(epoch: i64, worker_id: i64, datacenter_id: i64) -> Self {
        Self {
            epoch,
            worker_id,
            datacenter_id,
            id: AtomicI64::new(0),
        }
    }

    /// Get the milliseconds duration since this epoch.
    #[inline]
    pub fn millis_since_epoch(&self) -> i64 {
        Self::now_millis() - self.epoch
    }

    #[inline]
    pub fn generate(&self) -> i64 {
        self.generate_with_time(self.millis_since_epoch())
    }

    pub fn generate_with(&self, time_millis: i64, seq: i64) -> i64 {
        (time_millis << Self::TIME_SHIFT)
            | (self.worker_id << Self::WID_SHIFT)
            | (self.datacenter_id << Self::DID_SHIFT)
            | seq
    }

    pub fn generate_with_time(&self, time_millis: i64) -> i64 {
        let mut ts = time_millis;
        loop {
            // Get a copy.
            let last_id = self.id.load(Ordering::Acquire);
            // Parse timestamp
            let last_ts = last_id >> Self::TIME_SHIFT;
            // Parse sequence and increase 1.
            let mut seq = last_id.wrapping_add(1) & ((1i64 << Self::SEQ_BITS) - 1);
            if ts <= last_ts {
                ts = last_ts;
                if seq == 0 {
                    ts += 1;
                }
            } else {
                seq = 0;
            }

            let id = (ts << Self::TIME_SHIFT)
                | (self.worker_id << Self::WID_SHIFT)
                | (self.datacenter_id << Self::DID_SHIFT)
                | seq;
            if self
                .id
                .compare_exchange(last_id, id, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                break id;
            }
        }
    }
}
