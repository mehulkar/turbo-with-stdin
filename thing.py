import os
import pty
import subprocess
import time

# Create a pseudo-terminal pair
master, slave = pty.openpty()

# Fork a child process
pid = os.fork()

print("whathwhathatalkjkl", pid, os.getpid())

if pid == 0:
    # This is the child process (slave side)

    # Close the master side in the child process
    os.close(master)

    # Duplicate the slave side to standard input, output, and error
    os.dup2(slave, 0)  # Redirect standard input
    os.dup2(slave, 1)  # Redirect standard output
    os.dup2(slave, 2)  # Redirect standard error

    # Close the original slave side file descriptor
    os.close(slave)

    # Execute a command (e.g., a shell)
    subprocess.run(["/bin/bash"])
else:
    # This is the parent process (master side)

    # Close the slave side in the master process
    os.close(slave)

    # Write a message to the master side
    os.write(master, b"Hello from the master side!\n")

    # Read the response from the master side
    response = os.read(master, 1000)
    print("Received from the slave side:", response.decode())

    # Close the master side
    os.close(master)
