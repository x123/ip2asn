# Rust API Guidelines Checklist

## Naming (crate aligns with Rust naming conventions)

*   [x] Casing conforms to RFC 430 ([C-CASE](https://rust-lang.github.io/api-guidelines/naming.html#c-case))
*   [x] Ad-hoc conversions follow `as_`, `to_`, `into_` conventions ([C-CONV](https://rust-lang.github.io/api-guidelines/naming.html#c-conv))
*   [x] Getter names follow Rust convention ([C-GETTER](https://rust-lang.github.io/api-guidelines/naming.html#c-getter))
*   [x] Methods on collections that produce iterators follow `iter`, `iter_mut`, `into_iter` ([C-ITER](https://rust-lang.github.io/api-guidelines/naming.html#c-iter))
*   [x] Iterator type names match the methods that produce them ([C-ITER-TY](https://rust-lang.github.io/api-guidelines/naming.html#c-iter-ty))
*   [x] Feature names are free of placeholder words ([C-FEATURE](https://rust-lang.github.io/api-guidelines/naming.html#c-feature))
*   [x] Names use a consistent word order ([C-WORD-ORDER](https://rust-lang.github.io/api-guidelines/naming.html#c-word-order))

## Interoperability (crate interacts nicely with other library functionality)

*   [x] Types eagerly implement common traits ([C-COMMON-TRAITS](https://rust-lang.github.io/api-guidelines/interoperability.html#c-common-traits))
*   [x] Conversions use the standard traits `From`, `AsRef`, `AsMut` ([C-CONV-TRAITS](https://rust-lang.github.io/api-guidelines/interoperability.html#c-conv-traits))
*   [N] Collections implement `FromIterator` and `Extend` ([C-COLLECT](https://rust-lang.github.io/api-guidelines/interoperability.html#c-collect))
*   [N] Data structures implement Serde's `Serialize`, `Deserialize` ([C-SERDE](https://rust-lang.github.io/api-guidelines/interoperability.html#c-serde))
*   [x] Types are `Send` and `Sync` where possible ([C-SEND-SYNC](https://rust-lang.github.io/api-guidelines/interoperability.html#c-send-sync))
*   [x] Error types are meaningful and well-behaved ([C-GOOD-ERR](https://rust-lang.github.io/api-guidelines/interoperability.html#c-good-err))
*   [N] Binary number types provide `Hex`, `Octal`, `Binary` formatting ([C-NUM-FMT](https://rust-lang.github.io/api-guidelines/interoperability.html#c-num-fmt))
*   [x] Generic reader/writer functions take `"R: Read"` and `"W: Write"` by value ([C-RW-VALUE](https://rust-lang.github.io/api-guidelines/interoperability.html#c-rw-value))

## Macros (crate presents well-behaved macros)

*   [N] Input syntax is evocative of the output ([C-EVOCATIVE](https://rust-lang.github.io/api-guidelines/macros.html#c-evocative))
*   [N] Macros compose well with attributes ([C-MACRO-ATTR](https://rust-lang.github.io/api-guidelines/macros.html#c-macro-attr))
*   [N] Item macros work anywhere that items are allowed ([C-ANYWHERE](https://rust-lang.github.io/api-guidelines/macros.html#c-anywhere))
*   [N] Item macros support visibility specifiers ([C-MACRO-VIS](https://rust-lang.github.io/api-guidelines/macros.html#c-macro-vis))
*   [N] Type fragments are flexible ([C-MACRO-TY](https://rust-lang.github.io/api-guidelines/macros.html#c-macro-ty))

## Documentation (crate is abundantly documented)

*   [x] Crate level docs are thorough and include examples ([C-CRATE-DOC](https://rust-lang.github.io/api-guidelines/documentation.html#c-crate-doc))
*   [x] All items have a rustdoc example ([C-EXAMPLE](https://rust-lang.github.io/api-guidelines/documentation.html#c-example))
*   [x] Examples use `?`, not `try!`, not `unwrap` ([C-QUESTION-MARK](https://rust-lang.github.io/api-guidelines/documentation.html#c-question-mark))
*   [x] Function docs include error, panic, and safety considerations ([C-FAILURE](https://rust-lang.github.io/api-guidelines/documentation.html#c-failure))
*   [x] Prose contains hyperlinks to relevant things ([C-LINK](https://rust-lang.github.io/api-guidelines/documentation.html#c-link))
*   [x] Cargo.toml includes all common metadata ([C-METADATA](https://rust-lang.github.io/api-guidelines/documentation.html#c-metadata))
*   [ ] Release notes document all significant changes ([C-RELNOTES](https://rust-lang.github.io/api-guidelines/documentation.html#c-relnotes))
*   [x] Rustdoc does not show unhelpful implementation details ([C-HIDDEN](https://rust-lang.github.io/api-guidelines/documentation.html#c-hidden))

## Predictability (crate enables legible code that acts how it looks)

*   [N] Smart pointers do not add inherent methods ([C-SMART-PTR](https://rust-lang.github.io/api-guidelines/predictability.html#c-smart-ptr))
*   [N] Conversions live on the most specific type involved ([C-CONV-SPECIFIC](https://rust-lang.github.io/api-guidelines/predictability.html#c-conv-specific))
*   [x] Functions with a clear receiver are methods ([C-METHOD](https://rust-lang.github.io/api-guidelines/predictability.html#c-method))
*   [N] Functions do not take out-parameters ([C-NO-OUT](https://rust-lang.github.io/api-guidelines/predictability.html#c-no-out))
*   [N] Operator overloads are unsurprising ([C-OVERLOAD](https://rust-lang.github.io/api-guidelines/predictability.html#c-overload))
*   [x] Only smart pointers implement `Deref` and `DerefMut` ([C-DEREF](https://rust-lang.github.io/api-guidelines/predictability.html#c-deref))
*   [x] Constructors are static, inherent methods ([C-CTOR](https://rust-lang.github.io/api-guidelines/predictability.html#c-ctor))

## Flexibility (crate supports diverse real-world use cases)

*   [N] Functions expose intermediate results to avoid duplicate work ([C-INTERMEDIATE](https://rust-lang.github.io/api-guidelines/flexibility.html#c-intermediate))
*   [x] Caller decides where to copy and place data ([C-CALLER-CONTROL](https://rust-lang.github.io/api-guidelines/flexibility.html#c-caller-control))
*   [x] Functions minimize assumptions about parameters by using generics ([C-GENERIC](https://rust-lang.github.io/api-guidelines/flexibility.html#c-generic))
*   [N] Traits are object-safe if they may be useful as a trait object ([C-OBJECT](https://rust-lang.github.io/api-guidelines/flexibility.html#c-object))

## Type safety (crate leverages the type system effectively)

*   [N] Newtypes provide static distinctions ([C-NEWTYPE](https://rust-lang.github.io/api-guidelines/type-safety.html#c-newtype))
*   [x] Arguments convey meaning through types, not `bool` or `Option` ([C-CUSTOM-TYPE](https://rust-lang.github.io/api-guidelines/type-safety.html#c-custom-type))
*   [N] Types for a set of flags are `bitflags`, not enums ([C-BITFLAG](https://rust-lang.github.io/api-guidelines/type-safety.html#c-bitflag))
*   [x] Builders enable construction of complex values ([C-BUILDER](https://rust-lang.github.io/api-guidelines/type-safety.html#c-builder))

## Dependability (crate is unlikely to do the wrong thing)

*   [x] Functions validate their arguments ([C-VALIDATE](https://rust-lang.github.io/api-guidelines/dependability.html#c-validate))
*   [N] Destructors never fail ([C-DTOR-FAIL](https://rust-lang.github.io/api-guidelines/dependability.html#c-dtor-fail))
*   [N] Destructors that may block have alternatives ([C-DTOR-BLOCK](https://rust-lang.github.io/api-guidelines/dependability.html#c-dtor-block))

## Debuggability (crate is conducive to easy debugging)

*   [x] All public types implement `Debug` ([C-DEBUG](https://rust-lang.github.io/api-guidelines/debuggability.html#c-debug))
*   [x] `Debug` representation is never empty ([C-DEBUG-NONEMPTY](https://rust-lang.github.io/api-guidelines/debuggability.html#c-debug-nonempty))

## Future proofing (crate is free to improve without breaking users' code)

*   [N] Sealed traits protect against downstream implementations ([C-SEALED](https://rust-lang.github.io/api-guidelines/future-proofing.html#c-sealed))
*   [x] Structs have private fields ([C-STRUCT-PRIVATE](https://rust-lang.github.io/api-guidelines/future-proofing.html#c-struct-private))
*   [N] Newtypes encapsulate implementation details ([C-NEWTYPE-HIDE](https://rust-lang.github.io/api-guidelines/future-proofing.html#c-newtype-hide))
*   [N] Data structures do not duplicate derived trait bounds ([C-STRUCT-BOUNDS](https://rust-lang.github.io/api-guidelines/future-proofing.html#c-struct-bounds))

## Necessities (to whom they matter, they really matter)

*   [x] Public dependencies of a stable crate are stable ([C-STABLE](https://rust-lang.github.io/api-guidelines/necessities.html#c-stable))
*   [x] Crate and its dependencies have a permissive license ([C-PERMISSIVE](https://rust-lang.github.io/api-guidelines/necessities.html#c-permissive))