use std::fs;

use rari_types::globals::content_root;

pub(crate) struct SidebarFixtures {
    do_not_remove: bool,
}

impl SidebarFixtures {
    pub fn new(data: Vec<&str>) -> Self {
        Self::new_internal(data, false)
    }

    #[allow(dead_code)]
    pub fn debug_new(data: Vec<&str>) -> Self {
        Self::new_internal(data, true)
    }

    fn new_internal(data: Vec<&str>, do_not_remove: bool) -> Self {
        let mut path = content_root().to_path_buf();
        path.push("sidebars");

        if !path.exists() {
            fs::create_dir(&path).unwrap();
        }
        for (ct, d) in data.into_iter().enumerate() {
            let name = format!("sidebar_{ct}.yaml");
            fs::write(path.join(name), d.as_bytes()).unwrap();
        }

        SidebarFixtures { do_not_remove }
    }
}

impl Drop for SidebarFixtures {
    fn drop(&mut self) {
        if self.do_not_remove {
            println!("Leaving doc fixtures in place for debugging");
            return;
        }
        // Perform cleanup actions, recursively remove all files
        // in the sidebars folder, and the sidebars folder as well
        let mut path = content_root().to_path_buf();
        path.push("sidebars");

        let entries = fs::read_dir(&path).unwrap();

        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                fs::remove_dir_all(&path).unwrap();
            } else {
                fs::remove_file(&path).unwrap();
            }
        }
        fs::remove_dir_all(&path).unwrap();
    }
}
