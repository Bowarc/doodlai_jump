use std::io::Write;

#[derive(Debug, Clone)]
pub struct Path {
    p: std::borrow::Cow<'static, str>,
}

impl Path {
    pub const fn new(p: &'static str) -> Self {
        Self {
            p: std::borrow::Cow::Borrowed(p),
        }
    }

    pub fn new_owned(p: String) -> Self {
        Self {
            p: std::borrow::Cow::Owned(p),
        }
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.p)
    }
}

fn external_path(p: &str) -> String {
    if let Ok(cargo_manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        format!("{}/resources/external/{}", cargo_manifest_dir, p)
    } else {
        let Ok(mut exe_path) = std::env::current_exe() else {
            panic!("Failed to get current executable path");
        };
        exe_path.pop();

        format!("{}/resources/external/{}", exe_path.display(), p)
    }
    .replace("\\", "/")
    .replace("//", "/")
}

pub fn try_bytes(path: impl Into<Path>) -> Result<std::borrow::Cow<'static, [u8]>, std::io::Error> {
    use std::io::Read as _;
    let path = path.into();

    let stopwatch = time::Stopwatch::start_new();
    let start_info_message = format!("Loading /{}", path.p);

    let complete_path = external_path(&path.p);

    match std::fs::File::open(complete_path) {
        Ok(mut file) => {
            let mut bytes: Vec<u8> = Vec::new();
            let _ = file.read_to_end(&mut bytes);
            trace!("{start_info_message} . . success in {stopwatch}");
            Ok(bytes.into())
        }
        Err(e) => {
            // format!("Could not open path: {:?}, {}", path.fs, path.p);
            // error!("{} . . error: {e}", start_info_message);
            Err(e)
        }
    }
}

pub fn bytes(path: impl Into<Path>) -> std::borrow::Cow<'static, [u8]> {
    try_bytes(path).unwrap()
}

pub fn write(path: impl Into<Path>, data: &[u8]) {
    let path = path.into();

    let p = external_path(&path.p);

    trace!("Writing to {p}");

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .read(false)
        .open(&p)
        .unwrap_or(
            std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .read(false)
                .open(p)
                .unwrap(),
        );

    file.write_all(data).unwrap();
}
