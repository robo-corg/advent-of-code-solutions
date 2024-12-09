use std::io::{self, BufRead};

#[derive(Debug, Clone, Copy)]

enum SafeReason {
    SafeIncreasing,
    SafeDecreasing,
}

#[derive(Debug, Clone, Copy)]
struct Safe {
    removed: Option<usize>,
    reason: SafeReason,
}

#[derive(Debug, Clone, Copy)]
enum UnsafeReason {
    NoDiffer,
    TooMuchDiffer,
    IncDecMix,
}

#[derive(Debug, Clone, Copy)]
struct Unsafe {
    pos: usize,
    reason: UnsafeReason,
}

#[derive(Debug, Clone, Copy)]
enum Status {
    Safe(Safe),
    Unsafe(Unsafe),
}

impl Status {
    fn is_safe(&self) -> bool {
        match self {
            Status::Safe(_) => true,
            Status::Unsafe(_) => false,
        }
    }
}

#[derive(Debug, Clone)]
struct Report(Vec<i32>);

impl Report {
    fn from_str(s: &str) -> anyhow::Result<Self> {
        let readings_s = s.split(' ');

        let readings: Vec<i32> = readings_s
            .map(|reading_s| i32::from_str_radix(reading_s, 10).unwrap())
            .collect();

        Ok(Report(readings))
    }

    fn remove(&self, n: usize) -> Self {
        let mut ret = self.clone();
        ret.0.remove(n);
        ret
    }

    fn status(&self) -> Status {
        let mut maybe_dir_inc = None;

        for n in 0..(self.0.len() - 1) {
            let m = n + 1;
            let d = self.0[m] - self.0[n];

            if d == 0 {
                return Status::Unsafe(Unsafe {
                    reason: UnsafeReason::NoDiffer,
                    pos: n,
                });
            } else if i32::abs(d) > 3 {
                return Status::Unsafe(Unsafe {
                    reason: UnsafeReason::TooMuchDiffer,
                    pos: n,
                });
            }

            let cur_dir_inc = d > 0;

            if maybe_dir_inc.is_some() && maybe_dir_inc != Some(cur_dir_inc) {
                return Status::Unsafe(Unsafe {
                    reason: UnsafeReason::IncDecMix,
                    pos: n - 1,
                });
            } else {
                maybe_dir_inc = Some(cur_dir_inc);
            }
        }

        Status::Safe(Safe {
            removed: None,
            reason: match maybe_dir_inc.unwrap() {
                true => SafeReason::SafeIncreasing,
                false => SafeReason::SafeDecreasing,
            },
        })
    }

    fn status_with_dampen(&self) -> Status {
        let initial_status = self.status();

        match initial_status {
            Status::Unsafe(Unsafe { pos, reason }) => {
                let edits = match reason {
                    UnsafeReason::NoDiffer | UnsafeReason::TooMuchDiffer => vec![pos, pos + 1],
                    UnsafeReason::IncDecMix => vec![pos, pos + 1, pos + 2],
                };

                for edit in edits {
                    let report = self.remove(edit);
                    let status = report.status();

                    if let Status::Safe(Safe {
                        removed: None,
                        reason,
                    }) = status
                    {
                        return Status::Safe(Safe {
                            removed: Some(edit),
                            reason,
                        });
                    }
                }
            }
            _ => {}
        }

        initial_status
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = io::stdin();

    let mut reports = Vec::new();

    for maybe_line in stdin.lock().lines() {
        let line = maybe_line?;
        reports.push(Report::from_str(&line)?);
    }

    let safe_count = reports
        .iter()
        .map(|r| r.status())
        // .inspect(|s| {
        //     dbg!(s);
        // })
        .filter(Status::is_safe)
        .count();

    println!("Safe count: {}", safe_count);

    let safe_with_dampen_count = reports
        .iter()
        .map(|r| r.status_with_dampen())
        .inspect(|s| {
            dbg!(s);
        })
        .filter(Status::is_safe)
        .count();


    println!("Safe with dampen count: {}", safe_with_dampen_count);


    Ok(())
}
