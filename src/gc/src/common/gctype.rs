#![allow(dead_code)]

use std::sync::Arc;
use utils::POINTER_SIZE;
use utils::ByteSize;
use objectmodel;

use std::u32;
pub const GCTYPE_INIT_ID: u32 = u32::MAX;

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct GCType {
    pub id: u32,
    pub size: ByteSize,
    alignment: ByteSize,
    pub non_repeat_refs: Option<RefPattern>,
    pub repeat_refs    : Option<RepeatingRefPattern>,
}

impl GCType {
    pub fn new(id: u32, size: ByteSize, alignment: ByteSize, non_repeat_refs: Option<RefPattern>, repeat_refs: Option<RepeatingRefPattern>) -> GCType {
        GCType {
            id: id,
            size: size,
            alignment: objectmodel::check_alignment(alignment),
            non_repeat_refs: non_repeat_refs,
            repeat_refs: repeat_refs
        }
    }

    pub fn new_noreftype(size: ByteSize, align: ByteSize) -> GCType {
        GCType {
            id: GCTYPE_INIT_ID,
            size: size,
            alignment: align,
            non_repeat_refs: None,
            repeat_refs    : None,
        }
    }

    pub fn new_reftype() -> GCType {
        GCType {
            id: GCTYPE_INIT_ID,
            size: POINTER_SIZE,
            alignment: POINTER_SIZE,
            non_repeat_refs: Some(RefPattern::Map{
                offsets: vec![0],
                size: POINTER_SIZE
            }),
            repeat_refs: None
        }
    }

    #[allow(unused_assignments)]
    pub fn gen_ref_offsets(&self) -> Vec<ByteSize> {
        let mut ret = vec![];

        let mut cur_offset = 0;

        match self.non_repeat_refs {
            Some(ref pattern) => {
                cur_offset = pattern.append_offsets(cur_offset, &mut ret);
            }
            None => {}
        }

        if self.repeat_refs.is_some() {
            let repeat_refs = self.repeat_refs.as_ref().unwrap();

            cur_offset = repeat_refs.append_offsets(cur_offset, &mut ret);
        }

        ret
    }
}

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub enum RefPattern {
    Map{
        offsets: Vec<ByteSize>,
        size : usize
    },
    NestedType(Vec<Arc<GCType>>)
}

impl RefPattern {
    pub fn append_offsets(&self, base: ByteSize, vec: &mut Vec<ByteSize>) -> ByteSize {
        match self {
            &RefPattern::Map{ref offsets, size} => {
                for off in offsets {
                    vec.push(base + off);
                }

                base + size
            }
            &RefPattern::NestedType(ref types) => {
                let mut cur_base = base;

                for ty in types {
                    let nested_offset = ty.gen_ref_offsets();
                    let mut nested_offset = nested_offset.iter().map(|x| x + cur_base).collect();

                    vec.append(&mut nested_offset);

                    cur_base += ty.size;
                }

                cur_base
            }
        }
    }
}

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct RepeatingRefPattern {
    pub pattern: RefPattern,
    pub count: usize
}

impl RepeatingRefPattern {
    pub fn append_offsets(&self, base: ByteSize, vec: &mut Vec<ByteSize>) -> ByteSize {
        let mut cur_base = base;

        for _ in 0..self.count {
            cur_base = self.pattern.append_offsets(cur_base, vec);
        }

        cur_base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use utils::ByteSize;

    fn create_types() -> Vec<GCType> {
        // linked list: struct {ref, int64}
        let a = GCType{
            id: 0,
            size: 16,
            alignment: 8,
            non_repeat_refs: Some(RefPattern::Map{
                offsets: vec![0],
                size: 16
            }),
            repeat_refs    : None
        };

        // array of struct {ref, int64} with length 10
        let b = GCType {
            id: 1,
            size: 160,
            alignment: 8,
            non_repeat_refs: None,
            repeat_refs    : Some(RepeatingRefPattern {
                pattern: RefPattern::Map{
                    offsets: vec![0],
                    size   : 16
                },
                count  : 10
            }),
        };

        // array(10) of array(10) of struct {ref, int64}
        let c = GCType {
            id: 2,
            size: 1600,
            alignment: 8,
            non_repeat_refs: None,
            repeat_refs    : Some(RepeatingRefPattern {
                pattern: RefPattern::NestedType(vec![Arc::new(b.clone()).clone()]),
                count  : 10
            })
        };

        vec![a, b, c]
    }

    #[test]
    fn test_types() {
        create_types();
    }

    #[test]
    fn test_ref_offsets() {
        let vec = create_types();

        assert_eq!(vec[0].gen_ref_offsets(), vec![0]);
        assert_eq!(vec[1].gen_ref_offsets(), vec![0, 16, 32, 48, 64, 80, 96, 112, 128, 144]);
        assert_eq!(vec[2].gen_ref_offsets(), (0..100).map(|x| x * 16).collect::<Vec<ByteSize>>());

        let int = GCType {
            id: 3,
            size: 8,
            alignment: 8,
            non_repeat_refs: None,
            repeat_refs: None
        };

        assert_eq!(int.gen_ref_offsets(), vec![]);
    }
}
