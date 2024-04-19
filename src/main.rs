use deno_core::error::AnyError;
use std::{path::PathBuf, process::Command, rc::Rc, thread};

#[tokio::main]
async fn main() {
    let mut args = std::env::args();
    if args.len() != 2 {
        println!("Usage snaptikparser 'jsfile'");
        return;
    }

    let file_path = PathBuf::from(args.nth(1).unwrap());

    let current_dir = std::env::current_dir().unwrap();

    println!("{}", file_path.to_str().unwrap());
    println!("{}", current_dir.to_str().unwrap());

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


    let file_path = "C:\\Users\\do.tran\\Desktop\\newme\\snaptik-parser\\snaptik.js";
    let file_content = tokio::fs::read_to_string(file_path).await.unwrap();
    let splits: Vec<&str> = file_content.split("\n").collect();
    dbg!(splits[42]);
}

fn spawn_js_file(file_path: &PathBuf, current_dir: &PathBuf){
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
