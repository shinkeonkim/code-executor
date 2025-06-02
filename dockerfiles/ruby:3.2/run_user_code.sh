#!/bin/bash
set -e
CODE_FILE="/tmp/${EXECUTION_ID}_user_code.rb"
STDOUT_FILE="/tmp/${EXECUTION_ID}_stdout.txt"
STDERR_FILE="/tmp/${EXECUTION_ID}_stderr.txt"
TIME_FILE="/tmp/${EXECUTION_ID}_time.txt"
echo "$USER_CODE" > "$CODE_FILE"
/usr/bin/time -v -o "$TIME_FILE" ruby "$CODE_FILE" > "$STDOUT_FILE" 2> "$STDERR_FILE"
cat "$STDOUT_FILE"
cat "$STDERR_FILE" 1>&2
echo "===CODE_EXEC_TIME_BEGIN===" 1>&2
cat "$TIME_FILE" 1>&2
echo "===CODE_EXEC_TIME_END===" 1>&2
