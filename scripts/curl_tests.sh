#!/usr/bin/env bash
# Basic curl smoke tests for the file server.
# Assumes the server is already running (cargo run) on localhost:8080.
set -uo pipefail

BASE="http://localhost:8080"
TMP_FILE="$(mktemp)"
echo "hello from curl_tests.sh" > "$TMP_FILE"

pass=0
fail=0

check() {
    local desc="$1" expected="$2" actual="$3"
    if [ "$expected" = "$actual" ]; then
        echo "PASS: $desc"
        pass=$((pass + 1))
    else
        echo "FAIL: $desc (expected $expected, got $actual)"
        fail=$((fail + 1))
    fi
}

echo "--- upload: happy path ---"
upload_body="$(curl -s -X POST "$BASE/api/upload" -F "file=@${TMP_FILE};filename=test.txt")"
upload_status="$(curl -s -o /dev/null -w '%{http_code}' -X POST "$BASE/api/upload" -F "file=@${TMP_FILE};filename=test.txt")"
check "upload returns 200" "200" "$upload_status"
echo "$upload_body"
saved_name="$(echo "$upload_body" | grep -o '"filename":"[^"]*"' | cut -d'"' -f4)"

echo "--- list: uploaded file is present ---"
list_body="$(curl -s "$BASE/api/list")"
if echo "$list_body" | grep -q "$saved_name"; then
    echo "PASS: list contains uploaded file"
    pass=$((pass + 1))
else
    echo "FAIL: list does not contain uploaded file"
    fail=$((fail + 1))
fi

echo "--- download: happy path ---"
download_status="$(curl -s -o /dev/null -w '%{http_code}' "$BASE/api/download/${saved_name}")"
check "download of existing file returns 200" "200" "$download_status"

echo "--- download: missing file returns 404 ---"
missing_status="$(curl -s -o /dev/null -w '%{http_code}' "$BASE/api/download/does-not-exist.txt")"
check "download of missing file returns 404" "404" "$missing_status"

echo "--- download: path traversal rejected ---"
traversal_status="$(curl -s -o /dev/null -w '%{http_code}' "$BASE/api/download/..%2F..%2Fetc%2Fpasswd")"
check "download traversal returns 400" "400" "$traversal_status"

echo "--- upload: path traversal filename rejected ---"
upload_traversal_status="$(curl -s -o /dev/null -w '%{http_code}' -X POST "$BASE/api/upload" -F "file=@${TMP_FILE};filename=../../evil")"
check "upload traversal filename returns 400" "400" "$upload_traversal_status"

rm -f "$TMP_FILE"

echo
echo "$pass passed, $fail failed"
[ "$fail" -eq 0 ]
