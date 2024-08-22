use crate::{Cpu, Exception};

impl Cpu {
  pub(crate) fn execute_general(&mut self, inst: u64) -> Result<(), Exception> {
    macro_rules! inst {
      ($name:literal => { $($tt:tt)* }) => {
        { $($tt)* }
      };
    }

    let (opcode, rd, funct3, rs1, rs2, funct7) = (
      inst & 0x0000007f,
      (inst & 0x00000f80) >> 7,
      (inst & 0x00007000) >> 12,
      (inst & 0x000f8000) >> 15,
      (inst & 0x01f00000) >> 20,
      (inst & 0xfe000000) >> 25,
    );

    Ok(match opcode {
      0x13 => {
        // imm[11:0] = inst[31:20]
        let imm = (inst as i32 as i64 >> 20) as u64;
        let funct6 = funct7 >> 1;
        match funct3 {
          0x0 => inst!("addi" => {
            self.xregs.store(rd, self.xregs.load(rs1).wrapping_add(imm));
          }),
          _ => return Err(Exception::IllegalInst(inst)),
        }
      }
      0x33 => match (funct3, funct7) {
        (0x0, 0x00) => inst!("add" => {
          self.xregs.store(rd, self.xregs.load(rs1).wrapping_add(self.xregs.load(rs2)));
        }),
        _ => return Err(Exception::IllegalInst(inst)),
      },
      _ => return Err(Exception::IllegalInst(inst)),
    })
  }
}
