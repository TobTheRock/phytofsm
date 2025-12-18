use crate::error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FilePath(std::path::PathBuf);

impl std::fmt::Display for FilePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string_lossy())
    }
}

impl FilePath {
    /// Creates a FilePath, if relative, it will be resolved to an absolute path given by the spans
    /// location
    pub fn resolve(file_path: &str, span: proc_macro::Span) -> Self {
        let file_path = std::path::PathBuf::from(file_path.trim_matches('"'));
        if file_path.is_absolute() {
            return Self(file_path);
        }

        let caller_file = span.local_file().unwrap_or_default();
        let caller_dir = caller_file.parent().unwrap_or(std::path::Path::new("."));

        let path = caller_dir.join(file_path);
        Self(path)
    }
}

pub struct FsmFile {
    content: String,
}

impl FsmFile {
    pub fn try_open(file_path: FilePath) -> error::Result<Self> {
        let error =
            |e: std::io::Error| error::Error::InvalidFile(file_path.to_string(), e.to_string());
        let content = std::fs::read_to_string(&file_path.0).map_err(error)?;

        Ok(Self { content })
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

#[cfg(test)]
mod test {
    use crate::{
        file::{FilePath, FsmFile},
        test,
    };

    // TODO use procmacro2 and reenable the test
    // #[test]
    // fn test_abs_file_path_from_relative() {
    //     let file_path = file!();
    //     let expected = std::fs::canonicalize(file_path).expect("Failed to canonicalize path");
    //     let span = proc_macro2::Span::call_site();
    //
    //     let abs_file_path =
    //         FilePath::try_resolve(file_path, span).expect("Failed to create AbsFilePath");
    //
    //     assert_eq!(abs_file_path.0, dbg!(expected));
    // }
    //
    #[test]
    fn open_file() {
        let test_data = test::FsmTestData::four_seasons();

        // TODO use the actual method
        let file_path = FilePath(test_data.path);
        let fsm_file = FsmFile::try_open(file_path).expect("Failed to open FSM file");
        assert!(
            !fsm_file.content.is_empty(),
            "FSM file content should not be empty"
        );
    }
}
