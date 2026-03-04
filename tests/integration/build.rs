/// Build script for integration-tests.
///
/// Previously this script patched WASM binaries produced by the release build
/// to normalise padded `call_indirect` reserved bytes.  Integration tests now
/// use native contract registration (`Env::register_contract`) instead of
/// WASM binary loading, so no patching is required.
fn main() {}
