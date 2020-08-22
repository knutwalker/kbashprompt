use super::{ModuleInfo, ModuleOut};
use colorful::Color;
use std::{
    env,
    path::{Path, PathBuf},
};

const CYAN: Color = Color::LightSeaGreen; // 37

pub(super) fn get(wd: &Path) -> ModuleOut {
    let has_file = move |file: &str| -> bool {
        let build_file = wd.join(file);
        build_file.metadata().ok().filter(|f| f.is_file()).is_some()
    };

    for file in [
        "build.gradle",
        "pom.xml",
        "build.gradle.kts",
        "build.sbt",
        "build.xml",
        ".java-version",
    ]
    .iter()
    {
        if has_file(file) {
            let java_home = env::var_os("JAVA_HOME")?;
            let java_home = PathBuf::from(java_home);
            let java_home = java_home.read_link().unwrap_or(java_home);
            let java_home = java_home.file_name()?;
            let java_home = java_home.to_string_lossy();
            let java_home = java_home.into_owned();
            return ModuleInfo::of()
                .icon("☕️")
                .info(java_home)
                .color(CYAN)
                .some();
        }
    }

    None
}
