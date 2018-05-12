// TODO: Wrap up most things except main into a rubik2x2 crate, instead of gathering everything here.
extern crate winapi;
use std::env;
mod ui;
mod cli;
mod cube;
mod error;
mod log;
mod solver;
mod permutations;

fn report_error_and_exit(message: &String) -> !
{
    println!("{}", message);
    ::std::process::exit(1);
}

fn main()
{
    // TODO: Run tests in a test suite instead.
    solver::test();
    
    let console_context =
    {
        match ui::ConsoleContext::try_initialize()
        {
            Ok(context) => context,
            Err(error) => report_error_and_exit(&error.message()),
        }
    };
    
    let input =
    {
        match cli::try_read_arguments(&mut env::args().skip(1))
        {
            Ok(input) => input,
            Err(error) => report_error_and_exit(&error.message()),
        }
    };
    
    let solution_moves = solver::solution(&input.initial_cube);
    ui::run_main_loop(&input.aliases, &input.initial_cube, &console_context, &solution_moves);
    if let Some(error) = console_context.try_deinitialize()
    {
        log::log(&error.message());
    }
}
