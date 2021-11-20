pub trait VecOsStringToStringExt {
    fn to_str_vec(&self) -> Vec<&str>;
}

impl VecOsStringToStringExt for Vec<std::ffi::OsString> {
    fn to_str_vec(&self) -> Vec<&str> {
        // TODO: probs change return to result with error from e.to_str().ok_or(Err(e)) or something
        return self.iter().map(|e| e.to_str().unwrap_or("")).collect();
    }
}
