use macros::{imm, slice};

#[test]
fn imm() {
  let inst = 0x420daa6f; // jal x20, 893984

  let a = (((inst & 0x80000000) as i32 as i64 >> 11) as u64)
    | (inst & 0xff000)
    | ((inst >> 9) & 0x800)
    | ((inst >> 20) & 0x7fe);

  let inst = inst >> 12;

  let b = ((inst & (0b11111111 << 0)) >> 0) << 12
    | ((inst & (1 << 8)) >> 8) << 11
    | ((inst & (0b1111111111 << 9)) >> 9) << 1
    | ((inst & (1 << 19)) >> 19) << 20;

  let c = imm![inst in 20|10:1|11|19:12];

  assert_eq!(a, b);
  assert_eq!(b, c);

  let inst = 0x4c001963; // bne x0, x0, 1234
  let inst = (inst & 0b11111 << 7) >> 7 | (inst & 0b1111111 << 25);

  let x = imm![inst in 12|10:5|4:1|11];
  println!("{:014b}", 1234);
  println!("{:014b}", x);
  assert_eq!(1234, x);
}

#[test]
fn slice() {
  let repr = |inst: u64| {
    imm![
      slice![inst in 31:25|11:7] in 12|10:5|4:1|11
    ]
  };
  assert_eq!(1234, repr(0x4c000963)); // beq x0, x0, 1234
  assert_eq!(2222, repr(0x0a0057e3)); // bge x0, x0, 2222
  assert_eq!(0, repr(0x00000063)); // beq x0, x0, 0

  let store = |inst: u64| slice![inst in 31:25|11:7];
  assert_eq!(1234, store(0x4c000923)); // sb x0, 1234(x0)

  let auipc = |inst: u64| slice![inst in 31:12];
  assert_eq!(0, auipc(0x297)); // auipc x5, 0  
  assert_eq!(184185, auipc(0x2cf79017)); // auipc x0, 184185
}
