use std::path::Path;

use async_zip::{base::write::ZipFileWriter, Compression, ZipEntryBuilder};
use tokio::{
    fs::{self, File},
    process::Command,
};

use crate::{config::get_config, errors::CompileError};

pub async fn execute(wd: String) -> Result<(), CompileError> {
    let config = get_config().await;
    let author = config.author.clone();

    // Lets first prepare an output folder as final destination
    let final_folder = format!("{}\\{}", config.s4_mods_path, wd);
    if !Path::new(&final_folder).exists() {
        fs::create_dir(&final_folder).await?;
    }

    let mut tasks = vec![];

    // Check if there are any scripts to compile
    if has_sub_target(&wd, "scripts", "py").await? {
        let wd = wd.clone();
        let final_folder = final_folder.clone();
        let author = author.clone();
        let task = tokio::spawn(async move {
            attempt_script_compilation(
                &wd,
                &config.py_path,
                format!("{}\\{}_{}.ts4script", final_folder, &author, &wd),
            )
            .await
        });

        tasks.push(task);
    }

    // TODO: Add support for tuning?
    // if has_sub_target(&wd, "tuning", "xml").await? {
    //     let task = tokio::spawn(async move {
    //         attempt_tuning_compilation(
    //             &wd,
    //             format!("{}\\{}_{}.package", final_folder, &author, &wd),
    //         )
    //         .await
    //     });

    //     tasks.push(task);
    // }

    for task in tasks {
        task.await.unwrap()?;
    }

    Ok(())
}

async fn attempt_script_compilation(
    wd: &str,
    py_path: &str,
    dest: String,
) -> Result<(), CompileError> {
    let python_exe = format!("{}\\python.exe", py_path);

    compile_all(&python_exe, &wd).await?;

    // Package (zip) the tuning files
    let package_path = package_files(&wd, "scripts\\__pycache__", "pyc", "ts4script").await?;
    println!("package_path: {:?}", package_path);
    // And move them to the S4 Mods directory
    fs::rename(&package_path, &dest).await?;

    Ok(())
}

// async fn attempt_tuning_compilation(wd: &str, dest: String) -> Result<(), CompileError> {
//     // TODO: This
//     Ok(())
// }

async fn compile_all(py_exe: &str, scripts_dir: &str) -> Result<(), CompileError> {
    let output = Command::new(py_exe)
        .arg("-m")
        .arg("compileall")
        .arg(scripts_dir)
        .output()
        .await?;

    if output.status.success() {
        println!("Scripts compiled successfully.");
    } else {
        return Err(CompileError::CompileCommandError(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(())
}

async fn package_files(
    wd: &str,
    module_dir: &str,
    filter_ext: &str,
    target_ext: &str,
) -> Result<String, CompileError> {
    // E.g. example_mod\scripts
    let module_dir = format!("{}\\{}", wd, module_dir);
    // E.g. example_mod\scripts\example_mod.ts4script
    let zip_path = format!("{}\\{}.{}", module_dir, wd, target_ext);

    let zip_file = File::create(&zip_path).await?;
    let mut zip_writer = ZipFileWriter::with_tokio(zip_file);

    let mut entries = tokio::fs::read_dir(module_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some(filter_ext) {
            let mut file_name = path.file_name().unwrap().to_str().unwrap().to_string();
            if file_name.contains(".cpython-37") {
                file_name = file_name.replace(".cpython-37", "");
            }

            let zip_internal_path = format!("{}/{}", wd, file_name);

            let file_data = tokio::fs::read(&path).await?;
            let entry = ZipEntryBuilder::new(zip_internal_path.into(), Compression::Stored).build();

            zip_writer.write_entry_whole(entry, &file_data).await?;
        }
    }

    zip_writer.close().await?;

    Ok(zip_path)
}

async fn has_sub_target(target_wd: &str, sub_dir: &str, ext: &str) -> Result<bool, CompileError> {
    let sub_dir = format!("{}\\{}", target_wd, sub_dir);
    if !Path::new(&sub_dir).exists() {
        return Ok(false);
    }

    let mut file_count = 0;

    let mut entries = fs::read_dir(sub_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some(ext) {
            file_count += 1;

            if file_count > 1 {
                return Ok(true);
            }
        }
    }

    Ok(false)
}
