use crate::spec::{LinkerFlavor, LldFlavor, PanicStrategy, TargetOptions};

pub fn opts() -> TargetOptions {
    TargetOptions {
        os: "norostb".into(),
        executables: true,
        // TODO figure out why rust-lld is missing from stage2 folder and/or
        // why the one in $PATH isn't used.
        //linker: Some("rust-lld".into()),
        linker: Some("ld.lld".into()),
        linker_flavor: LinkerFlavor::Lld(LldFlavor::Ld),
        panic_strategy: PanicStrategy::Abort,
        ..Default::default()
    }
}
