use crate::spec::{LinkerFlavor, LldFlavor, TargetOptions, LinkArgs, PanicStrategy};

pub fn opts() -> TargetOptions {
	let mut pre_link_args = LinkArgs::new();
	pre_link_args.insert(LinkerFlavor::Lld(LldFlavor::Ld), Vec::from(["/tmp/hello_std/rtbegin.o".into()]));
    TargetOptions {
        os: "norostb".into(),
        executables: true,
        // TODO figure out why rust-lld is missing from stage2 folder and/or
        // why the one in $PATH isn't used.
        //linker: Some("rust-lld".into()),
        linker: Some("ld.lld".into()),
        linker_flavor: LinkerFlavor::Lld(LldFlavor::Ld),
		pre_link_args,
		panic_strategy: PanicStrategy::Abort,
        ..Default::default()
    }
}
