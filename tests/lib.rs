extern crate skuld;

use skuld::{generator, parser, tokenizer};

#[test]
fn ret() {
    do_test("ret", "c3");
}

#[test]
fn push() {
    do_test("push 1", "6a 01");
    do_test("push rax", "50");
    do_test("push r8", "41 50");
}

#[test]
fn pop() {
    do_test("pop rax", "58");
    do_test("pop r8", "41 58");
}

#[test]
fn idiv() {
    do_test("idiv eax", "f7 f8");
    do_test("idiv rax", "48 f7 f8");
    do_test("idiv r8", "49 f7 f8");
}

#[test]
fn jmp() {
    do_test("label: jmp label", "e9 fb ff ff ff");
}

#[test]
fn je() {
    do_test("label: je label", "0f 84 fa ff ff ff");
}

#[test]
fn call() {
    do_test("label: call label", "e8 fb ff ff ff");
}

#[test]
fn sete() {
    do_test("sete al", "0f 94 c0");
    do_test("sete r9b", "41 0f 94 c1");
}

#[test]
fn setne() {
    do_test("setne al", "0f 95 c0");
    do_test("setne r9b", "41 0f 95 c1");
}

#[test]
fn setl() {
    do_test("setl al", "0f 9c c0");
    do_test("setl r9b", "41 0f 9c c1");
}

#[test]
fn setle() {
    do_test("setle al", "0f 9e c0");
    do_test("setle r9b", "41 0f 9e c1");
}

#[test]
fn setg() {
    do_test("setg al", "0f 9f c0");
    do_test("setg r9b", "41 0f 9f c1");
}

#[test]
fn setge() {
    do_test("setge al", "0f 9d c0");
    do_test("setge r9b", "41 0f 9d c1");
}

#[test]
fn add() {
    do_test("add eax,1", "83 c0 01");
    do_test("add rax,1", "48 83 c0 01");
    do_test("add r9,1", "49 83 c1 01");
    do_test("add eax,eax", "01 c0");
    do_test("add rax,rax", "48 01 c0");
    do_test("add rax,r9", "4c 01 c8");
    do_test("add r9,rax", "49 01 c1");
    do_test("add r9,r9", "4d 01 c9");
}

#[test]
fn sub() {
    do_test("sub eax,1", "83 e8 01");
    do_test("sub rax,1", "48 83 e8 01");
    do_test("sub r9,1", "49 83 e9 01");
    do_test("sub eax,eax", "29 c0");
    do_test("sub rax,rax", "48 29 c0");
    do_test("sub rax,r9", "4c 29 c8");
    do_test("sub r9,rax", "49 29 c1");
    do_test("sub r9,r9", "4d 29 c9");
}

#[test]
fn imul() {
    do_test("imul eax,eax", "0f af c0");
    do_test("imul rax,rax", "48 0f af c0");
    do_test("imul rax,r9", "49 0f af c1");
    do_test("imul r9,rax", "4c 0f af c8");
    do_test("imul r9,r9", "4d 0f af c9");
}

#[test]
fn xor() {
    do_test("xor eax,1", "83 f0 01");
    do_test("xor rax,1", "48 83 f0 01");
    do_test("xor r9,1", "49 83 f1 01");
    do_test("xor eax,eax", "31 c0");
    do_test("xor rax,rax", "48 31 c0");
    do_test("xor rax,r9", "4c 31 c8");
    do_test("xor r9,rax", "49 31 c1");
    do_test("xor r9,r9", "4d 31 c9");
}

#[test]
fn and() {
    do_test("and eax,1", "83 e0 01");
    do_test("and rax,1", "48 83 e0 01");
    do_test("and r9,1", "49 83 e1 01");
    do_test("and eax,eax", "23 c0"); // 21 c0
    do_test("and rax,rax", "48 23 c0");
    do_test("and rax,r9", "49 23 c1");
    do_test("and r9,rax", "4c 23 c8");
    do_test("and r9,r9", "4d 23 c9");
}

#[test]
fn or() {
    do_test("or eax,1", "83 c8 01");
    do_test("or rax,1", "48 83 c8 01");
    do_test("or r9,1", "49 83 c9 01");
    do_test("or eax,eax", "09 c0");
    do_test("or rax,rax", "48 09 c0");
    do_test("or rax,r9", "4c 09 c8");
    do_test("or r9,rax", "49 09 c1");
    do_test("or r9,r9", "4d 09 c9");
}

#[test]
fn cmp() {
    do_test("cmp eax,1", "83 f8 01");
    do_test("cmp rax,1", "48 83 f8 01");
    do_test("cmp r9,1", "49 83 f9 01");
    do_test("cmp eax,eax", "39 c0");
    do_test("cmp rax,rax", "48 39 c0");
    do_test("cmp rax,r9", "4c 39 c8");
    do_test("cmp r9,rax", "49 39 c1");
    do_test("cmp r9,r9", "4d 39 c9");
}

fn do_test(source: &str, expected_output: &str) {
    let output = tokenizer::tokenize(source.to_string())
        .and_then(|tokens| parser::parse(tokens))
        .and_then(|insts| generator::generate(insts))
        .unwrap();
    let actual_output = bytes_to_str(&output);
    assert_eq!(expected_output, actual_output, "failed with '{}'", source);
}

fn bytes_to_str(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<Vec<String>>()
        .join(" ")
}
