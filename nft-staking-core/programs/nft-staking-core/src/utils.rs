// Constants - 9AM-5PM UTC (challenge requirement)
pub const SECONDS_IN_AN_HOUR: i64 = 3600;
pub const SECONDS_IN_A_MINUTE: i64 = 60;
pub const SECONDS_IN_A_DAY: i64 = 86400;
pub const TRANSFER_OPEN_TIME: i64 = 9 * SECONDS_IN_AN_HOUR; // 9:00 UTC
pub const TRANSFER_CLOSE_TIME: i64 = 17 * SECONDS_IN_AN_HOUR; // 17:00 UTC (5PM)
pub const MARKET_OPEN_CLOSE_BOUNDARY: i64 = 10 * SECONDS_IN_A_MINUTE; // 10 minutes
pub const REWARD_IN_LAMPORTS: u64 = 10_000_000;

/// Transfers allowed 9AM-5PM UTC
pub fn is_transfer_allowed(unix_timestamp: i64) -> bool {
    let seconds_since_midnight = unix_timestamp % SECONDS_IN_A_DAY;
    seconds_since_midnight >= TRANSFER_OPEN_TIME && seconds_since_midnight < TRANSFER_CLOSE_TIME
}

/// Within 15 min of 9AM or 5PM UTC (for crank reward)
pub fn is_close_to_open_close(unix_timestamp: i64) -> bool {
    let seconds_since_midnight = unix_timestamp % SECONDS_IN_A_DAY;
    (seconds_since_midnight >= TRANSFER_OPEN_TIME
        && seconds_since_midnight < TRANSFER_OPEN_TIME + MARKET_OPEN_CLOSE_BOUNDARY)
        || (seconds_since_midnight >= TRANSFER_CLOSE_TIME
            && seconds_since_midnight < TRANSFER_CLOSE_TIME + MARKET_OPEN_CLOSE_BOUNDARY)
}
