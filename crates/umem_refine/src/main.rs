use umem_refine::Segmenter;

fn main() {
    let t = Segmenter::process("hey there, how are you doing").unwrap();
    print!("{:?}", t);
}
