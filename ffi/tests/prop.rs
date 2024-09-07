use {
  ffi::{
    vrisc::{Bus, Cpu, Dram, Mode, State, Xregs, DRAM_SIZE, REG_COUNT},
    CpuRepr,
  },
  proptest::{collection::vec, prelude::*},
};

fn mode() -> impl Strategy<Value = Mode> {
  prop_oneof![Just(Mode::User), Just(Mode::Supervisor), Just(Mode::Machine)]
}

prop_compose! {
  fn xregs()(repr in vec(any::<u64>(), REG_COUNT..=REG_COUNT)) -> Xregs {
    let mut xregs = Xregs::new();
    for i in 0..REG_COUNT {
      xregs.store(i as u64, repr[i]);
    }
    xregs
  }
}

prop_compose! {
  fn bus()(bytes in vec(any::<u8>(), 128..=128)) -> Bus {
    let mut dram = Dram::with_capacity(bytes.len());
    dram.init(&bytes);
    Bus { dram }
  }
}

prop_compose! {
  fn cpu()(pc in any::<u64>(), mode in mode(), xregs in xregs(), bus in bus()) -> Cpu {
    Cpu { pc, mode, xregs, state: State::new(), bus }
  }
}

proptest! {
    #[test]
    fn prop(a in cpu()) {
      let repr = CpuRepr::from_cpu(&a);

      let mut b = Cpu::new(128);
      unsafe { repr.map_to(&mut b); }

      prop_assert!(a.pc == b.pc);
      prop_assert!(a.mode as u8 == b.mode as u8);
      prop_assert!(a.xregs.into_inner() == b.xregs.into_inner());
      prop_assert!(a.bus.dram.as_slice() == b.bus.dram.as_slice());
    }
}
