1. Install build dependencies
sudo apt install build-essential

2. Install Rust
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh

3. Reload $PATH
source $HOME/.cargo/env

4. You already have this repo in your local drive, now get into the folder that contains this README file

5. Restore some file system setup before running the test
mkdir test_data/dir0_0_inaccessible
chmod a-rwx test_data/dir0_0_inaccessible
mkdir test_data/dir0_1_empty
chmod a-rwx test_data/dir0_2/file1_0_inaccessible.txt

6. Run the unit tests
cargo test

7. Use the program, e.g.,
cargo run 'test_data' ''
cargo run 'test_data' 'wo rd'
