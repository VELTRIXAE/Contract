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
