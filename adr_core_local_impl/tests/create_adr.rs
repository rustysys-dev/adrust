use cucumber::{after, before, cucumber};

extern crate directories;
use directories::ProjectDirs;

use std::fs;

pub struct AdrNames {
    name: String,
    by: String,
}

impl cucumber::World for AdrNames {}

impl std::default::Default for AdrNames {
    fn default() -> AdrNames {
        AdrNames {
            name: "default-name".to_string(),
            by: "default-name".to_string(),
        }
    }
}

mod helper {
    use std::io::{self};
    extern crate directories;
    use directories::ProjectDirs;

    pub fn create_decision(name: &str) -> io::Result<(bool)> {
        let project_dirs: ProjectDirs = match ProjectDirs::from("murex", "adrust-tool", "test") {
            None => panic!("issue while preparing test"),
            Some(project_dirs) => project_dirs,
        };

        let mut cfg: adr_config::config::AdrToolConfig = adr_config::config::get_config();
        cfg.use_id_prefix = false;

        Ok(adr_core::adr_repo::create_adr(
            cfg, 
            name,
            project_dirs.cache_dir().join("templates/adr-template-v0.1.adoc").as_path(),
            project_dirs.cache_dir().join("src").as_path(),
        )
        .unwrap())
    }
}

mod create_new_adr_steps {
    use cucumber::steps;
    use std::fs;
    extern crate directories;
    use directories::ProjectDirs;

    extern crate adr_core;
    use crate::helper;

    steps! (crate::AdrNames => {
        given regex r"^A decision (.+) I need to make$" (String) |adr, name, _step| {
            adr.name = name.to_string();
        };

        when "I create a new Decision Record" |_adr, _step| {

        };

        then regex r"A new file named (.+) is created$" (String) |adr, name, _step| {   
            let is_created = helper::create_decision(&adr.name).unwrap();
            assert_eq!(is_created, true);
            
            // TODO there is certainly a way to return project_dir as part of create_decision
            let project_dirs: ProjectDirs = match ProjectDirs::from("murex", "adrust-tool", "test") {
                None => panic!("issue while preparing test"),
                Some(project_dirs) => project_dirs
            };
            let t = project_dirs.cache_dir().join(
                format!("src/{}.adoc", name.to_string())
            );
            //
            let expected_path = t.as_path();
            assert_eq!(expected_path.exists(), true);
            //
            let content: String = fs::read_to_string(expected_path).unwrap();
            assert!(content.contains("{wip}"));
        };
    });
}

mod create_already_adr_steps {
    use cucumber::steps;
    extern crate adr_core;
    use crate::helper;

    steps! (crate::AdrNames => {
        given regex r"^A new decision (.+) that already exists$" (String) |adr, name, _step| {
            adr.name = name.to_string();
            //the decision should already exist so we create it. 
            helper::create_decision(&adr.name).unwrap();
        };

        when "I create a new ADR" |_adr, _step| {
            
        };

        then "The creation fails" | adr, _step | {
            //create the same file 
            let is_created = helper::create_decision(&adr.name).unwrap();
            assert_eq!(is_created, false);
        };
    });
}

mod update_adr_decided_steps {
    use cucumber::steps;
    use std::fs;
    use std::fs::File;
    use std::io::prelude::*;

    extern crate directories;
    use directories::ProjectDirs;

    extern crate adr_core;

    steps! (crate::AdrNames => {
        given "An existing In Progress Decision" |adr, _step| {
            let project_dirs: ProjectDirs = match ProjectDirs::from("murex", "adrust-tool", "test") {
                None => panic!("issue while preparing test"),
                Some(project_dirs) => project_dirs
            };
            fs::copy("./tests/my-decision-1.adoc", 
                project_dirs.cache_dir().join("src/my-decision-1.adoc").as_path()
            ).unwrap();

            adr.name = String::from(format!("{}", 
                project_dirs.cache_dir().join("src/my-decision-1.adoc").as_path().display())
            );
        };

        when "I change its status to decided" |adr, _step| {
            let mut f = File::open(&adr.name).unwrap();
            let mut content = String::new();
            f.read_to_string(&mut content).unwrap();

            assert_eq!(content.contains("{wip}"), true);

            let contains = adr_core::adr_repo::transition_to_decided(&adr.name).unwrap();
            assert_eq!(contains, true);


        };

        then "The content of the file is updated to Decided" |adr, _step| {
            let mut f = File::open(&adr.name).unwrap();
            let mut content = String::new();
            f.read_to_string(&mut content).unwrap();

            assert_eq!(content.contains("{decided}"), true);
        };
    });
}

mod not_update_adr_decided_steps {
    use cucumber::steps;
    use std::fs;
    use std::fs::File;
    use std::io::prelude::*;

    extern crate directories;
    use directories::ProjectDirs;

    extern crate adr_core;

    steps! (crate::AdrNames => {
        given "An existing not In Progress Decision" |adr, _step| {
            let project_dirs: ProjectDirs = match ProjectDirs::from("murex", "adrust-tool", "test") {
                None => panic!("issue while preparing test"),
                Some(project_dirs) => project_dirs
            };
            fs::copy("./tests/my-wrong-decision-1.adoc", 
                project_dirs.cache_dir().join("src/my-wrong-decision-1.adoc").as_path()
            ).unwrap();

            adr.name = String::from(format!("{}", 
                project_dirs.cache_dir().join("src/my-wrong-decision-1.adoc").as_path().display())
            );
        };

        when "I update its status to decided" |adr, _step| {
            let contains = adr_core::adr_repo::transition_to_decided(&adr.name).unwrap();
            assert_eq!(contains, false);
        };

        then "The content of the file is not changed" |adr, _step| {
            let mut f = File::open(&adr.name).unwrap();
            let mut content = String::new();
            f.read_to_string(&mut content).unwrap();

            assert_eq!(content.contains("{superseded} /tmp/adr-samples/src/my-decision-2.adoc"), true);
        };
    });
}

mod transition_adr_to_completed_steps {
    use cucumber::steps;
    use std::fs;
    use std::fs::File;
    use std::io::prelude::*;

    extern crate directories;
    use directories::ProjectDirs;

    extern crate adr_core;

    steps! (crate::AdrNames => {
        given "A Decided Decision" |adr, _step| {
            let project_dirs: ProjectDirs = match ProjectDirs::from("murex", "adrust-tool", "test") {
                None => panic!("issue while preparing test"),
                Some(project_dirs) => project_dirs
            };
            fs::copy("./tests/my-initial-decided-decision.adoc", 
                project_dirs.cache_dir().join("src/my-initial-decided-decision.adoc").as_path()
            ).unwrap();
            fs::copy("./tests/my-new-decided-decision.adoc", 
                project_dirs.cache_dir().join("src/my-new-decided-decision.adoc").as_path()
            ).unwrap();

            adr.name = String::from(format!("{}", 
                project_dirs.cache_dir().join("src/my-initial-decided-decision.adoc").as_path().display())
            );
            adr.by = String::from(format!("{}", 
                project_dirs.cache_dir().join("src/my-new-decided-decision.adoc").as_path().display())
            );
        };

        when "I update its status to completed-by" |adr, _step| {
            let contains = adr_core::adr_repo::transition_to_completed_by(&adr.name, &adr.by).unwrap();
            assert_eq!(contains, true);
        };

        then "The status of the initial decision is completed-by" |adr, _step| {
            let mut f = File::open(&adr.name).unwrap();
            let mut content = String::new();
            f.read_to_string(&mut content).unwrap();

            let expected_value = format!("{{completed}} {}", &adr.by);
            assert_eq!(content.contains(&expected_value), true);
        };

        then "The status of the new decision is completes" |adr, _step| {
            let mut f = File::open(&adr.by).unwrap();
            let mut content = String::new();
            f.read_to_string(&mut content).unwrap();

            let expected_value = format!("{{completes}} {}", &adr.name);
            assert_eq!(content.contains(&expected_value), true);
        };
    });
}

before!(a_before_fn => |_scenario| {
    let project_dirs: ProjectDirs = match ProjectDirs::from("murex", "adrust-tool", "test") {
        None => panic!("issue while preparing test"),
        Some(project_dirs) => project_dirs
    };
    fs::create_dir_all(project_dirs.cache_dir().join("src").as_path()).unwrap();
    fs::create_dir_all(project_dirs.cache_dir().join("templates").as_path()).unwrap();

    fs::copy("./tests/adr-template-v0.1.adoc", project_dirs.cache_dir().join("templates/adr-template-v0.1.adoc").as_path()).unwrap();
});

after!(an_after_fn => |_scenario| {
    let project_dirs: ProjectDirs = match ProjectDirs::from("murex", "adrust-tool", "test") {
        None => panic!("issue while preparing test"),
        Some(project_dirs) => project_dirs
    };
    fs::remove_dir_all(project_dirs.cache_dir()).unwrap();
});

fn setup() {}

cucumber! {
    features: "./features/adr_lifecycle",
    world: crate::Adr,
    steps: &[
        create_new_adr_steps::steps,
        create_already_adr_steps::steps,
        update_adr_decided_steps::steps,
        not_update_adr_decided_steps::steps,
        transition_adr_to_completed_steps::steps,
    ],
    setup: setup,
    before: &[a_before_fn],
    after: &[an_after_fn]
}
