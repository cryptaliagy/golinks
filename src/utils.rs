use std::time::Duration;

/// Formats a `Duration` as a string.
pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();
    let micros = duration.subsec_micros() - millis * 1000;
    let nanos = duration.subsec_nanos() - micros * 1000;
    if secs > 0 {
        format!("{}.{} s", secs, millis)
    } else if millis > 0 {
        format!("{}.{} ms", millis, micros)
    } else {
        format!("{}.{} μs", micros, nanos)
    }
}
