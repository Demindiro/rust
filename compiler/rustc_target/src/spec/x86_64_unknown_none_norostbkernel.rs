use crate::spec::{Target, TargetOptions};

pub fn target() -> Target {
    Target {
        llvm_target: "x86_64-unknown-none".into(),
        pointer_width: 64,
        data_layout: "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
            .into(),
        arch: "x86_64".into(),
        options: TargetOptions {
            cpu: "x86-64".into(),
            max_atomic_width: Some(64),
            features:
                "-mmx,-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-3dnow,-3dnowa,-avx,-avx2,+soft-float"
                    .into(),
            disable_redzone: true,
            ..super::norostb_kernel_base::opts()
        },
    }
}
