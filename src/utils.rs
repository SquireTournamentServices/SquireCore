pub fn try_into_bool(s: &str) -> Option<bool> {
    let s = s.to_lowercase();
    match s.as_str() {
        "1" | "t" | "true" => Some(true),
        "0" | "f" | "false" => Some(false),
        _ => None,
    }
}
