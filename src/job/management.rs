use super::status::{JobInfo, JobStatus};
use crate::bindings;
use crate::constants::WHICHJOBS_ALL;
use crate::error::{Error, Result};
use std::ffi::CString;
use std::ptr;

pub fn cancel_job(job_id: i32) -> Result<()> {
    let destinations = crate::get_all_destinations()?;

    for dest in destinations {
        let dest_ptr = dest.as_ptr();
        if dest_ptr.is_null() {
            continue;
        }

        let status = unsafe { bindings::cupsCancelDestJob(ptr::null_mut(), dest_ptr, job_id) };

        unsafe {
            let dest_box = Box::from_raw(dest_ptr);
            if !dest_box.name.is_null() {
                let _ = CString::from_raw(dest_box.name);
            }
            if !dest_box.instance.is_null() {
                let _ = CString::from_raw(dest_box.instance);
            }
            if !dest_box.options.is_null() {
                bindings::cupsFreeOptions(dest_box.num_options, dest_box.options);
            }
        }

        if status == bindings::ipp_status_e_IPP_STATUS_OK as bindings::ipp_status_t {
            return Ok(());
        }
    }

    Err(Error::JobManagementFailed(format!(
        "Failed to cancel job {} on any destination",
        job_id
    )))
}

pub fn get_jobs(dest_name: Option<&str>) -> Result<Vec<JobInfo>> {
    get_jobs_with_filter(dest_name, WHICHJOBS_ALL)
}

pub fn get_active_jobs(dest_name: Option<&str>) -> Result<Vec<JobInfo>> {
    get_jobs_with_filter(dest_name, crate::constants::WHICHJOBS_ACTIVE)
}

pub fn get_completed_jobs(dest_name: Option<&str>) -> Result<Vec<JobInfo>> {
    get_jobs_with_filter(dest_name, crate::constants::WHICHJOBS_COMPLETED)
}

fn get_jobs_with_filter(dest_name: Option<&str>, which_jobs: i32) -> Result<Vec<JobInfo>> {
    let dest_name_c = match dest_name {
        Some(name) => Some(CString::new(name)?),
        None => None,
    };

    let dest_ptr = match &dest_name_c {
        Some(name) => name.as_ptr(),
        None => ptr::null(),
    };

    let mut jobs_ptr: *mut bindings::cups_job_s = ptr::null_mut();
    let num_jobs =
        unsafe { bindings::cupsGetJobs2(ptr::null_mut(), &mut jobs_ptr, dest_ptr, 0, which_jobs) };

    if num_jobs < 0 {
        return Ok(Vec::new());
    }

    if jobs_ptr.is_null() {
        return Ok(Vec::new());
    }

    let mut job_infos = Vec::with_capacity(num_jobs as usize);

    for i in 0..num_jobs {
        unsafe {
            let job = &*(jobs_ptr.offset(i as isize));

            let title = if job.title.is_null() {
                String::new()
            } else {
                std::ffi::CStr::from_ptr(job.title)
                    .to_string_lossy()
                    .into_owned()
            };

            let user = if job.user.is_null() {
                String::new()
            } else {
                std::ffi::CStr::from_ptr(job.user)
                    .to_string_lossy()
                    .into_owned()
            };

            let dest = if job.dest.is_null() {
                String::new()
            } else {
                std::ffi::CStr::from_ptr(job.dest)
                    .to_string_lossy()
                    .into_owned()
            };

            job_infos.push(JobInfo {
                id: job.id,
                title,
                user,
                dest,
                status: JobStatus::from_cups_state(job.state as i32),
                size: job.size,
                priority: job.priority,
                creation_time: job.creation_time as i64,
                processing_time: job.processing_time as i64,
                completed_time: job.completed_time as i64,
            });
        }
    }

    unsafe {
        if !jobs_ptr.is_null() {
            bindings::cupsFreeJobs(num_jobs, jobs_ptr);
        }
    }

    Ok(job_infos)
}

pub fn get_job_info(job_id: i32) -> Result<JobInfo> {
    let jobs = get_jobs(None)?;

    jobs.into_iter()
        .find(|job| job.id == job_id)
        .ok_or_else(|| {
            let active_jobs = get_active_jobs(None).unwrap_or_default();
            let completed_jobs = get_completed_jobs(None).unwrap_or_default();
            Error::JobManagementFailed(format!(
                "Job {} not found (active: {}, completed: {})",
                job_id,
                active_jobs.len(),
                completed_jobs.len()
            ))
        })
}
