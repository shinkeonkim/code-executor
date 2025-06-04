#!/bin/bash
set -e

if [ -z "$USER_CODE" ]; then
  echo "No user code provided!" >&2
  exit 1
fi

WORK_DIR="/tmp/${EXECUTION_ID}_java"
CODE_FILE="$WORK_DIR/Main.java"
CLASS_DIR="$WORK_DIR/classes"
STDOUT_FILE="$WORK_DIR/stdout.txt"
STDERR_FILE="$WORK_DIR/stderr.txt"
TIME_FILE="$WORK_DIR/time.txt"

mkdir -p "$CLASS_DIR"
mkdir -p "$WORK_DIR"
echo "$USER_CODE" > "$CODE_FILE"

# Compile
if ! javac -d "$CLASS_DIR" "$CODE_FILE"; then
  echo "Compilation failed" >&2
  exit 2
fi

# Run with time and timeout (assume main class is always Main)
if ! /usr/bin/time -v -o "$TIME_FILE" timeout ${TIMEOUT:-10} java -cp "$CLASS_DIR" Main > "$STDOUT_FILE" 2> "$STDERR_FILE"; then
  # If runtime error, still print outputs
  :
fi

cat "$STDOUT_FILE"
cat "$STDERR_FILE" 1>&2
echo "===CODE_EXEC_TIME_BEGIN===" 1>&2
cat "$TIME_FILE" 1>&2
echo "===CODE_EXEC_TIME_END===" 1>&2 