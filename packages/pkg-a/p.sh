#!/bin/bash

function progressBar {
    local iteration=$1
    local total=$2
    local length=30

    # Calculate percentage and length of filled bar
    local percent=$((iteration * 100 / total))
    local filledLength=$((length * iteration / total))

    # Print the progress bar
    printf "\rProgress: ["
    printf "%-${filledLength}s" "#" | tr ' ' '='
    printf "%-$((length - filledLength))s" "] $percent%%"

    # Move the cursor to the beginning of the line
    printf "\r"
}

# Example usage
totalIterations=100

for ((i = 1; i <= totalIterations; i++)); do
    # Simulate some task that takes time
    sleep 0.01
    progressBar $i $totalIterations
done

# Move to the next line when complete
printf "\n"
