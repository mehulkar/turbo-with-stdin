// This is a copy of:
// https://github.com/miquels/tokio-process-pty/blob/5686cfd11539570b3d739555b5f17de184e4fdfd/examples/shell_log.rs
// Copied and some changes to make it work with local crate.

use std::process::exit;
use std::{io, vec};
use termion::raw::IntoRawMode;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::task;
use tokioprocesspty::Command;
mod tokioprocesspty;

#[tokio::main]
async fn main() {
    let child = spawn("pnpm", vec!["-F", "pkg-a", "run", "build"])
        .await
        .unwrap();
    let status = child.await.unwrap();

    println!("process exited with status {:?}", status);
    exit(0);
}

async fn spawn(prog: &str, args: Vec<&str>) -> io::Result<tokioprocesspty::Child> {
    // get terminal size and spawn a shell.
    let (rows, cols) = termion::terminal_size()?;
    let mut child = Command::new(prog)
        .args(args)
        .pty()
        .pty_size(cols, rows)
        .new_session()
        .spawn()?;

    // set the local tty into raw mode.
    let raw_guard = io::stdout().into_raw_mode()?;

    // handles to process stdin/stdout.
    let pty_stdin = child.stdin.take().unwrap();
    let pty_stdout = child.stdout.take().unwrap();

    // copy pty stdout -> tty stdout, and log.
    let from_pty = task::spawn(async move { copy_pty_tty(pty_stdout, io::stdout()).await });

    // copy tty_stdin -> pty_stdin.
    let to_pty = task::spawn(async move { copy_tty_pty(io::stdin(), pty_stdin).await });

    // wait for the first one to finish.
    let _ = futures_util::future::select(from_pty, to_pty).await;
    drop(raw_guard);

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
