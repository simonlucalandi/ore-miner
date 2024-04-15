cargo run --release -- \
    --rpc $RPC_URL \
    --priority-fee 100000 \
    bundle-mine \
    --threads 8 \
    --key-folder ../wallets \
    --max-adaptive-tip 400000


