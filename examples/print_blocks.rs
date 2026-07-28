use cellvm::bytecode;
use cellvm::module::Function;
use cellvm::verifier::typeflow::TypeFlow;

fn blocks(code: &[u8]) -> Vec<(Vec<&'static str>, Vec<usize>)> {
    #[derive(Clone)]
    enum Op { Int, Ref, Get }
    struct Block { ops: Vec<Op>, succs: Vec<usize> }
    let mut blocks = vec![Block { ops: vec![], succs: vec![1] }];
    let mut cur = Block { ops: vec![], succs: vec![] };
    let mut pc = 0usize;
    while pc < code.len() {
        let op = code[pc];
        match op {
            x if x == bytecode::PUSH8 => { cur.ops.push(Op::Int); pc += 2; }
            x if x == bytecode::NEWARR => { cur.ops.push(Op::Ref); pc += 4; }
            x if x == bytecode::GETFIELD => { cur.ops.push(Op::Get); pc += 4; }
            x if x == bytecode::SJMP => {
                blocks.push(cur);
                cur = Block { ops: vec![], succs: vec![0] };
                pc += 2;
            }
            _ => {
                if let Some(len) = bytecode::instr_len(op) { pc += len; } else { break; }
            }
        }
    }
    if !cur.ops.is_empty() { blocks.push(cur); }
    blocks.into_iter().enumerate().map(|(i, b)| {
        let ops: Vec<_> = b.ops.iter().map(|o| match o {
            Op::Int => "Int",
            Op::Ref => "Ref",
            Op::Get => "Get",
        }).collect();
        (ops, b.succs)
    }).collect()
}

fn main() {
    let code = [
        0x20, 0x0A, 0x40, 0, 0, 0, 0x41, 0, 0, 0, 0x20, 0x02,
        0x01, 0x00, 0x20, 0xF2, 0xFF,
    ];
    for (i, (ops, succs)) in blocks(&code).into_iter().enumerate() {
        println!("block {i}: ops={ops:?} succs={succs:?}");
    }
    let f = Function {
        code: code.to_vec(),
        max_locals: 4,
        frame_size: 16,
        consts: vec![],
        string_pool: vec![],
        upvalues: vec![],
        name: "t".into(),
    };
    println!("verify={:?}", TypeFlow::verify_function(&f));
}
