
pub fn unwrap_or_warn<T>(value: Option<T>, default: T, msg: &str) -> T {
    value.unwrap_or_else(|| {
        eprintln!("warn: {}", msg);
        default
    })
}
