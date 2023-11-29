#!/bin/sh

# Run tests for Linux with HTTP, AES and EKE
env \
    callback_port='80' \
    callback_jitter='0' \
    post_uri='data' \
    callback_interval='1000' \
    killdate='1709510400' \
    headers='[{"name": "User-Agent", "key": "User-Agent", "value": "Mozilla/5.0 (Windows NT 6.3; Trident/7.0; rv:11.0) like Gecko"}]' \
    callback_hosts='http://mythic.local.vm' \
    query_path_name='q' \
    encrypted_exchange_check='T' \
    get_uri='index' \
    UUID='51057f63-e654-4d4e-bafa-4652afa2c769' \
    AESKEY='6qPbbhTdLJ5KFvgNIukaDnwR6cWtwLsBPNDGyVuDYJU=' \
    connection_retries='1' \
    working_hours='0-1439' \
    cargo +nightly test -F http,AES,EKE,pwd --target x86_64-unknown-linux-gnu --workspace --exclude profile-tcp || exit 1

# Run tests for Linux with HTTP and no AES or EKE
env \
    callback_port='80' \
    callback_jitter='0' \
    post_uri='data' \
    callback_interval='1000' \
    killdate='1709510400' \
    headers='[{"name": "User-Agent", "key": "User-Agent", "value": "Mozilla/5.0 (Windows NT 6.3; Trident/7.0; rv:11.0) like Gecko"}]' \
    callback_hosts='http://mythic.local.vm' \
    query_path_name='q' \
    encrypted_exchange_check='T' \
    get_uri='index' \
    UUID='51057f63-e654-4d4e-bafa-4652afa2c769' \
    AESKEY='6qPbbhTdLJ5KFvgNIukaDnwR6cWtwLsBPNDGyVuDYJU=' \
    connection_retries='1' \
    working_hours='0-1439' \
    cargo +nightly test --color auto -F http,profile-http?/http --target x86_64-unknown-linux-gnu || exit 1
