# Changelog

##  v0.3.0

### Breaking changes
- Upgraded `rand` to v0.10, exposed in the public API.
- Set MSRV to 1.71.0.

### Performance
- Use `#[inline(always)]` for a 20% speed improvement on residue computations.
