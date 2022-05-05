use crate::ffi::{OsStr, OsString};
use crate::fmt;
use crate::io;
use crate::num::NonZeroI32;
use crate::os::norostb::prelude::*;
use crate::path::Path;
use crate::slice::Iter;
use crate::sys::fs::File;
use crate::sys::pipe::AnonPipe;
use crate::sys::unsupported;
use crate::sys_common::process::{CommandEnv, CommandEnvs};
use norostb_rt as rt;

pub use crate::ffi::OsString as EnvKey;

////////////////////////////////////////////////////////////////////////////////
// Command
////////////////////////////////////////////////////////////////////////////////

pub struct Command {
    program: OsString,
    env: CommandEnv,
    dir: OsString,
    args: Vec<OsString>,
}

// passed back to std::process with the pipes connected to the child, if any
// were requested
pub struct StdioPipes {
    pub stdin: Option<AnonPipe>,
    pub stdout: Option<AnonPipe>,
    pub stderr: Option<AnonPipe>,
}

pub enum Stdio {
    Inherit,
    Null,
    MakePipe,
}

impl Command {
    pub fn new(program: &OsStr) -> Self {
        Self {
            program: program.into(),
            env: Default::default(),
            dir: Default::default(),
            args: Default::default(),
        }
    }

    pub fn arg(&mut self, arg: &OsStr) {
        self.args.push(arg.into());
    }

    pub fn env_mut(&mut self) -> &mut CommandEnv {
        &mut self.env
    }

    pub fn cwd(&mut self, dir: &OsStr) {
        self.dir = dir.into();
    }

    pub fn stdin(&mut self, _stdin: Stdio) {
        todo!()
    }

    pub fn stdout(&mut self, _stdout: Stdio) {
        todo!()
    }

    pub fn stderr(&mut self, _stderr: Stdio) {
        todo!()
    }

    pub fn get_program(&self) -> &OsStr {
        panic!("unsupported")
    }

    pub fn get_args(&self) -> CommandArgs<'_> {
        CommandArgs(self.args.iter())
    }

    pub fn get_envs(&self) -> CommandEnvs<'_> {
        self.env.iter()
    }

    pub fn get_current_dir(&self) -> Option<&Path> {
        None
    }

    pub fn spawn(
        &mut self,
        _default: Stdio,
        _needs_stdin: bool,
    ) -> io::Result<(Process, StdioPipes)> {
        rt::Process::new(
            &*rt::io::process_root().ok_or(super::ERR_UNSET)?,
            &rt::io::file_root()
                .ok_or(super::ERR_UNSET)?
                .open(self.program.as_bytes())
                .map_err(super::cvt_err)?,
            rt::process::Process::default_handles(),
            self.args.iter().map(|s| s.as_bytes()),
            self.env.capture().iter().map(|(k, v)| (k.as_bytes(), v.as_bytes())),
        )
        .map_err(super::cvt_err)
        .map(|p| (Process(p), StdioPipes { stdin: None, stdout: None, stderr: None }))
    }
}

impl From<AnonPipe> for Stdio {
    fn from(pipe: AnonPipe) -> Stdio {
        pipe.diverge()
    }
}

impl From<File> for Stdio {
    fn from(_file: File) -> Stdio {
        panic!("unsupported")
    }
}

impl fmt::Debug for Command {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

pub struct ExitStatus(!);

impl ExitStatus {
    pub fn exit_ok(&self) -> Result<(), ExitStatusError> {
        self.0
    }

    pub fn code(&self) -> Option<i32> {
        self.0
    }
}

impl Clone for ExitStatus {
    fn clone(&self) -> ExitStatus {
        self.0
    }
}

impl Copy for ExitStatus {}

impl PartialEq for ExitStatus {
    fn eq(&self, _other: &ExitStatus) -> bool {
        self.0
    }
}

impl Eq for ExitStatus {}

impl fmt::Debug for ExitStatus {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0
    }
}

impl fmt::Display for ExitStatus {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct ExitStatusError(ExitStatus);

impl Into<ExitStatus> for ExitStatusError {
    fn into(self) -> ExitStatus {
        self.0.0
    }
}

impl ExitStatusError {
    pub fn code(self) -> Option<NonZeroI32> {
        self.0.0
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct ExitCode(bool);

impl ExitCode {
    pub const SUCCESS: ExitCode = ExitCode(false);
    pub const FAILURE: ExitCode = ExitCode(true);

    pub fn as_i32(&self) -> i32 {
        self.0 as i32
    }
}

impl From<u8> for ExitCode {
    fn from(code: u8) -> Self {
        match code {
            0 => Self::SUCCESS,
            1..=255 => Self::FAILURE,
        }
    }
}

pub struct Process(rt::Process);

impl Process {
    pub fn id(&self) -> u32 {
        self.0.as_object().as_raw()
    }

    pub fn kill(&mut self) -> io::Result<()> {
        unsupported()
    }

    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        unsupported()
    }

    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        unsupported()
    }
}

pub struct CommandArgs<'a>(Iter<'a, OsString>);

impl<'a> Iterator for CommandArgs<'a> {
    type Item = &'a OsStr;
    fn next(&mut self) -> Option<&'a OsStr> {
        self.0.next().map(|s| &**s)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a> ExactSizeIterator for CommandArgs<'a> {}

impl<'a> fmt::Debug for CommandArgs<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().finish()
    }
}
