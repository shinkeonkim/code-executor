#!/bin/bash
set -e

# Write user code to file
if [ -z "$USER_CODE" ]; then
  echo "No user code provided!" >&2
  exit 1
fi

CODE_FILE="/tmp/${EXECUTION_ID}_user_code.cpp"
BIN_FILE="/tmp/${EXECUTION_ID}_main"
STDOUT_FILE="/tmp/${EXECUTION_ID}_stdout.txt"
STDERR_FILE="/tmp/${EXECUTION_ID}_stderr.txt"
TIME_FILE="/tmp/${EXECUTION_ID}_time.txt"

echo "$USER_CODE" > "$CODE_FILE"

# Compile with C++23
if ! g++ -std=c++23 -O2 -o "$BIN_FILE" "$CODE_FILE"; then
  echo "Compilation failed" >&2
  exit 2
fi

# Run with time and timeout
if ! /usr/bin/time -v -o "$TIME_FILE" timeout ${TIMEOUT:-10} "$BIN_FILE" > "$STDOUT_FILE" 2> "$STDERR_FILE"; then
  # If runtime error, still print outputs
  :
fi

cat "$STDOUT_FILE"
cat "$STDERR_FILE" 1>&2
echo "===CODE_EXEC_TIME_BEGIN===" 1>&2
cat "$TIME_FILE" 1>&2
echo "===CODE_EXEC_TIME_END===" 1>&2
