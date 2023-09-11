use core::ptr::null_mut;
use fiveos_riscv::cpu::registers::mcause::{AsyncCause, KnownCause, SyncCause, TrapCause};
use fiveos_virtio::plic::PLIC;
use fiveos_virtio::uart::{Uart, Uart0, UART_BASE_ADDRESS};

use crate::layout::StaticLayout;
use crate::{print, println};

/// Context information collected in trap.s before calling rust trap handler
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TrapFrame {
    /// General purpose registers
    pub regs: [usize; 32],
    /// Floating point purpose registers
    pub fregs: [usize; 32],
    /// satp
    pub satp: usize,
    /// stack pointer
    pub trap_stack: *mut u8,
    /// core id
    pub hartid: usize,
}

impl TrapFrame {
    const NULL: TrapFrame = TrapFrame {
        regs: [0; 32],
        fregs: [0; 32],
        satp: 0,
        trap_stack: null_mut(),
        hartid: 0,
    };
    pub const fn zero() -> TrapFrame {
        TrapFrame::NULL
    }
}

/// Global store of trapframes, one per core/hart. we're providing for 4 harts here.
/// todo: initialize in kinit with accurate number of harts
pub static mut GLOBAL_TRAPFRAMES: &mut [TrapFrame; 4] = &mut [TrapFrame::NULL; 4];

fn handle_external_interrupt(uart: &mut Uart0, hart: usize) {
    if let Some(interrupt) = PLIC.claim() {
        match interrupt.number() {
            10 => {
                // on qemu/virtio this is the UART singaling input
                // todo: can these match statements be extricated from the trap handler logic
                // so different hardware can be supported by compile options

                if let Some(c) = uart.get() {
                    match c {
                        8 => {
                            print!(uart, "\x08 \x08");
                        }
                        10 | 13 => {
                            println!(uart, "");
                        }
                        _ => {
                            print!(uart, "{}", c as char);
                        }
                    }
                }
            }
            _ => (),
        }
        PLIC.complete(interrupt);
    } else {
        println!(uart, "Machine external interrupt: core#{}", hart);
    }
}

#[no_mangle]
#[repr(align(4))]
extern "C" fn rust_trap(
    epc: usize,
    tval: usize,
    acause: usize,
    hart: usize,
    status: usize,
    frame: &mut TrapFrame,
) -> usize {
    let cause = TrapCause(acause).known_cause();
    let mut return_pc = epc;
    // let cause_number = cause.number();
    // safety: this is not actually safe. Only in place for debug prints.
    let mut uart = unsafe { Uart0::new() };
    use AsyncCause as AC;
    use KnownCause as KC;
    use SyncCause as SC;
    match cause {
        KC::Sync(sync_cause) => match sync_cause {
            SC::MachineSoftwareInterrupt => {
                println!(uart, "Machine software interrupt: core#{}", hart)
            }
            SC::MachineTimerInterrupt => println!(uart, "Machine timer interrupt: core#{}", hart),
            SC::MachineExternalInterrupt => handle_external_interrupt(&mut uart, hart),
            _ => panic!("Unhandled async trap: core#{} -> {:#x}\n", hart, acause),
        },
        KC::Async(async_cause) => match async_cause {
            AC::InstructionAddressMisaligned => panic!(
                "Instruction address misaligned: #{}/0x{:08x}/{}",
                hart, epc, tval
            ),
            AC::InstructionAccessFault => panic!(
                "Instruction access fault: #{}/0x{:08x}/{} {status}: {frame:?}",
                hart, epc, tval
            ),
            AC::IllegalInstruction => {
                panic!("Illegal instruction: #{}/0x{:08x}/{}", hart, epc, tval)
            }
            AC::Breakpoint => {
                println!(uart, "Skipped breakpoint: #{}/0x{:08x}", hart, epc);
                return_pc += 4;
            }
            AC::LoadAddressMisaligned => {
                panic!("Load address misaligned: #{}/0x{:08x}/{}", hart, epc, tval)
            }
            AC::LoadAccessFault => panic!("Load access fault: #{}/0x{:08x}/{}", hart, epc, tval),
            AC::StoreAMOAddressMisaligned => panic!(
                "Store/AMO address misaligned: #{}/0x{:08x}/{}",
                hart, epc, tval
            ),
            AC::StoreAMOAccessFault => panic!("Store/AMO fault: #{}/0x{:08x}/{}", hart, epc, tval),
            AC::EnvironmentCallUmode => {
                println!(
                    uart,
                    "External call from user mode: #{}/0x{:08x}", hart, epc
                );
                return_pc += 4;
            }
            AC::EnvironmentCallSmode => {
                println!(
                    uart,
                    "External call from supervisor mode:#{}/0x{:08x}", hart, epc
                );
                return_pc += 4;
            }
            AC::EnvironmentCallMmode => {
                println!(
                    uart,
                    "External call from Machine mode?!:#{}/0x{:08x}", hart, epc
                );
                return_pc += 4;
            }
            AC::InstructionPageFault => {
                panic!("Instruction page fault: #{}/0x{:08x}/{}", hart, epc, tval);
            }
            AC::LoadPageFault => {
                panic!("Load page fault: #{}/0x{:08x}/{}", hart, epc, tval);
            }
            AC::StoreAMOPageFault => {
                panic!("Store page fault: #{}/0x{:08x}/{}", hart, epc, tval);
            }
            AC::Reserved => {
                panic!("Unhandled synchronous trap: #{}/{:?}", hart, acause);
            }
        },
    }
    return_pc
}

pub fn setup_trap_handler() {
    let layout = StaticLayout::get();
    let _ts = layout.trap_start;
    // TODO: set up trap handler
}
