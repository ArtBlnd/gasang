#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RmRnRd {
    pub rm: u8,
    pub rn: u8,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RmRaRnRd {
    pub rm: u8,
    pub ra: u8,
    pub rn: u8,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RnRd {
    pub rn: u8,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShImm12RnRd {
    pub sh: u8,
    pub imm12: u16,
    pub rn: u8,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SizeImm12RnRt {
    pub idxt: u8,
    pub size: u8,
    pub imm12: u16,
    pub rn: u8,
    pub rt: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Imm26 {
    pub imm26: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Imm19Cond {
    pub imm19: u32,
    pub cond: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RmCondRnRd {
    pub rm: u8,
    pub cond: u8,
    pub rn: u8,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HwImm16Rd {
    pub hw: u8,
    pub imm16: u16,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Imm16Rd {
    pub imm16: u16,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct B5B40Imm14Rt {
    pub b5: u8,
    pub b40: u8,
    pub imm14: u16,
    pub rt: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShiftRmImm6RnRd {
    pub shift: u8,
    pub rm: u8,
    pub imm6: u8,
    pub rn: u8,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UncondBranchReg {
    pub z: u8,
    pub op: u8,
    pub a: u8,
    pub rn: u8,
    pub rm: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcRelAddressing {
    pub immlo: u8,
    pub immhi: u32,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExceptionGen {
    pub opc: u8,
    pub imm16: u16,
    pub op2: u8,
    pub ll: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadStoreRegRegOffset {
    pub size: u8,
    pub v: u8,
    pub opc: u8,
    pub rm: u8,
    pub option: u8,
    pub s: u8,
    pub rn: u8,
    pub rt: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddSubtractExtReg {
    pub rm: u8,
    pub option: u8,
    pub imm3: u8,
    pub rn: u8,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bitfield {
    pub n: u8,
    pub immr: u8,
    pub imms: u8,
    pub rn: u8,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogicalImm {
    pub n: u8,
    pub immr: u8,
    pub imms: u8,
    pub rn: u8,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadStoreRegPair {
    pub imm7: u8,
    pub o: u8,
    pub rt2: u8,
    pub rn: u8,
    pub rt: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddSubImmWithTags {
    pub o2: u8,
    pub uimm6: u8,
    pub op3: u8,
    pub uimm4: u8,
    pub rn: u8,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtractImm {
    pub rm: u8,
    pub imms: u8,
    pub rn: u8,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Imm19Rt {
    pub imm19: u32,
    pub rt: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataProc3Src {
    pub rm: u8,
    pub ra: u8,
    pub rn: u8,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SysRegMov {
    pub o0: u8,
    pub op1: u8,
    pub crn: u8,
    pub crm: u8,
    pub op2: u8,
    pub rt: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataProc2Src {
    pub rm: u8,
    pub rn: u8,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Barriers {
    pub crm: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdvancedSimdCopy {
    pub imm5: u8,
    pub imm4: u8,
    pub rn: u8,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CondCmpReg {
    pub rm: u8,
    pub cond: u8,
    pub rn: u8,
    pub nzcv: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdvSimdLdStMultiStructures {
    pub q: u8,
    pub size: u8,
    pub rn: u8,
    pub rt: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdvancedSimdExtract {
    pub q: u8,
    pub rm: u8,
    pub imm4: u8,
    pub rn: u8,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdvSimdLdStMultiStructuresPostIndexed {
    pub q: u8,
    pub rm: u8,
    pub size: u8,
    pub rn: u8,
    pub rt: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdvSimdModifiedImm {
    pub q: u8,
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub cmode: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub g: u8,
    pub h: u8,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CondCmpImm {
    pub imm5: u8,
    pub cond: u8,
    pub rn: u8,
    pub nzcv: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadStoreRegExclusive {
    pub rs: u8,
    pub rt2: u8,
    pub rn: u8,
    pub rt: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadStoreOrdered {
    pub rs: u8,
    pub rt2: u8,
    pub rn: u8,
    pub rt: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QSizeRmRnRd {
    pub q: u8,
    pub size: u8,
    pub rm: u8,
    pub rn: u8,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdvSimdShiftByImm {
    pub q: u8,
    pub immb: u8,
    pub rn: u8,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdvSimdScalarPairwise {
    pub size: u8,
    pub rn: u8,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdvSimdLdStSingleStructure {
    pub q: u8,
    pub s: u8,
    pub size: u8,
    pub rn: u8,
    pub rt: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QSizeRnRd {
    pub q: u8,
    pub size: u8,
    pub rn: u8,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Imm16 {
    pub imm16: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RsRnRt {
    pub rs: u8,
    pub rn: u8,
    pub rt: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FloatingPointCompare {
    pub ptype: u8,
    pub rm: u8,
    pub rn: u8,
    pub opcode2: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FloatingPointImmediate {
    pub imm8: u8,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConvBetweenFloatAndFixedPoint {
    pub scale: u8,
    pub rn: u8,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdvSimdXIndexedElem {
    pub q: u8,
    pub size: u8,
    pub l: u8,
    pub m: u8,
    pub rm: u8,
    pub h: u8,
    pub rn: u8,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdvSimdScalarXIndexedElem {
    pub size: u8,
    pub l: u8,
    pub m: u8,
    pub rm: u8,
    pub h: u8,
    pub rn: u8,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rt {
    pub rt: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rn {
    pub rn: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pstate {
    pub op1: u8,
    pub crm: u8,
    pub op2: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemInstructions {
    pub op1: u8,
    pub crn: u8,
    pub crm: u8,
    pub op2: u8,
    pub rt: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RotateRightIntoFlags {
    pub imm6: u8,
    pub rn: u8,
    pub mask: u8,
}
