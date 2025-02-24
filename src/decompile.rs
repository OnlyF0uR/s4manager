use std::{path::Path, process::Stdio, sync::Arc};

use futures_lite::AsyncReadExt;
use tokio::{
    fs::{create_dir_all, remove_dir_all, File},
    io::BufReader,
    process::Command,
    sync::Semaphore,
};

use crate::{config::get_config, errors::DecompileError};

pub async fn execute(target: Option<&str>) -> Result<(), DecompileError> {
    println!("Decompiling: {:?}", target.unwrap_or("base game"));

    match target {
        Some("base") => decomp_base().await,
        None => decomp_base().await,
        Some(target) => {
            if target.ends_with(".ts4script") {
                decomp_mod(target).await
            } else {
                Err(DecompileError::InvalidTarget)
            }
        }
    }
}

async fn decomp_base() -> Result<(), DecompileError> {
    let config = get_config().await;

    let base_dir = Path::new(&config.s4_install_path)
        .join("Data")
        .join("Simulation")
        .join("Gameplay");

    if !base_dir.exists() {
        return Err(DecompileError::BaseDirectoryNotFound);
    }

    ensure_wd().await?; // Ensure working directory is set

    let mut unzip_tasks = vec![];

    let mut entries = tokio::fs::read_dir(base_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        // This loop is per ZIP file
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("zip") {
            let task = tokio::spawn(async move {
                let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
                let fn_only = file_name.replace(".zip", "");

                // Create directory for the mod
                let mod_dir = format!("decompiled\\{}", fn_only);
                if Path::new(&mod_dir).exists() {
                    // Delete it if it already exists
                    remove_dir_all(&mod_dir).await?;
                }

                create_dir_all(&mod_dir).await?;

                // Open the ZIP file
                unzip(&path.to_str().unwrap(), &mod_dir).await?;

                let semaphore = Arc::new(Semaphore::new(16));
                decompile_folder(&mod_dir, &semaphore).await
            });

            unzip_tasks.push(task);
        }
    }

    for task in unzip_tasks {
        task.await.unwrap()?;
    }

    Ok(())
}

async fn decompile_folder(
    directory: &str,
    semaphore: &Arc<Semaphore>,
) -> Result<(), DecompileError> {
    // Read the current folder
    let mut files_and_folders = tokio::fs::read_dir(directory).await?;

    let mut tasks = vec![];

    while let Some(entry) = files_and_folders.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            match path.extension().and_then(|ext| ext.to_str()) {
                Some(ext) => {
                    if ext != "pyc" {
                        continue;
                    }
                }
                None => continue,
            };

            // If we encounter a file, we acquire a permit and spawn a task to decompile it
            let semaphore_clone = Arc::clone(semaphore);
            let task = tokio::spawn(async move {
                let _permit = semaphore_clone.acquire().await.unwrap();
                decompile_file(&path.to_str().unwrap()).await
            });

            tasks.push(task);
        } else {
            // If we encouter a folder, we recursively decompile that one too
            Box::pin(decompile_folder(&path.to_str().unwrap(), semaphore)).await?;
        }
    }

    for task in tasks {
        task.await.unwrap()?;
    }

    Ok(())
}

async fn decompile_file(path: &str) -> Result<(), DecompileError> {
    println!("Decompiling: {:?}", path);

    let output_path = path.split("\\").collect::<Vec<&str>>()
        [..path.split("\\").collect::<Vec<&str>>().len() - 1]
        .join("\\");

    let status = Command::new("uncompyle6")
        .arg(&path)
        .arg("-o")
        .arg(&output_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .expect("Failed to execute uncompyle6");

    if status.success() {
        // Delete the original file
        tokio::fs::remove_file(path).await?;
        println!("Successfully decompiled: {}", path);
    } else {
        eprintln!("Failed to decompile: {}", path);
    }

    Ok(())
}

async fn unzip(path: &str, mod_dir: &str) -> Result<(), DecompileError> {
    println!("Unzipping: {}", path);

    let mut file = File::open(&path).await?;
    let mut reader =
        async_zip::base::read::seek::ZipFileReader::with_tokio(BufReader::new(&mut file))
            .await
            .unwrap();

    for index in 0..reader.file().entries().len() {
        let entry = reader.file().entries().get(index).unwrap();
        if entry.dir().unwrap() {
            continue;
        }

        let write_to_file = format!(
            "{}\\{}",
            mod_dir,
            entry.filename().as_str().unwrap().replace("/", "\\")
        );

        let mut entry_reader = reader
            .reader_without_entry(index)
            .await
            .expect("Failed to read ZipEntry");

        let folder_to_write = Path::new(&write_to_file).parent().unwrap();
        if !folder_to_write.exists() {
            create_dir_all(folder_to_write).await?;
        }

        let mut buffer = vec![];
        entry_reader
            .read_to_end(&mut buffer)
            .await
            .expect("Failed to read ZipEntry");

        tokio::fs::write(&write_to_file, &buffer).await?;
    }

    Ok(())
}

async fn decomp_mod(target: &str) -> Result<(), DecompileError> {
    if !Path::new(target).exists() {
        return Err(DecompileError::TargetFileNotFound);
    };

    ensure_wd().await?;

    // Create directory for the mod
    let mod_dir = format!("decompiled\\{}", target.replace(".ts4script", ""));
    if Path::new(&mod_dir).exists() {
        // Delete it if it already exists
        remove_dir_all(&mod_dir).await?;
    }

    create_dir_all(&mod_dir).await?;

    // Open the ZIP file
    unzip(target, &mod_dir).await?;

    let semaphore = Arc::new(Semaphore::new(16));
    decompile_folder(&mod_dir, &semaphore).await?;

    Ok(())
}

async fn ensure_wd() -> Result<String, DecompileError> {
    let wd = "decompiled";
    if !Path::new(wd).exists() {
        tokio::fs::create_dir(wd).await?;
    }

    Ok(wd.to_string())
}
