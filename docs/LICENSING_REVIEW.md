# Licensing evidence and unresolved questions

Engineering inventory reviewed 2026-11-04. This records repository declarations
and provenance; it is not a legal determination of distribution rights.

## Preserved material

- `LICENSE`: VELTRIXAE MIT license and copyright notice, unchanged.
- `LICENSE-APACHE`: Apache License 2.0 text, unchanged.
- `NOTICE`: VELTRIXAE copyright, Soroswap/PaltaLabs attribution, exact imported
  commit and Uniswap V2 design lineage, unchanged.
- Contract manifests retain existing original-author attribution and license
  declarations; no upstream authorship or AMM originality claim was added.
- The independent spec-tool declares Apache-2.0. VELTRIXAE-authored scripts and
  documentation use the repository MIT license where applicable.

## Exact upstream evidence

Imported source is attributed to Soroswap Core commit
[`6eade008ee6794075ad941609e0fef0d1124d3eb`](https://github.com/soroswap/core/tree/6eade008ee6794075ad941609e0fef0d1124d3eb).
At that tree, root `LICENSE` is Apache-2.0, while
[`contracts/library/Cargo.toml`](https://github.com/soroswap/core/blob/6eade008ee6794075ad941609e0fef0d1124d3eb/contracts/library/Cargo.toml)
explicitly declares `GPL-3.0` and names esteblock as author. Upstream also
contains `contracts/factory/UniswapV2/LICENSE` alongside Solidity reference
sources; that reference subtree was not imported into this repository.

The local library inherits the workspace's existing Apache-2.0 declaration.
That declaration does not itself resolve the conflicting upstream GPL manifest.
Using vendored source instead of the published soroswap-library package does
not establish relicensing permission. No license declaration was changed in
this review to imply the conflict has been settled.

Reproduce without relying on a current upstream branch:

```bash
git -C /tmp/soroswap-review show 6eade008ee6794075ad941609e0fef0d1124d3eb:LICENSE
git -C /tmp/soroswap-review show 6eade008ee6794075ad941609e0fef0d1124d3eb:contracts/library/Cargo.toml
git -C /tmp/soroswap-review ls-tree -r --name-only 6eade008ee6794075ad941609e0fef0d1124d3eb
# Confirm preserved local license/notice texts against the review baseline:
git diff 499fef4 -- LICENSE LICENSE-APACHE NOTICE
```

## Maintainer blocker

Obtain written upstream clarification covering the exact library source and/or
qualified licensing advice before making an unambiguous distribution/commercial
release claim. Determine applicable notice, source and license obligations from
that clarification. Preserve existing texts and attribution meanwhile. This
engineering review does not choose between conflicting legal interpretations.

## Exact implicated files and artifact edges (PR #9 follow-up)

The following imported files fall within the library's conflicting declarations;
this is a provenance inventory, not a determination of each file's final license:

| Category | Local paths |
|---|---|
| Production Rust | `contracts/library/src/lib.rs`, `error.rs`, `math.rs`, `quotes.rs`, `reserves.rs`, `tokens.rs` (all under the same `src/`) |
| Imported tests | `contracts/library/src/test.rs`, `contracts/library/src/test/get.rs`, `quote.rs`, `tokens.rs` |
| Existing notices/metadata | `contracts/library/Cargo.toml`, `contracts/library/README.md` |

Upstream README line 51 and the unchanged local library README explicitly state
GPL-3.0. This is stronger evidence than a manifest field alone. No per-Rust-file
copyright/license headers were found in those library sources. That absence
resolves nothing and must not be treated as a permissive grant. The newly added
`contracts/library/tests/amm_properties.rs` is VELTRIXAE test work rather than an
imported file; it does not settle the imported implementation's licensing.

The published `soroswap-library 0.3.0` archive was downloaded without executing
it. Both Cargo.toml (line 29) and Cargo.toml.orig (line 9) declare GPL-3.0;
README line 51 carries the same notice. The archive has no separate LICENSE,
NOTICE or COPYING entry. Its SHA-256 is
`26352158999f8f2bbbc70481c0218d6e9cd8cc39f1d3eaa54a133e6b914f4dec`,
matching the checksum recorded in the exact upstream router Cargo.lock. Its
original manifest directly requires soroban-sdk 20.2.0 and num-integer 0.1.45
(with i128, no default features). These differ from the current stable SDK 22
path-library graph. The
upstream router manifest depends on that published 0.3.0 crate; the in-repository
upstream library manifest describes version 2.0.0. Do not conflate those versions.

Current VELTRIXAE Cargo.lock contains no soroswap-library registry dependency.
Instead `contracts/router/Cargo.toml` links path dependency `veltrixae-library`,
which is built into router WASM as an rlib dependency. The local library is not
itself a default standalone WASM artifact. Factory/pair have no library dependency
edge. Generated clients contain interface specs; this review makes no automatic
license-propagation conclusion about those specs. Their treatment also needs a
provenance-based legal assessment where relevant.

## Possible compliant paths — require rightsholder/legal review

These are options to evaluate, not findings that permission has been established:

1. Obtain written clarification or an explicit compatible grant from all relevant
   rightsholders, covering the exact imported library source/version and its
   README/manifest conflict. Retain evidence and required notices; an upstream
   metadata edit alone must not be assumed to resolve historical rights.
2. If GPLv3 governs, assess a GPLv3-compliant distribution of the affected combined
   work, including applicable license text, corresponding-source/build material
   and preserved Apache/MIT notices. Determine scope and exact GPL version with
   qualified advice before changing declarations or release packaging.
3. Replace the uncleared implementation using independently established
   permissively licensed provenance and an appropriate review process. Retain
   required attribution/history for retained code and the Soroswap/Uniswap design
   lineage; validate API, arithmetic and behavior. Renaming or vendoring alone
   is not this solution.
4. Defer distribution/release claims for affected artifacts while clarification
   is pending. This question is not limited to commercial releases.

The [Apache Software Foundation's compatibility discussion](https://www.apache.org/licenses/GPL-compatibility.html)
recognizes incorporation of Apache-2.0 code into GPLv3 works. Therefore the issue
here is conflicting grant/declaration evidence and applicable obligations, not
an assertion that Apache-2.0 and GPLv3 can never be combined. This review does
not select a legal path, remove notices, add GPL terms to third-party code, or
claim a right to relicense it.

Current library direct declarations resolve to soroban-sdk 22.0.11 (declared
Apache-2.0) and num-integer 0.1.47 (MIT OR Apache-2.0). The dev-only proptest
1.11.0 declares MIT OR Apache-2.0. These dependency declarations do not resolve
the imported library's own README/manifest conflict or waive its notices.

## File-by-file matrix and wider scope (PR #10 follow-up)

[PR10_LICENSING_SECURITY_REVIEW.md](review/PR10_LICENSING_SECURITY_REVIEW.md) has the
per-file provenance and remediation matrix. It also adds a question this inventory
did not cover: the absence of a library dependency edge does not address the GPL-3.0
Uniswap V2 lineage of the router, pair and factory, which should also go to counsel.
No properly licensed upstream version or drop-in permissive replacement was found.
