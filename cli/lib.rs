// Copyright 2018-2023 the Deno authors. All rights reserved. MIT license.

mod args;
mod auth_tokens;
mod cache;
mod deno_std;
mod emit;
mod errors;
mod factory;
mod file_fetcher;
mod graph_util;
mod http_util;
mod js;
mod lsp;
mod module_loader;
mod napi;
mod node;
mod npm;
mod ops;
mod resolver;
mod standalone;
mod tools;
mod tsc;
mod util;
mod version;
mod worker;

use crate::args::DenoSubcommand;
use crate::args::Flags;
use crate::util::display;
use crate::util::v8::get_v8_flags_from_env;
use crate::util::v8::init_v8_flags;
use crate::worker::CliMainWorker;

use deno_core::error::AnyError;
use deno_core::serde_v8;
use deno_core::v8;
use deno_runtime::colors;
use deno_runtime::permissions::Permissions;
use deno_runtime::permissions::PermissionsContainer;
use deno_runtime::tokio_util::create_and_run_current_thread;
use factory::CliFactory;

use crate::args::RunFlags;
use deno_core::anyhow::bail;
use deno_core::serde_json;

fn run_function(
  worker: CliMainWorker,
  function: &str,
  arg: serde_json::Value,
) -> Result<deno_core::serde_json::Value, deno_core::serde_v8::Error> {
  let mut main_worker = worker.into_main_worker();
  let scope = &mut main_worker.js_runtime.handle_scope();
  let testfn =
    deno_core::JsRuntime::eval::<v8::Function>(scope, function).unwrap();
  let this = v8::undefined(scope).into();
  let args = serde_v8::to_v8(scope, arg).unwrap();
  let local = testfn.call(scope, this, &[args]).unwrap();

  serde_v8::from_v8::<deno_core::serde_json::Value>(scope, local)
}

async fn setup_worker_and_run_function(
  flags: Flags,
  function: &str,
  arg: serde_json::Value,
) -> Result<deno_core::serde_json::Value, AnyError> {
  if !flags.has_permission() && flags.has_permission_in_argv() {
    bail!(
      r#"Permission flags have likely been incorrectly set after the script argument.
To grant permissions, set them before the script argument. For example:
    deno run --allow-read=. main.js"#
    );
  }

  let factory = CliFactory::from_flags(flags).await?;
  let cli_options = factory.cli_options();

  let main_module = cli_options.resolve_main_module()?;

  let permissions = PermissionsContainer::new(Permissions::from_options(
    &cli_options.permissions_options(),
  )?);
  let worker_factory = factory.create_cli_main_worker_factory().await?;
  let mut worker = worker_factory
    .create_main_worker(main_module, permissions)
    .await?;

  // I guess I should use this exit code somehow but I don't know what it is for
  let _exit_code = worker.run().await?;

  run_function(worker, function, arg).map_err(|e| e.into())
}

// The function name is conceptually correct, but wrong from the Deno creates
// setup, as we are actually using code from deno_cli, rather than from
// deno_runtime. I think Deno project should move many features that are
// currently implemented inside deno_cli to deno_runtime.
pub fn run_js_function_with_arg_and_get_its_returning_value_using_deno_runtime(
  main_module_url_or_path: &str,
  function: &str,
  arg: serde_json::Value,
) -> Result<serde_json::Value, AnyError> {
  util::unix::raise_fd_limit();
  util::windows::ensure_stdio_open();
  #[cfg(windows)]
  colors::enable_ansi(); // For Windows 10
  deno_runtime::permissions::set_prompt_callbacks(
    Box::new(util::draw_thread::DrawThread::hide),
    Box::new(util::draw_thread::DrawThread::show),
  );

  let script = main_module_url_or_path.to_string();
  let function = function.to_string();
  let mut flags = Flags::default();

  flags.subcommand = DenoSubcommand::Run(RunFlags {
    script,
    watch: None,
  });

  let future = async move {
    let default_v8_flags = vec![];
    init_v8_flags(&default_v8_flags, &flags.v8_flags, get_v8_flags_from_env());

    util::logger::init(flags.log_level);

    setup_worker_and_run_function(flags, &function, arg).await
  };

  let value = create_and_run_current_thread(future)?;

  Ok(value)
}

#[test]
fn test_run_js_function_with_arg_and_get_its_returning_value_using_deno_runtime(
) {
  let main_module_path = "../../deno_example/example.tsx";
  let function = "globalThis.test";
  let arg = deno_core::serde_json::json!({ "content": "Hello world!" });
  let result =
    run_js_function_with_arg_and_get_its_returning_value_using_deno_runtime(
      main_module_path,
      function,
      arg,
    );
  println!("Result: {result:?}");
}
