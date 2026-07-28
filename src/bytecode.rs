//! Opcode set and instruction sizing (runtime ISA).

pub const NOP: u8 = 0x00;
pub const PUSH8: u8 = 0x01;
pub const WINDOW: u8 = 0x02;
pub const SJMP: u8 = 0x20;
pub const LOADK: u8 = 0x30;
pub const LOADLOCAL: u8 = 0x31;
pub const STORELOCAL: u8 = 0x32;
pub const INTERN: u8 = 0x33;
pub const STRCMP: u8 = 0x34;
pub const CALL: u8 = 0x35;
pub const TAIL: u8 = 0x36;
pub const CLOSE: u8 = 0x37;
pub const GETUPVAL: u8 = 0x38;
pub const SETUPVAL: u8 = 0x39;
pub const THROW: u8 = 0x3A;
pub const OPENUPVAL: u8 = 0x3B;
pub const NEWARR: u8 = 0x40;
pub const GETFIELD: u8 = 0x41;
pub const RET: u8 = 0xFF;

pub fn instr_len(op: u8) -> Option<usize> {
    match op {
        NOP | RET | STRCMP | THROW => Some(1),
        PUSH8 | WINDOW | SJMP | CLOSE | GETUPVAL | SETUPVAL | OPENUPVAL => Some(2),
        LOADK | LOADLOCAL | STORELOCAL | INTERN | CALL | TAIL => Some(3),
        NEWARR | GETFIELD => Some(4),
        _ => None,
    }
}

pub fn is_control(op: u8) -> bool {
    matches!(op, SJMP | CALL | TAIL | THROW | RET)
}

pub fn name(op: u8) -> Option<&'static str> {
    match op {
        NOP => Some("nop"),
        PUSH8 => Some("push8"),
        WINDOW => Some("window"),
        SJMP => Some("sjmp"),
        LOADK => Some("loadk"),
        LOADLOCAL => Some("loadlocal"),
        STORELOCAL => Some("storelocal"),
        INTERN => Some("intern"),
        STRCMP => Some("strcmp"),
        CALL => Some("call"),
        TAIL => Some("tail"),
        CLOSE => Some("close"),
        GETUPVAL => Some("getupval"),
        SETUPVAL => Some("setupval"),
        THROW => Some("throw"),
        OPENUPVAL => Some("openupval"),
        NEWARR => Some("newarr"),
        GETFIELD => Some("getfield"),
        RET => Some("ret"),
        _ => None,
    }
}

pub fn stack_effect(op: u8) -> i8 {
    match op {
        PUSH8 | LOADK | LOADLOCAL | INTERN | GETUPVAL | NEWARR => 1,
        STORELOCAL | SETUPVAL | STRCMP => -1,
        GETFIELD => 0,
        _ => 0,
    }
}
