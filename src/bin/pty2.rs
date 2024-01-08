use pty::fork::*;
use std::io::Write;
use std::sync::Arc;
use tokio::{
    io::{self, AsyncReadExt},
    sync::Mutex,
    task::JoinHandle,
};

#[tokio::main]
async fn main() {
    let stdin_lock = Arc::new(Mutex::new(io::stdin()));

    // Function to handle child process and its IO
    async fn handle_child_process(stdin_lock: Arc<Mutex<io::Stdin>>, command: &str, args: &[&str]) {
        let fork = Fork::from_ptmx().unwrap();

        if let Some(mut master) = fork.is_parent().ok() {
            let mut stdin = stdin_lock.lock().await;
            let mut input = vec![0; 1024];
            let mut output = vec![0; 1024];

            loop {
                tokio::select! {
                    n = stdin.read(&mut input) => {
                        let n = n.unwrap();
                        if n == 0 { break; } // Exit on EOF

                        // Write to the child's pty
                        master.write_all(&input[..n]).unwrap();
                    },
                    n = master.read(&mut output) => {
                        let n = n.unwrap();
                        if n > 0 {
                            // Output from the child process
                            println!("Output from child: {}", String::from_utf8_lossy(&output[..n]));
                        }
                    }
                }
            }
        } else {
            // Child process executes the command
            std::process::Command::new(command)
                .args(args)
                .status()
                .unwrap();
        }
    }

    // Spawn first child process
    let child1 = tokio::spawn(handle_child_process(
        stdin_lock.clone(),
        "pnpm",
        &["-F", "pkg-a", "run", "build"],
    ));

    // Wait for the first child to finish
    child1.await.unwrap();

    // Spawn second child process with different command
    let child2 = tokio::spawn(handle_child_process(
        stdin_lock,
        "pnpm",
        &["-F", "pkg-b", "run", "build"],
    )); // Replace with your second command and arguments

    // Wait for the second child to finish
    child2.await.unwrap();
}
