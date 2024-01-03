// This is a copy of:
// https://github.com/miquels/tokio-process-pty/blob/5686cfd11539570b3d739555b5f17de184e4fdfd/examples/shell_log.rs
// Copied and some changes to make it work with local crate.

use std::{
    io::{self, BufRead},
    process::exit,
    sync, thread, vec,
};
// use termion::raw::IntoRawMode;
use tokio::io::{
    AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt,
};
use tokio::task;
use tokioprocesspty::Command;
mod tokioprocesspty;
use futures::{stream::FuturesUnordered, StreamExt};

#[tokio::main]
async fn main() {
    let mut tasks = FuturesUnordered::new();

    tasks.push(tokio::spawn(async {
        let prog = "pnpm";
        let args = vec!["-F", "pkg-a", "run", "build"];
        let child = spawn("pkg-a#build", prog, args.clone()).await.unwrap();
        let status = child.await.unwrap();
        println!("{} {:?} exited with status {:?}", prog, args, status);
    }));

    // Second spawn
    tasks.push(tokio::spawn(async {
        let prog = "pnpm";
        let args = vec!["-F", "pkg-b", "run", "build"];
        let child = spawn("pkg-b#build", prog, args.clone()).await.unwrap();
        let status = child.await.unwrap();
        println!("{} {:?} exited with status {:?}", prog, args, status);
    }));

    while let Some(result) = tasks.next().await {
        result.expect("task executor panicked");
    }

    exit(0);
}

async fn spawn(task: &str, prog: &str, args: Vec<&str>) -> io::Result<tokioprocesspty::Child> {
    println!("Spawning {} with {} and args {:?}", task, prog, args);
    let (sender, receiver) = sync::mpsc::sync_channel::<(
        tokioprocesspty::ChildStdin,
        tokioprocesspty::ChildStdout,
        &str,
    )>(1);

    thread::spawn(async || {
        let mut parent_stdin_handle = io::stdin().lock();
        let mut parent_stdout_handle = io::stdout().lock();

        let (mut child_stdin, child_stdout, task_id) = match receiver.recv() {
            Ok(stdin) => stdin,
            // Exit no message from child
            Err(_) => return,
        };

        println!("Granting stdin lock to {}", task_id);

        let mut buffer = String::new();
        let _ = parent_stdin_handle.read_line(&mut buffer);

        // let raw_guard = io::stdout().into_raw_mode().expect("failed raw mode");
        // let from_pty =
        //     task::spawn(async move { copy_pty_tty(child_stdout, parent_stdout_handle).await });

        copy_tty_pty(parent_stdin_handle, child_stdin);

        // drop(raw_guard);
    });

    let (rows, cols) = termion::terminal_size()?;
    let mut child = Command::new(prog)
        .args(args)
        .pty()
        .pty_size(cols, rows)
        .new_session()
        .spawn()?;

    let stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let _ = sender.send((stdin, stdout, task.clone()));

    Ok(child)
}

// copy AsyncRead -> Write.
async fn copy_pty_tty<R, W>(mut from: R, mut to: W) -> io::Result<()>
where
    R: AsyncRead + Unpin,
    W: io::Write + Send + 'static,
{
    let mut buffer = [0u8; 1000];
    loop {
        let n = from.read(&mut buffer[..]).await?;
        if n == 0 {
            break;
        }
        // tokio doesn't have async-write to stdout, so use block-in-place.
        task::block_in_place(|| {
            to.write_all(&buffer[0..n])?;
            to.flush()
        })?;
    }
    Ok(())
}

// copy Read -> AsyncWrite.
async fn copy_tty_pty<R, W>(mut from: R, mut to: W) -> io::Result<()>
where
    R: io::Read + Send + 'static,
    W: AsyncWrite + Unpin,
{
    loop {
        let mut buffer = [0u8; 1000];
        // tokio doesn't have async-read from stdin, so use block-in-place.
        let n = task::block_in_place(|| from.read(&mut buffer[..]))?;
        if n == 0 {
            break;
        }
        to.write_all(&buffer[..n]).await?;
    }
    Ok(())
}
