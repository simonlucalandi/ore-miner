export CUDA_VISIBLE_DEVICES=0

cargo run --release -- \
    --rpc $RPC_URL \
    --priority-fee 100000 \
    bundle-mine-gpu \
    --key-folder ../wallets \
    --max-adaptive-tip 400000


