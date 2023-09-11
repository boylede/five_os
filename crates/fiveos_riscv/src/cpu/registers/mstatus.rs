///! Access to the mstatus csr, based on The RISC-V Instruction Set Manual Vol II, Privileged Architecture Version 1.9
use core::arch::asm;
use num_enum::{FromPrimitive, IntoPrimitive};
use paste::paste;

macro_rules! bool_access {
    ($field_name:ident, $desc:literal ,$mask:ident) => {
        paste! {
            #[doc="Get the \"" $desc "\" field from the mstatus register"]
            #[inline]
            pub fn [< get_ $field_name >](&self) -> bool {
                let tmp: usize;
                unsafe { asm!("csrr {tmp}, mstatus", tmp = out(reg) tmp); }
                (tmp & mask::$mask) != 0
            }

            #[doc="Set the \"" $desc "\" field in the mstatus register"]
            #[inline]
            pub fn [< set_ $field_name >] (&self, value: bool) {
                if value {
                    unsafe { asm!("csrs mstatus, {tmp}", tmp = in(reg) mask::$mask); }
                } else {
                    unsafe { asm!("csrc mstatus, {tmp}", tmp = in(reg) mask::$mask); }
                }
            }
        }
    };
}

macro_rules! enum_access {
    ($field_name:ident, $desc:literal, $enum_name:ident, $mask_offset:ident) => {
        paste! {
            #[doc="Get the \"" $desc "\" field from the mstatus register"]
            #[inline]
            pub fn [< get_ $field_name >] (&self) -> $enum_name {
                let tmp: usize;
                unsafe {asm!("csrr {tmp}, mstatus", tmp = out(reg) tmp);}
                let tmp: usize = ((tmp & mask::$mask_offset.0) >> mask::$mask_offset.1);
                <$enum_name as From<u8>>::from(tmp as u8)
            }
            #[doc="Set the \"" $desc "\" field in the mstatus register, first clearing that field."]
            #[inline]
            pub fn [< set_ $field_name >] (&self, value: $enum_name){
                let tmp = (<$enum_name as Into<u8>>::into(value) as usize) << mask::$mask_offset.1;
                let mask = mask::$mask_offset.0;
                unsafe {asm!("csrc mstatus, {mask}", "csrs mstatus, {tmp}", mask = in(reg) mask, tmp = in(reg) tmp);}
            }
        }
    };
}

/// ZST for accessing the MStatus register
#[derive(Clone, Copy)]
pub struct MStatus {}

impl MStatus {
    bool_access!(mie, "machine interrupts enabled", MIE);
    bool_access!(hie, "hypervisor interrupts enabled", HIE);
    bool_access!(sie, "supervisor interrupts enabled", SIE);
    bool_access!(uie, "user interrupts enabled", UIE);

    bool_access!(mpie, "machine interrupts enabled previously", MPIE);
    bool_access!(hpie, "hypervisor interrupts enabled previously", HPIE);
    bool_access!(spie, "supervisor interrupts enabled previously", SPIE);
    bool_access!(upie, "user interrupts enabled previously", UPIE);

    bool_access!(spp, "supervisor previous privilege", SPP);
    enum_access!(hpp, "hypervisor previous privilege", PrivilegeMode, HPP);
    enum_access!(mpp, "machine previous privilege", PrivilegeMode, MPP);

    enum_access!(fs, "floating point status", UnitStatus, FS);
    enum_access!(xs, "extension status", UnitStatus, XS);
    bool_access!(sd, "some dirty", SD);

    bool_access!(mprv, "memory privilege", MPRV);
    bool_access!(mxr, "make executable readable", MXR);
    bool_access!(pum, "protect user memory", PUM);
    enum_access!(vm, "virtualization management", VirtualMemoryMode, VM);
}

/// State of processor units that may need context saves,
/// e.g. the floating point unit or vendor-defined extension
///
/// In the case of multiple vendor defined extensions, the
/// value provided is the worst-case.
#[derive(Debug, Eq, PartialEq, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum UnitStatus {
    /// All are off
    #[num_enum(default)]
    Off = 0,
    /// None are dirty or clean
    Initial = 1,
    /// None are dirty
    Clean = 2,
    /// Some are dirty
    Dirty = 3,
}

/// The state of virtual memory management.
/// Note that reserved values are not preserved here
#[repr(u8)]
#[derive(Debug, Eq, PartialEq, FromPrimitive, IntoPrimitive)]
pub enum VirtualMemoryMode {
    Bare = 0,
    BaseBound = 1,
    SeparateBaseBound = 2,
    #[num_enum(default)]
    Reserved,
    Sv32 = 8,
    Sv39 = 9,
    Sv48 = 10,
    Sv57 = 11,
    Sv64 = 12,
}

/// The mode that determines what access the CPU currently has
#[derive(Debug, Eq, PartialEq, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum PrivilegeMode {
    #[num_enum(default)]
    User = 0,
    Supervisor = 1,
    Hypervisor = 2,
    Machine = 3,
}

mod mask {
    //! masks for the fields inside mstatus

    use core::mem::size_of;
    const XLEN: usize = size_of::<usize>() * 8;

    // interrupt-enable bits for each privilege mode
    pub const UIE: usize = 1 << 0;
    pub const SIE: usize = 1 << 1;
    pub const HIE: usize = 1 << 2;
    pub const MIE: usize = 1 << 3;

    // previous interrupt enable status
    pub const UPIE: usize = 1 << 4;
    pub const SPIE: usize = 1 << 5;
    pub const HPIE: usize = 1 << 6;
    pub const MPIE: usize = 1 << 7;

    // previous privilege modes
    pub const SPP: usize = 1 << 8;
    pub const HPP: (usize, usize) = (0b11 << 9, 9);
    pub const MPP: (usize, usize) = (0b11 << 11, 11);

    // processor extension status
    pub const FS: (usize, usize) = (0b11 << 13, 13);
    pub const XS: (usize, usize) = (0b11 << 15, 15);
    pub const SD: usize = 1 << (XLEN - 1);

    // protection modes
    pub const MPRV: usize = 1 << 17;
    pub const PUM: usize = 1 << 18;
    pub const MXR: usize = 1 << 19;
    pub const VM: (usize, usize) = (0b1111 << 24, 24);
}
