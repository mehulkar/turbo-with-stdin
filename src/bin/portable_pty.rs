use futures::{stream::FuturesUnordered, StreamExt};
use portable_pty::{Child, CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::{
    io::{self, BufRead, Write},
    process::exit,
    thread, vec,
};
use tokio::{io::AsyncRead, task};

#[tokio::main]
async fn main() {
    let mut tasks = FuturesUnordered::new();
    tasks.push(tokio::spawn(async {
        let prog = "pnpm";
        let args = vec!["-F", "pkg-a", "run", "build"];
        let mut child = spawn("pkg-a#build", prog, args.clone()).await.unwrap();
        let status = child.wait().unwrap();
        println!("{prog} {:?} exited {:?}", args, status);
    }));

    // Second spawn
    tasks.push(tokio::spawn(async {
        let prog = "pnpm";
        let args = vec!["-F", "pkg-b", "run", "build"];
        let mut child = spawn("pkg-b#build", prog, args.clone()).await.unwrap();
        let status = child.wait().unwrap();
        println!("{} {:?} exited {:?}", prog, args, status);
    }));

    while let Some(result) = tasks.next().await {
        result.expect("task executor panicked");
    }

    exit(0);
}

async fn spawn(
    task: &str,
    prog: &str,
    args: Vec<&str>,
) -> Result<Box<dyn Child + Send + Sync>, anyhow::Error> {
    // task_id is for debugging
    let task_id = task.to_string();

    let pty_system = NativePtySystem::default();

    // Build a command to run
    let mut cmd = CommandBuilder::new(prog);
    cmd.args(args);

    // create a pty pair.
    // - The "master" side connects to the parent process
    // - The "slave" side connects to the child process
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();

    let child = pair.slave.spawn_command(cmd).map_err(anyhow::Error::msg); // the child process
    let mut reader = pair.master.try_clone_reader().unwrap(); // we will read child stdout from this
    let mut writer = pair.master.take_writer().unwrap(); // we will write io::stdin to this

    // we can drop both of these (TODO: not sure if necessary)
    drop(pair.slave);
    drop(pair.master);

    // setup channels to send bytes to and from the pty
    let (in_sender, mut in_receiver) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
    let (out_sender, mut out_receiver) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

    // stdin shenanigans (copy from io::stdin and write to pty)
    thread::spawn(move || {
        let mut handle = io::stdin().lock();
        println!("{task_id} has stdin lock");
        let mut s = String::new();
        let _ = handle.read_line(&mut s);
        in_sender.send(s.into_bytes()).unwrap();
    });
    tokio::spawn(async move {
        match in_receiver.recv().await {
            Some(bytes) => {
                writer.write_all(&bytes).unwrap();
            }
            None => println!("No bytes received"),
        }
    });

    // stdout shenanigans (copy from pty and write to io::stdout)
    tokio::spawn(async move {
        let mut s = String::new();
        reader.read_to_string(&mut s).unwrap();
        out_sender.send(s.into_bytes()).unwrap();
    });

    tokio::spawn(async move {
        match out_receiver.recv().await {
            Some(bytes) => {
                io::stdout().write_all(&bytes).unwrap();
            }
            None => println!("No bytes received"),
        }
    });

    return child;
}
