fn main() {
    let output_res = std::process::Command::new("/nonexistent").arg("-v").output().ok();
    println!("{:?}", output_res);
}
