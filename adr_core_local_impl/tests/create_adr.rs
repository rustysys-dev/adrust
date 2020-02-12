use cucumber::{after, before, cucumber};

extern crate directories;
use directories::ProjectDirs;

use std::fs;

pub struct AdrNames {
    name: String,
    by: String,
    has_transitioned: bool,
    base_path: String,
}

impl cucumber::World for AdrNames {}

impl std::default::Default for AdrNames {
    fn default() -> AdrNames {
        AdrNames {
            name: "default-name".to_string(),
            by: "default-name".to_string(),
            has_transitioned: false,
            base_path: String::from(""),
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

mod check_transitions_and_lifecycle_of_adr {
    use cucumber::steps;
    use std::fs;
    use std::path::Path;

    extern crate directories;
    use directories::ProjectDirs;

    extern crate adr_core;
    use adr_core::adr_repo::*;

    use walkdir::{WalkDir};

    use std::path::PathBuf;

    steps! (crate::AdrNames => {

        given regex r"^a decision with status (.+)$" (String) |adr, status, _step| {
        
            let project_dirs: ProjectDirs = match ProjectDirs::from("murex", "adrust-tool", "test") {
                None => panic!("issue while preparing test"),
                Some(project_dirs) => project_dirs
            };
            
            //copy all files to ease the different transitions
            let rep = "./tests/data/";
            let srcdir = PathBuf::from(rep);

            for entry in WalkDir::new(srcdir.as_path()).into_iter().filter_map(|e| e.ok() ) {
                if ! entry.path().is_dir() {
                    let from = entry.path();
                    let file_name = entry.path().file_name().unwrap();
                    let to = project_dirs.cache_dir().join("src").join(file_name);

                    println!("[init] Want to copy file [{:?}] to [{:?}]", from, to);
                    match fs::copy(from, to.as_path()) {
                        Ok(_) => (),
                        Err(why) => panic!(why),
                    };
                }
            }
            
            adr.base_path = format!("{}", project_dirs.cache_dir().join("src").display());
            adr.name = format!("{}", project_dirs.cache_dir().join("src").join(status).with_extension("adoc").display());
            println!("The current scenario will use adr path [{}]", adr.name);
        };

        when regex r"^the decision is transitioned to (.+) by (.+)$" (String, String) |adr, transition, by, _step| {
            match "n/a" == by {
                true => {
                    println!("calling transition_to() with [{}] [{}] [{}]", transition, adr.name, by);
                    match adr_core::adr_repo::transition_to(TransitionStatus::from_str(transition), adr.name.as_str(), "") {
                        Ok(transitioned) => adr.has_transitioned = transitioned,
                        Err(why) => panic!(why)
                    };
                },
                false => {            
                    let by = format!("{}", PathBuf::from(adr.base_path.as_str()).join(by).display());
                    println!("calling transition_to() with [{}] [{}] [{}]", transition, adr.name, by);
                    match adr_core::adr_repo::transition_to(TransitionStatus::from_str(transition), adr.name.as_str(), by.as_str()) {
                        Ok(transitioned) => adr.has_transitioned = transitioned,
                        Err(why) => panic!(why)
                    };
                },
            };
        };

        then regex r"^the transition is (.+)$" (bool) |adr, has_transition, _step| {
            //compare expected with actual
            assert_eq!(has_transition, adr.has_transitioned)
        };

        then regex r"^the new status is (.+) by (.+)$" (String, String) |adr, new_status, by, _step| {
            //TODO manage the by
            let path = Path::new(adr.name.as_str());
            let new_adr = match adr_core::adr_repo::build_adr_from_path(&path) {
                Ok(adr) => adr,
                Err(why) => panic!(why),
            };

            assert_eq!(Status::from_str(new_status), new_adr.status);
        };

        then "the date is updated to today TO BE REMOVED" |_adr, _step| {
            //TODO manage the date
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
        check_transitions_and_lifecycle_of_adr::steps,
        // update_adr_decided_steps::steps,
        // not_update_adr_decided_steps::steps,
        // transition_adr_to_completed_steps::steps,
    ],
    setup: setup,
    before: &[a_before_fn],
    after: &[an_after_fn]
}
