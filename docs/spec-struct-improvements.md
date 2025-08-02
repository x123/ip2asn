#### **1. Empty `IpAsnMap` Constructor**

**Objective:** Provide a simple and idiomatic way to create an empty `IpAsnMap`.

**Implementation Details:**

1.  **Implement the `Default` Trait for `IpAsnMap`:**
    *   The `default()` method should return a new `IpAsnMap` instance where the `table` is an empty `IpNetworkTable::new()` and `organizations` is an empty `Vec::new()`.

2.  **Implement `IpAsnMap::new()`:**
    *   Add a public function `pub fn new() -> Self`.
    *   This function should be a simple wrapper that calls `Self::default()`.
    *   Add a doc comment explaining that it creates a new, empty map.

---

#### **2. Owned ASN Information Struct and Lookup Method**

**Objective:** Provide a safe, owned version of the lookup result to improve ergonomics in async contexts and other situations where a self-contained struct is needed.

**Implementation Details:**

1.  **Add an Optional `serde` Feature:**
    *   In `Cargo.toml`, add a new optional feature named `serde`.
    *   This feature should enable the `serde` dependency (with the `derive` feature) and also enable the `serde` feature for the `ip_network` dependency.
        ```toml
        [features]
        serde = ["dep:serde", "ip_network/serde"]

        [dependencies]
        serde = { version = "1.0", features = ["derive"], optional = true }
        ip_network = { version = "0.4", features = ["serde"] }
        ```

2.  **Define the `AsnInfo` Struct:**
    *   Create a new public struct named `AsnInfo`.
    *   The struct definition should be as follows:
        ```rust
        #[derive(Debug, Clone, PartialEq, Eq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub struct AsnInfo {
            pub network: ip_network::IpNetwork,
            pub asn: u32,
            pub country_code: String,
            pub organization: String,
        }
        ```
    *   Ensure all fields are public.
    *   Add comprehensive doc comments for the struct and each of its fields.

3.  **Implement `IpAsnMap::lookup_owned()`:**
    *   Add a new public method to `IpAsnMap` with the signature `pub fn lookup_owned(&self, ip: IpAddr) -> Option<AsnInfo>`.
    *   The implementation should call the existing `self.lookup(ip)` method.
    *   If the lookup returns `Some(view)`, it should map the `AsnInfoView` to a new `AsnInfo` instance by cloning the `&str` fields into `String`s.
    *   Add a doc comment explaining that this method returns an owned result, making it suitable for async contexts, at the cost of a small allocation.

4.  **Create a `From<AsnInfoView<'_>> for AsnInfo` Implementation:**
    *   To make the conversion cleaner, implement the `From` trait: `impl From<AsnInfoView<'_>> for AsnInfo`.
    *   The `from` method will take an `AsnInfoView` and return a new `AsnInfo`, containing the logic for converting `&str` to `String`.
    *   The `lookup_owned` method can then be simplified to `self.lookup(ip).map(Into::into)`.
