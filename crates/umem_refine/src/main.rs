use umem_refine::Refine;

fn main() {
    let t = Refine::process("hey there, how are you doing").unwrap();
    print!("{:?}", t);
}
