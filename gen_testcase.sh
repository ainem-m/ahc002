python3 gen_seeds_txt.py >tools/seeds.txt 
cd tools/
cargo run --release --bin gen seeds.txt
