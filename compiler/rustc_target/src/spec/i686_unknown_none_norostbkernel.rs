use crate::spec::{Target, TargetOptions};

pub fn target() -> Target {
    Target {
        llvm_target: "i686-unknown-none".into(),
        pointer_width: 32,
        data_layout:
            "e-m:e-p:32:32-p270:32:32-p271:32:32-p272:64:64-f64:32:64-f80:32-n8:16:32-S128".into(),
        arch: "x86".into(),
        options: TargetOptions {
            cpu: "i686".into(),
            max_atomic_width: Some(32),
            features:
                "-mmx,-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-3dnow,-3dnowa,-avx,-avx2,+soft-float"
                    .into(),
            disable_redzone: true,
            ..super::norostb_kernel_base::opts()
        },
    }
}
