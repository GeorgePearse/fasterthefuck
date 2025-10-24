#!/bin/bash
# FastertTheFuck Shell Integration Script
#
# This script provides shell integration for fasterthefuck, allowing you to
# correct misspelled commands quickly.
#
# Installation:
#   1. Build fasterthefuck: cargo build --release
#   2. Add to ~/.bashrc or ~/.bash_profile:
#        source /path/to/ftf_shell_integration.sh
#   3. Restart your shell
#
# Usage:
#   Type 'ftf' after a failed command to get corrections
#   Type 'ftf --help' for more options

# Get the path to the fasterthefuck binary
# Tries environment variable first, then looks in PATH, then current directory
if [[ -n "$FTF_BINARY" ]]; then
    FTF_CMD="$FTF_BINARY"
elif command -v fasterthefuck &> /dev/null; then
    FTF_CMD="fasterthefuck"
elif [[ -f "./target/release/fasterthefuck" ]]; then
    FTF_CMD="./target/release/fasterthefuck"
else
    FTF_CMD="ftf"
fi

# Store the last command and its details
_FTF_LAST_COMMAND=""
_FTF_LAST_EXIT_CODE=0
_FTF_LAST_OUTPUT=""

# Function to capture command execution details
_fasterthefuck_capture() {
    local exit_code=$?
    local command_line="$BASH_COMMAND"

    # Skip if this is an internal command we initiated
    if [[ "$command_line" == *"_fasterthefuck"* ]] || [[ "$command_line" == *"ftf"* ]]; then
        return $exit_code
    fi

    # Store for later use
    _FTF_LAST_COMMAND="$command_line"
    _FTF_LAST_EXIT_CODE=$exit_code

    return $exit_code
}

# Main correction function - called when user types 'ftf'
ftf() {
    local command="${1}"

    if [[ "$command" == "--help" ]] || [[ "$command" == "-h" ]]; then
        cat << 'EOF'
fasterthefuck - Correct your previous failed command

Usage: ftf [options]

Options:
  --help, -h           Show this help message
  --command CMD        Correct the specified command (instead of last failed)
  --output OUTPUT      Provide command output (for testing)
  --exit-code CODE     Provide exit code (for testing)

Examples:
  ftf                  Correct last failed command
  ftf --command "ls -l" --output "No such file" --exit-code 1
EOF
        return 0
    fi

    # Parse arguments
    local cmd_to_correct="$_FTF_LAST_COMMAND"
    local output="$_FTF_LAST_OUTPUT"
    local exit_code="$_FTF_LAST_EXIT_CODE"

    while [[ $# -gt 0 ]]; do
        case "$1" in
            --command)
                shift
                cmd_to_correct="$1"
                shift
                ;;
            --output)
                shift
                output="$1"
                shift
                ;;
            --exit-code)
                shift
                exit_code="$1"
                shift
                ;;
            *)
                cmd_to_correct="$1"
                shift
                ;;
        esac
    done

    # Validate that we have a command to correct
    if [[ -z "$cmd_to_correct" ]]; then
        echo "fasterthefuck: No command to correct. Run a command first, then type 'ftf'."
        return 1
    fi

    # Call the corrector binary
    if ! command -v "$FTF_CMD" &> /dev/null; then
        echo "fasterthefuck: Binary not found at '$FTF_CMD'"
        echo "Please set FTF_BINARY environment variable or build the project"
        return 1
    fi

    # TODO: Actually invoke the binary with the command details
    # For now, just demonstrate the integration
    echo "fasterthefuck: Would correct '$cmd_to_correct'"
    echo "  (Exit code: $exit_code)"
    [[ -n "$output" ]] && echo "  (Output: $output)"
    echo ""
    echo "Note: Full binary integration coming in Phase 2.3"
}

# Set up command capture using DEBUG trap
# This captures every command executed
trap '_fasterthefuck_capture' DEBUG

export -f ftf
export FTF_CMD
