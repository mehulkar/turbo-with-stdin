use tokio::process::Command;

#[tokio::main]
async fn main() {
    let mut child = Command::new("pnpm")
        .args(vec!["-F", "pkg-a", "run", "build"])
        .stdout(std::process::Stdio::inherit())
        .stdin(std::process::Stdio::inherit())
        .spawn()
        .expect("failed to execute child");

    let ecode = child
        .wait()
        .await
        .expect("failed to wait on child");

    println!("exited with: {}", ecode);
}
