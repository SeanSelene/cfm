fn main() {
    if let Err(e) = cfm::run() {
        eprintln!("错误: {}", e);
        std::process::exit(1);
    }
}
