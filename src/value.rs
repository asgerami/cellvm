//! Tagged runtime values.

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tag {
    Int = 1,
    Ref = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Value {
    tag: Tag,
    payload: u64,
}

impl Value {
    pub fn int(v: i64) -> Self {
        Self { tag: Tag::Int, payload: v as u64 }
    }

    pub fn ref_obj(id: u32) -> Self {
        Self { tag: Tag::Ref, payload: id as u64 }
    }

    pub fn tag(self) -> Tag {
        self.tag
    }

    pub fn payload(self) -> u64 {
        self.payload
    }

    pub fn as_int(self) -> Option<i64> {
        if self.tag == Tag::Int { Some(self.payload as i64) } else { None }
    }

    pub fn as_ref_id(self) -> Option<u32> {
        if self.tag == Tag::Ref { Some(self.payload as u32) } else { None }
    }
}

#[repr(C)]
pub struct Obj {
    pub kind: u32,
    pub mark: u32,
}
