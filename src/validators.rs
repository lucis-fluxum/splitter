const VALID_BITRATE_VALUES: [&str; 16] = [
    "8k", "16k", "24k", "32k", "40k", "48k", "64k", "80k", "96k", "112k", "128k", "160k", "192k",
    "224k", "256k", "320k",
];

pub fn file_name(value: String) -> Result<(), String> {
    if value.to_lowercase().ends_with(".aax") {
        Ok(())
    } else {
        Err(String::from("The file must be an AAX file"))
    }
}

pub fn bitrate(value: String) -> Result<(), String> {
    if VALID_BITRATE_VALUES.contains(&value.as_str()) {
        Ok(())
    } else {
        Err(format!(
            "The bitrate must be one of: {:?}",
            VALID_BITRATE_VALUES
        ))
    }
}
