use crate::spec::Target;

pub fn target() -> Target {
	// TODO copied from kernel config. Relax when kernel starts handling certain
	// features properly.

	let mut base = super::norostb_base::opts();
    base.cpu = "x86-64".into();
	base.disable_redzone = true;
	base.features = "-mmx,-sse,+soft-float".into();
    base.max_atomic_width = Some(64);

    Target {
        llvm_target: "x86_64-unknown-none".into(),
        pointer_width: 64,
		data_layout: "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128".into(),
		// Causes ICE because it doesn't match LLVM's layout.
        //data_layout: "e-m:e-i64:64-f80:128-n8:16:32:64-S128".into(),
        arch: "x86_64".into(),
        options: base,
    }
}
