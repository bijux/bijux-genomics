use super::{
    lifecycle::{terminate, ChildGuard},
    SupervisedOutput,
};
use anyhow::{Context, Result};
use rustix::event::{poll, PollFd, PollFlags, Timespec};
use rustix::fs::{fcntl_getfl, fcntl_setfl, OFlags};
use std::io::{ErrorKind, Read};
use std::process::Child;
use std::time::{Duration, Instant};
use wait_timeout::ChildExt as _;

pub(super) fn collect(
    child: &mut Child,
    timeout: Duration,
    max_output_bytes: Option<usize>,
    label: &str,
) -> Result<SupervisedOutput> {
    let mut owned = ChildGuard { child, armed: true };
    let deadline =
        Instant::now().checked_add(timeout).context("process collection deadline overflow")?;
    let mut stdout = owned.child.stdout.take();
    let mut stderr = owned.child.stderr.take();
    if let Some(pipe) = &stdout {
        fcntl_setfl(pipe, fcntl_getfl(pipe)? | OFlags::NONBLOCK)?;
    }
    if let Some(pipe) = &stderr {
        fcntl_setfl(pipe, fcntl_getfl(pipe)? | OFlags::NONBLOCK)?;
    }
    let mut output = Vec::new();
    let mut errors = Vec::new();
    let mut timed_out = false;
    let mut output_limit_exceeded = false;
    let mut remaining = max_output_bytes;
    while stdout.is_some() || stderr.is_some() {
        let Some(duration) = deadline.checked_duration_since(Instant::now()) else {
            timed_out = true;
            break;
        };
        let mut descriptors = Vec::with_capacity(2);
        if let Some(pipe) = &stdout {
            descriptors.push(PollFd::new(pipe, PollFlags::IN));
        }
        if let Some(pipe) = &stderr {
            descriptors.push(PollFd::new(pipe, PollFlags::IN));
        }
        let interval = Timespec::try_from(duration)?;
        match poll(&mut descriptors, Some(&interval)) {
            Ok(0) => {
                timed_out = true;
                break;
            }
            Err(rustix::io::Errno::INTR) => continue,
            Err(error) => return Err(error).with_context(|| format!("poll output for {label}")),
            Ok(_) => {}
        }
        drop(descriptors);
        if let Some(pipe) = &mut stdout {
            if drain(pipe, &mut output, deadline, &mut remaining, &mut output_limit_exceeded)? {
                stdout = None;
            }
        }
        if output_limit_exceeded {
            break;
        }
        if let Some(pipe) = &mut stderr {
            if drain(pipe, &mut errors, deadline, &mut remaining, &mut output_limit_exceeded)? {
                stderr = None;
            }
        }
        if output_limit_exceeded {
            break;
        }
    }
    let status = if timed_out || output_limit_exceeded {
        terminate(owned.child, label)?
    } else if let Some(status) =
        owned.child.wait_timeout(deadline.saturating_duration_since(Instant::now()))?
    {
        status
    } else {
        timed_out = true;
        terminate(owned.child, label)?
    };
    owned.armed = false;
    Ok(SupervisedOutput {
        status,
        stdout: output,
        stderr: errors,
        timed_out,
        output_limit_exceeded,
    })
}

fn drain(
    pipe: &mut impl Read,
    bytes: &mut Vec<u8>,
    deadline: Instant,
    remaining: &mut Option<usize>,
    exceeded: &mut bool,
) -> Result<bool> {
    let mut buffer = [0_u8; 8192];
    while Instant::now() < deadline {
        match pipe.read(&mut buffer) {
            Ok(0) => return Ok(true),
            Ok(count) => {
                let admitted = remaining.map_or(count, |budget| budget.min(count));
                bytes.extend_from_slice(&buffer[..admitted]);
                if let Some(budget) = remaining {
                    *budget -= admitted;
                }
                if admitted < count {
                    *exceeded = true;
                    return Ok(false);
                }
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => return Ok(false),
            Err(error) if error.kind() == ErrorKind::Interrupted => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(false)
}
