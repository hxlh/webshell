set -e

rm -rf dist
mkdir dist

cargo build --release
cp target/release/webshell dist/
cp -r web dist/

echo "done"