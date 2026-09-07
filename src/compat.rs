#[cfg(cups2)]
pub(crate) type CupsCount = std::os::raw::c_int;

#[cfg(cups3)]
pub(crate) type CupsCount = usize;

#[cfg(cups2)]
pub(crate) type CupsMedia = crate::bindings::cups_size_s;

#[cfg(cups3)]
pub(crate) type CupsMedia = crate::bindings::cups_media_s;

#[cfg(cups2)]
pub(crate) fn count_to_usize(count: CupsCount) -> usize {
    usize::try_from(count).unwrap_or(0)
}

#[cfg(cups3)]
pub(crate) fn count_to_usize(count: CupsCount) -> usize {
    count
}

#[cfg(cups2)]
pub(crate) fn usize_to_count(count: usize) -> CupsCount {
    std::os::raw::c_int::try_from(count).unwrap_or(std::os::raw::c_int::MAX)
}

#[cfg(cups3)]
pub(crate) fn usize_to_count(count: usize) -> CupsCount {
    count
}

pub(crate) fn empty_media() -> CupsMedia {
    unsafe { std::mem::zeroed() }
}

#[cfg(cups2)]
pub(crate) fn cups_bool(value: std::os::raw::c_int) -> bool {
    value != 0
}

#[cfg(cups3)]
pub(crate) fn cups_bool(value: bool) -> bool {
    value
}
