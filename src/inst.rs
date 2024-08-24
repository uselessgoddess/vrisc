use crate::{
  cpu::{BYTE, DWORD, HALF, WORD},
  Cpu, Exception,
};

impl Cpu {
  pub(crate) fn execute_general(&mut self, inst: u64) -> Result<(), Exception> {
    macro_rules! inst {
      ($name:literal => { $($tt:tt)* }) => {
        { self.debug(inst, $name); $($tt)* }
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
          0x1 => inst!("slli" => {
            let shift = (inst >> 20) & 0x3f;
            self.xregs.store(rd, self.xregs.load(rs1) << shift);
          }),
          0x2 => inst!("slti" => {
            let bit = if (self.xregs.load(rs1) as i64) < (imm as i64) { 1 } else { 0 };
            self.xregs.store(rd, bit);
          }),
          0x3 => inst!("sltiu" => {
            self.xregs.store(rd, if self.xregs.load(rs1) < imm  { 1 } else { 0 });
          }),
          0x4 => inst!("xori" => {
            self.xregs.store(rd, self.xregs.load(rs1) ^ imm);
          }),
          0x5 => match funct6 {
            0x00 => inst!("srli" => {
              let shift = (inst >> 20) & 0x3f;
              self.xregs.store(rd, self.xregs.load(rs1) >> shift);
            }),
            0x10 => inst!("srai" => {
              let shift = (inst >> 20) & 0x3f;
              self.xregs.store(rd, ((self.xregs.load(rs1) as i64) >> shift) as u64);
            }),
            _ => return Err(Exception::IllegalInst(inst)),
          },
          0x6 => inst!("ori" => {
            self.xregs.store(rd, self.xregs.load(rs1) | imm);
          }),
          0x7 => inst!("andi" => {
            self.xregs.store(rd, self.xregs.load(rs1) & imm);
          }),
          _ => return Err(Exception::IllegalInst(inst)),
        }
      }
      0x23 => {
        // offset[11:5|4:0] = inst[31:25|11:7]
        let offset = (((inst & 0xfe000000) as i32 as i64 >> 20) as u64)
          | ((inst >> 7) & 0x1f);
        let addr = self.xregs.load(rs1).wrapping_add(offset);
        match funct3 {
          0x0 => inst!("sb" => {
            self.store(addr, self.xregs.load(rs2), BYTE)?;
          }),
          0x1 => inst!("sh" => {
            self.store(addr, self.xregs.load(rs2), HALF)?;
          }),
          0x2 => inst!("sw" => {
            self.store(addr, self.xregs.load(rs2), WORD)?;
          }),
          0x3 => inst!("sd" => {
            self.store(addr, self.xregs.load(rs2), DWORD)?;
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
      0x67 => inst!("jalr" => {
        let t = self.pc.wrapping_add(4);

        let imm = inst as i32 as i64 >> 20;
        let target = (self.xregs.load(rs1) as i64).wrapping_add(imm) & !1;

        self.pc = (target as u64).wrapping_sub(4);
        self.xregs.store(rd, t);
      }),
      0x6f => inst!("jal" => {
        self.xregs.store(rd, self.pc.wrapping_add(4));

        let imm = (((inst & 0x80000000) as i32 as i64 >> 11) as u64)
          | (inst & 0xff000)
          | ((inst >> 9) & 0x800)
          | ((inst >> 20) & 0x7fe);
        self.pc = self.pc.wrapping_add(imm).wrapping_sub(4);
      }),
      _ => return Err(Exception::IllegalInst(inst)),
    })
  }
}
