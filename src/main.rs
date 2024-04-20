use deno_core::error::AnyError;
use std::{path::PathBuf, process::Command, rc::Rc, thread};
use tokio::io::{AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() {
    let mut args = std::env::args();
    if args.len() != 2 {
        println!("Usage snaptikparser 'jsfile'");
        return;
    }

    let file_path = PathBuf::from(args.nth(1).unwrap());

    let working_dir = std::env::current_dir().unwrap();

    // println!("{}", file_path.to_str().unwrap());
    // println!("{}", working_dir.to_str().unwrap());

    if cfg!(target_os = "windows") {
        let mut command = Command::new("cmd");
        command.args([
            "npx",
            "biome",
            "format",
            "--write",
            file_path.to_str().unwrap(),
        ]);

        command.output().expect("Something unexpected happens");
    }
    if cfg!(target_os = "linux") {
        let mut command = Command::new("bash");
        command.args([
            "npx",
            "biome",
            "format",
            "--write",
            file_path.to_str().unwrap(),
        ]);

        command.output().expect("Something unexpected happens");
        insert_console_log(&file_path).await.unwrap();
        spawn_js_file(file_path.clone(), working_dir.clone());
    }
}

async fn insert_console_log(file_path: &PathBuf) -> std::io::Result<()> {
    let file_content = tokio::fs::read_to_string(file_path).await?;
    let lines: Vec<String> = file_content.split('\n').map(|x| x.to_owned()).collect();
    let mut new_vec_content = Vec::with_capacity(lines.len() + 1);
    for (index, line) in lines.iter().enumerate() {
        if index == 44 {
            new_vec_content.push("console.log(decodeURIComponent(escape(r)))");
        }
        new_vec_content.push(line);
    }

    let new_content = new_vec_content.join("\n");
    tokio::fs::write(file_path, new_content).await?;
    Ok(())
}

fn spawn_js_file(file_path: PathBuf, current_dir: PathBuf) {
    let thread = thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        if let Err(error) = runtime.block_on(run_js(
            file_path.to_str().unwrap(),
            current_dir.to_str().unwrap(),
        )) {
            eprintln!("error: {}", error);
        }
    });

    thread.join().unwrap();
}

async fn run_js(file_path: &str, current_dir: &str) -> Result<(), AnyError> {
    let main_module = deno_core::resolve_path(file_path, current_dir.as_ref())?;

    let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
        ..Default::default()
    });

    js_runtime
        .execute_script("[snaptikparser:runtime.js]", include_str!("runtime.js"))
        .unwrap();

    let mod_id = js_runtime.load_main_es_module(&main_module).await?;

    let result = js_runtime.mod_evaluate(mod_id);

    js_runtime
        .run_event_loop(deno_core::PollEventLoopOptions::default())
        .await?;

    result.await
}
