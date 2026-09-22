use core::hash::{Hash, Hasher};
use core::num::NonZero;

use aldebaran_dsa::{collect::static_vec::StaticVec, prelude::ReverseMap};
use aldebaran_hash::fnv::FnvHasher;
use aldebaran_heap::{boxed::Box as HeapBox, heap::Heap};
use aldebaran_ice::Ice;
use aldebaran_id::{
    Id,
    prelude::{Id as IdTrait, IdMap},
};
use aldebaran_primitive::prelude::{LosslessCast, TryCast};
use aldebaran_span::prelude::Span;

Id!(
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    become SymbolId
    as "symbol#{}"
);

SymbolId!(32 become);

fn main() {
    let widened: u16 = 42_u8.lossless_cast();
    let narrowed: Option<u8> = 250_u16.try_cast();

    assert_eq!(widened, 42);
    assert_eq!(narrowed, Some(250));

    let length = NonZero::new(3).expect("three is nonzero");
    let span = Span::new(4, length);

    assert!(span.contains(5));
    assert_eq!(span.range(), 4..7);

    let mut inline = StaticVec::<u8, 4>::empty();
    assert_eq!(inline.push(1), Ok(()));
    assert_eq!(inline.push(2), Ok(()));
    assert_eq!(inline.as_slice(), &[1, 2]);

    let mut paired = ReverseMap::new();
    paired.push("answer", 42_u8);
    paired.push("fallback", 7_u8);
    assert_eq!(paired.get_key_value(0), Some((&"answer", &42)));

    let mut hasher = FnvHasher::default();
    "aldebaran".hash(&mut hasher);
    let symbol_hash = hasher.finish();

    let boxed: HeapBox<u32> = HeapBox::new_in(42, Heap::DEFAULT);
    assert_eq!(*boxed, 42);

    let symbol_id = SymbolId::MIN;
    let mut symbols: IdMap<SymbolId, &str> = IdMap::new();
    assert_eq!(symbols.insert(symbol_id, "aldebaran"), None);
    assert_eq!(symbols.get(symbol_id), Some(&"aldebaran"));
    assert_eq!(IdTrait::primitive(&symbol_id), 1);

    let guaranteed = Ice::<Option<&str>>::unwrap(Some("reachable"));
    assert_eq!(guaranteed, "reachable");

    println!("span {span:?}");
    println!("symbol {symbol_id} hashes source text to {symbol_hash:#x}");
}
