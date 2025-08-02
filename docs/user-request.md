### **Title: Improve Ergonomics for Common Use Cases: Empty Maps and Async Lookups**

**Is your feature request related to a problem? Please describe.**

While using `ip2asn` in a high-performance, asynchronous application, I've encountered two ergonomic hurdles that require non-obvious or `unsafe` code to work around.

1.  **Creating an empty `IpAsnMap` is verbose and unintuitive.** The only way to create an empty map for an initial state (e.g., before data has been loaded) is via the builder with an empty source: `Builder::new().with_source(&[][..]).unwrap().build().unwrap()`. This is not immediately obvious from the documentation and is quite verbose for such a common requirement.

2.  **Using `lookup` in async contexts with `arc-swap` requires `unsafe` code.** The `lookup` method returns a lifetime-bound `AsnInfoView<'_>`. When the `IpAsnMap` is wrapped in an `ArcSwap` for zero-downtime reloads, the `Guard` from `arc_swap` is dropped at the end of the async function, making it impossible to return the view. The only workaround is to use `unsafe { std::mem::transmute(v) }` to extend the lifetime to `'static`. While this is technically safe in this specific `ArcSwap` context (because the underlying data is `Arc`'d and won't be deallocated), it introduces `unsafe` into otherwise safe application code and relies on a deep understanding of the implementation details of both libraries.

**Describe the solution you'd like**

I propose two enhancements to the `ip2asn` API to address these issues:

1.  **Add `IpAsnMap::new()` or `IpAsnMap::empty()`:** A simple, direct constructor for creating an empty map would significantly improve ergonomics.

    ```rust
    // Ideal API
    let empty_map = IpAsnMap::new();
    ```

2.  **Provide an owned lookup variant:** Add a method like `lookup_owned` that returns an owned struct (e.g., `AsnInfo`) instead of a view. This would involve cloning the underlying `&str` fields into `String`s. While this has a minor performance cost (an allocation), it would provide a safe, simple, and idiomatic way to handle lookups in async contexts, completely eliminating the need for `unsafe`.

    ```rust
    // An owned version of AsnInfo
    pub struct AsnInfo {
        pub asn: u32,
        pub country_code: String,
        pub organization: String,
    }

    impl IpAsnMap {
        // New method
        pub fn lookup_owned(&self, ip: IpAddr) -> Option<AsnInfo> {
            self.lookup(ip).map(|view| AsnInfo {
                asn: view.asn,
                country_code: view.country_code.to_string(),
                organization: view.organization.to_string(),
            })
        }
    }
    ```

**Describe alternatives you've considered**

-   For the empty map, the current `Builder` approach works but is not ideal.
-   For the async lookup, I considered trying to use `Box::leak` or other complex lifetime extension techniques, but they are all more complex and less clear than the proposed `lookup_owned` method. The `unsafe` transmute is the most direct, albeit undesirable, solution currently available.

These changes would make the `ip2asn` crate even more pleasant and safe
to use in a wider variety of applications. Thank you for your
consideration!
