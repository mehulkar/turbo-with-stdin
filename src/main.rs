use libc;
use pty;
use pty_process;
// use std::ffi::CString;
// use std::io::{stdin, Read};
// use std::process::{Command, Stdio};
// use tokio::process:Command

// use pty::fork::*;

fn main() {
    // Create pty
    let pty = pty_process::Pty::new().expect("create");

    // size of new terminal
    pty.resize(pty_process::Size::new(24, 80)).expect("resize");

    let mut cmd = pty_process::Command::new("nethack");

    let xx = &pty.pts().unwrap();

    let child = cmd.spawn(xx);
}

// fn main() {
//     let fork = Fork::from_ptmx().unwrap();

//     if let Some(mut master) = fork.is_parent().ok() {
//         // Read output via PTY master
//         let mut output = String::new();

//         match master.read_to_string(&mut output) {
//             Ok(_nread) => println!("{}", output.trim()),
//             Err(e) => panic!("read error: {}", e),
//         }
//     } else {
//         // Child process just exec `tty`
//         Command::new("npm")
//             .args(vec!["run", "build-a"])
//             .status()
//             .expect("could not execute tty");
//     }
// }

// use std::io::Read;
// use std::process::{Command, Stdio};

// fn run_npm_test() {
//     // Configure the command to run "npm run test"
//     let mut command = Command::new("npm");
//     command.arg("run").arg("test");

//     // Set up to capture the stdout
//     command.stdout(Stdio::piped());
//     command.stdin(Stdio::inherit());

//     // Spawn the process
//     let mut child = command.spawn().expect("Failed to start npm run test");

//     // Wait for the child process to finish
//     let status = child.wait().expect("Failed to wait for process");

//     // Check if the process was successful
//     if status.success() {
//         // Read the stdout and print it
//         if let Some(mut stdout) = child.stdout.take() {
//             let mut output = String::new();
//             stdout
//                 .read_to_string(&mut output)
//                 .expect("Failed to read stdout");
//             println!("npm run test output:\n{}", output);
//         } else {
//             println!("No stdout captured");
//         }
//     } else {
//         println!("npm run test failed with: {:?}", status);
//     }
// }

// fn main() {
//     run_npm_test();
// }
