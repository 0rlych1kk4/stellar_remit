pub fn is_valid_address(addr: &str) -> bool {
    addr.starts_with("G") && addr.len() == 56
}

