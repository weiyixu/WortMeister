//
//  WortMeister-Bridging-Header.h
//  Exposes the Rust core C API to Swift. In Xcode, set this file as the
//  target's "Objective-C Bridging Header" (Build Settings ->
//  SWIFT_OBJC_BRIDGING_HEADER). The header itself lives in the Rust crate at
//  core/include so it stays in sync with src/ffi.rs.
//

#import "wortmeister_core.h"
