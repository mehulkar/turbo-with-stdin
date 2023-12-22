import os
import pty
import subprocess
import time

# Create a pseudo-terminal pair
principle, follower = pty.openpty()

# Fork a child process
pid = os.fork()

print("whathwhathatalkjkl", pid, os.getpid())

if pid == 0:
    # This is the child process (follower side)
    # Close the principle side in the child process
    os.close(principle)
    # Duplicate the follower side to standard input, output, and error
    os.dup2(follower, 0)  # Redirect standard input
    os.dup2(follower, 1)  # Redirect standard output
    os.dup2(follower, 2)  # Redirect standard error
    # Close the original follower side file descriptor
    os.close(follower)
    # Execute a command (e.g., a shell)
    subprocess.run(["/bin/bash"])
else:
    # This is the parent process (principle side)
    # Close the follower side in the principle process
    os.close(follower)
    # Write a message to the principle side
    os.write(principle, b"Hello from the principle side!\n")
    # Read the response from the principle side
    response = os.read(principle, 1000)
    print("Follower side:", response.decode())
    # Close the principle side
    os.close(principle)
