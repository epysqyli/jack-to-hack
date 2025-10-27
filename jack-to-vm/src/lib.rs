use std::{collections::HashMap, fs::read_to_string, path::PathBuf};

#[path = "syntax-analyzer.rs"]
mod syntax_analyzer;

#[allow(unused)]
pub fn compile(program_path: &PathBuf) {
    let jack_classes = read_jack_classes_from_fs(program_path);
    jack_classes.into_iter().for_each(|(name, content)| {
        let derivation_tree = syntax_analyzer::run(content);
        // Each derivation tree should be supplied to the code generator.
        // the final output being the intermediate representation vm code.
    });
}

fn read_jack_classes_from_fs(program_path: &PathBuf) -> HashMap<String, String> {
    /* path => content */
    let mut classes: HashMap<String, String> = HashMap::new();

    if program_path.is_file() {
        classes.insert(
            program_path.to_string_lossy().into_owned(),
            read_to_string(program_path).unwrap(),
        );

        return classes;
    }

    if let Ok(dir) = program_path.read_dir() {
        for entry in dir {
            if let Ok(dir_entry) = entry {
                if let Some(ext) = dir_entry.path().extension()
                    && ext == "jack"
                {
                    classes.insert(
                        dir_entry.path().to_string_lossy().into_owned(),
                        read_to_string(dir_entry.path()).unwrap(),
                    );
                }
            }
        }
    }

    classes
}
