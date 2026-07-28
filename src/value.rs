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

impl Value {
    pub fn is_int(self) -> bool {
        self.tag == Tag::Int
    }

    pub fn is_ref(self) -> bool {
        self.tag == Tag::Ref
    }

    pub fn nil_ref() -> Self {
        Self::ref_obj(0)
    }

    pub fn zero() -> Self {
        Self::int(0)
    }

    pub fn one() -> Self {
        Self::int(1)
    }

    pub fn from_bool(b: bool) -> Self {
        Self::int(i64::from(b))
    }

    pub fn to_bool(self) -> bool {
        match self.tag {
            Tag::Int => self.payload != 0,
            Tag::Ref => self.payload != 0,
        }
    }

    pub fn bitcast_payload(self) -> u64 {
        self.payload
    }

    pub fn with_payload(tag: Tag, payload: u64) -> Self {
        Self { tag, payload }
    }
}

impl Obj {
    pub fn new(kind: u32) -> Self {
        Self { kind, mark: 0 }
    }

    pub fn with_mark(kind: u32, mark: u32) -> Self {
        Self { kind, mark }
    }

    pub fn touch(&mut self, mark: u32) {
        self.mark = mark;
    }

    pub fn same_kind(&self, other: &Obj) -> bool {
        self.kind == other.kind
    }
}

pub fn tag_name(tag: Tag) -> &'static str {
    match tag {
        Tag::Int => "int",
        Tag::Ref => "ref",
    }
}

pub fn values_equal(a: Value, b: Value) -> bool {
    a.tag() == b.tag() && a.payload() == b.payload()
}

pub fn pack_value_pair(a: Value, b: Value) -> (u8, u64, u8, u64) {
    (a.tag() as u8, a.payload(), b.tag() as u8, b.payload())
}
