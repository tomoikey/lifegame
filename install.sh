cargo build --release

BASE_NAME=$(basename "$(pwd)")
cp target/release/"$BASE_NAME" /usr/local/bin/"$BASE_NAME"