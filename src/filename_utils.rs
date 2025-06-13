use std::{
    ffi::{OsStr, OsString},
    path::Path,
};

/// If no extension is present, appends to the end to make distinct file names.
/// TODO: Not extensively tested against imagemagick and might diverge in edge cases.
pub fn insert_suffix_before_extension_in_path(
    os_path_string: &OsStr,
    suffix_to_insert: &OsStr,
) -> OsString {
    let path = Path::new(os_path_string);

    if let Some(filename_os_str) = path.file_name() {
        let filename_path_view = Path::new(filename_os_str);

        if let Some(ext_os_str) = filename_path_view.extension() {
            let stem_os_str = filename_path_view
                .file_stem()
                .unwrap_or_else(|| OsStr::new(""));

            let mut new_filename = OsString::new();
            new_filename.push(stem_os_str);
            new_filename.push(suffix_to_insert);
            new_filename.push(".");
            new_filename.push(ext_os_str);

            if let Some(parent_dir) = path.parent() {
                // If the parent directory is empty (e.g. for relative paths like "./file.txt"),
                // Path::join might produce just the new_filename.
                // If parent_dir is empty AND PathBuf is empty, it returns new_filename.
                // If parent_dir is "." it becomes "./new_filename".
                // This conversion is key: PathBuf -> OsString
                parent_dir.join(new_filename).into_os_string()
            } else {
                new_filename
            }
        } else {
            let mut result = os_path_string.to_owned();
            result.push(suffix_to_insert);
            result
        }
    } else {
        let mut result = os_path_string.to_owned();
        result.push(suffix_to_insert);
        result
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::*;

    // TODO: This is actually just generic sanity check and isn't extensively tested against imagemagick
    #[test]
    fn append_number_suffix() {
        let test_cases = vec![
            // Simple filenames
            ("filename.txt", "filename-1.txt"),
            ("archive.tar.gz", "archive.tar-1.gz"),
            ("nodotfile", "nodotfile-1"),
            //(".bashrc", "-1.bashrc"), ??
            //("filewithtrailingdot.", "filewithtrailingdot.-1"), ??
            ("..hidden_file.txt", "..hidden_file-1.txt"),
        ];

        for (input_str, expected_str) in test_cases {
            let input_os_string = OsString::from(input_str);
            let expected_os_string = OsString::from(expected_str);
            let result_os_string =
                insert_suffix_before_extension_in_path(&input_os_string, OsStr::new("-1"));

            assert_eq!(
                result_os_string, expected_os_string,
                "Test failed for input: {}",
                input_str
            );
        }
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn append_number_suffix_unix() {
        // This really isn't expected to fail and these tests are only going the extra mile.
        // I cannot be bothered to write tests with windows separators.
        // If you're looking at this and it's causing problems, just delete it.
        let unix_test_cases = vec![
            // Paths with folders
            ("some_folder/filename.txt", "some_folder/filename-1.txt"),
            ("some_folder/archive.tar.gz", "some_folder/archive.tar-1.gz"),
            ("some_folder/nodotfile", "some_folder/nodotfile-1"),
            //("some_folder/.bashrc", "some_folder/-1.bashrc"), ??
            //("some_folder/filewithtrailingdot.", "some_folder/filewithtrailingdot."), ??
            ("foo/bar/baz.longext", "foo/bar/baz-1.longext"),
            ("a/b/.hidd.en", "a/b/.hidd-1.en"),
        ];

        for (input_str, expected_str) in unix_test_cases {
            let input_os_string = OsString::from(input_str);
            let expected_os_string = OsString::from(expected_str);
            let result_os_string =
                insert_suffix_before_extension_in_path(&input_os_string, OsStr::new("-1"));

            assert_eq!(
                result_os_string, expected_os_string,
                "Test failed for input: {}",
                input_str
            );
        }

        // Example with OsString that might not be valid UTF-8 (on Unix)
        use std::os::unix::ffi::OsStringExt;
        let invalid_dir_name_bytes = vec![0x64, 0xff, 0x72]; // "d\xFFr"
        let invalid_file_stem_bytes = vec![0x66, 0xfe, 0x6c, 0x65]; // "f\xFEle"
        let extension_and_dot_bytes = vec![b'.', 0x74, 0x78, 0x74]; // ".txt"

        let mut path_bytes = Vec::new();
        path_bytes.extend_from_slice(&invalid_dir_name_bytes);
        path_bytes.push(b'/');
        path_bytes.extend_from_slice(&invalid_file_stem_bytes);
        path_bytes.extend_from_slice(&extension_and_dot_bytes);
        let non_utf8_path = OsString::from_vec(path_bytes.clone());

        let mut expected_path_bytes = Vec::new();
        expected_path_bytes.extend_from_slice(&invalid_dir_name_bytes);
        expected_path_bytes.push(b'/');
        expected_path_bytes.extend_from_slice(&invalid_file_stem_bytes);
        expected_path_bytes.extend_from_slice(b"-1");
        expected_path_bytes.extend_from_slice(&extension_and_dot_bytes);
        let expected_non_utf8_path = OsString::from_vec(expected_path_bytes);

        let result_non_utf8 =
            insert_suffix_before_extension_in_path(&non_utf8_path, OsStr::new("-1"));
        println!(
            "Input (non-UTF8): {:?}, Expected: {:?}, Got: {:?}",
            non_utf8_path, expected_non_utf8_path, result_non_utf8
        );
        assert_eq!(
            result_non_utf8, expected_non_utf8_path,
            "Test for non-UTF8 path"
        );
    }
}
