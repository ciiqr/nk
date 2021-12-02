pub trait VecOsStringToStringExt {
    fn to_str_vec(&self) -> Result<Vec<&str>, String>;
}

impl VecOsStringToStringExt for Vec<std::ffi::OsString> {
    fn to_str_vec(&self) -> Result<Vec<&str>, String> {
        self.iter()
            .map(|e| e.to_str())
            .map(|e| e.ok_or_else(|| "Invalid UTF-8 characters in OsString of to_str_vec".into()))
            .collect()
    }
}
