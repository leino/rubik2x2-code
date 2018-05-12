use std;
use cube;
use winapi;
use error;
use log;

#[derive(Debug)]
pub enum Input {
    Forward,
    Back,
    Exit,
}

pub struct SideAliases
{
    pub aliases: std::collections::HashMap<String, cube::Side>
}

impl SideAliases
{
    pub fn alias(&self, side: cube::Side) -> String
    {
        for (a, s) in self.aliases.iter() {
            if side == *s {
                return a.clone();
            }
        }
        panic!();
    }

    pub fn optional_side(&self, alias: &str) -> Option<&cube::Side>
    {
        self.aliases.get(alias)
    }
}

pub fn run_main_loop(
    aliases: &SideAliases,
    starting_cube: &cube::Cube,
    console_context: &ConsoleContext,
    solution_moves: &Vec<cube::Move>
)
{
    let page_moves_count = 4;
    let mut page_idx = 0;

    if solution_moves.len() > 0 {
        loop {
            use std::cmp::min;

            let pages_count = (solution_moves.len() + 1) / page_moves_count;
            assert!(pages_count > 0);
            
            // Render
            {
                console_context.clear();

                println!("Progress: {}/{}", page_idx + 1, pages_count);
                print!("\n\n\n");
                
                let page_moves_lo_idx = page_moves_count * page_idx;                
                let page_moves_hi_idx =
                    min(solution_moves.len(), page_moves_lo_idx + page_moves_count);

                {
                    let cube = starting_cube.sequence_moves(solution_moves[0..page_moves_lo_idx].iter());
                    print_diagram(&mut |side| aliases.alias(side), &cube);
                }                

                print!("\n\n");
                let page_moves = &solution_moves[page_moves_lo_idx .. page_moves_hi_idx];
                for m in page_moves {                    
                    print!("{:?}  ", m);
                }

                print!("\n\n");
                
                {
                    let cube = starting_cube.sequence_moves(solution_moves[0..page_moves_hi_idx].iter());
                    print_diagram(&mut |side| aliases.alias(side), &cube);
                }


                println!("\n\n\n");
                println!("Left key: previous page");
                println!("Right key: next page");
                println!("Escape: exit");
            }
            
            // Get input
            let input = console_context.wait_for_input();
            
            // Advance state
            {
                match input {
                    Input::Forward => {
                        if page_idx < pages_count - 1 {
                            page_idx = page_idx + 1;
                        }
                    },
                    Input::Back => {
                        if page_idx > 0 {
                            page_idx = page_idx - 1;
                        }
                    },
                    Input::Exit => {
                        break;
                    },
                }
            }
        }
    }
    
}



pub struct ConsoleContext {
    buffer_width: u32,
    buffer_height: u32,
    character_attributes: winapi::shared::minwindef::WORD,
    output_device_handle: winapi::um::winnt::HANDLE,
    input_device_handle: winapi::um::winnt::HANDLE,
    initial_console_mode: winapi::shared::minwindef::DWORD,
}

pub enum ConsoleContextInitializationError
{
    StandardInputDeviceHandleIsInvalid{error_code: winapi::shared::minwindef::DWORD},
    StandardOutputDeviceHandleIsInvalid{error_code: winapi::shared::minwindef::DWORD},
    StandardInputDeviceDoesNotExist,
    StandardOutputDeviceDoesNotExist,
}

pub enum ConsoleContextDeinitializationError
{
    FailedToResetConsoleMode{error_code: winapi::shared::minwindef::DWORD},
}

impl ConsoleContextDeinitializationError
{
    pub fn message(&self) -> String
    {
        use self::ConsoleContextDeinitializationError::*;
        match self
        {
            &FailedToResetConsoleMode{error_code} =>
                format!("failed to reset console mode: error code: {}", error_code),
        }
    }
}

impl ConsoleContextInitializationError
{
    pub fn message(&self) -> String
    {
        use self::ConsoleContextInitializationError::*;
        match self
        {
            &StandardOutputDeviceHandleIsInvalid{error_code} =>
                format!("Output device handle is invalid: error code: {}", error_code),
            &StandardInputDeviceHandleIsInvalid{error_code} =>
                format!("Input device handle is invalid: error code: {}", error_code),
            &StandardOutputDeviceDoesNotExist =>
                String::from("There is no standard output device"),
            &StandardInputDeviceDoesNotExist =>
                String::from("There is no standard input device"),
        }
    }
}

impl ConsoleContext {

    fn wait_for_input(&self) -> Input {
        use winapi::um::winuser::*;
        use winapi::um::wincon::*;
        use winapi::um::consoleapi::*;
        use winapi::um::synchapi::*;
        use winapi::shared::minwindef::*;
        use winapi::um::winbase::*;

        loop {
            {
                let timeout_duration = INFINITE;
                let result =
                    unsafe {
                        WaitForSingleObject(
                            self.input_device_handle,
                            timeout_duration
                        )
                    };
                if result != WAIT_OBJECT_0 {
                    log::log_get_last_error_code_and_exit(file!(), line!(), column!());
                }
            }
        
            {
                let mut input_records: [INPUT_RECORD; 128] = unsafe{std::mem::uninitialized()};
                let mut read_event_count = 0;
                let result =
                    unsafe {
                        ReadConsoleInputW(
                            self.input_device_handle,
                            input_records.as_mut_ptr(),
                            input_records.len() as u32,
                            &mut read_event_count
                        )
                    };
                if result == 0 {
                    log::log_get_last_error_code_and_exit(file!(), line!(), column!());
                }

                for read_event_idx in 0..read_event_count {
                    let input_record = &input_records[read_event_idx as usize];
                    match input_record.EventType {
                        KEY_EVENT => {
                            let event = unsafe { input_record.Event.KeyEvent() };
                            if (event.wVirtualKeyCode as i32) == VK_RIGHT && event.bKeyDown == TRUE {
                                return Input::Forward;
                            }
                            if (event.wVirtualKeyCode as i32) == VK_LEFT && event.bKeyDown == TRUE {
                                return Input::Back;
                            }
                            if (event.wVirtualKeyCode as i32) == VK_ESCAPE && event.bKeyDown == TRUE {
                                return Input::Exit;
                            }                            
                        },
                        _ => {},
                    }
                }
            }
        }
    }
    
    pub fn try_initialize() -> Result<ConsoleContext, ConsoleContextInitializationError> {

        use winapi::um::winbase::*;
        use winapi::um::handleapi::*;
        use winapi::um::consoleapi::*;
        use winapi::um::winnt::HANDLE;
        use self::ConsoleContextInitializationError::*;
        let output_device_handle = unsafe {
            let standard_device_handle = STD_OUTPUT_HANDLE;
            winapi::um::processenv::GetStdHandle(standard_device_handle) as HANDLE
        };
        if output_device_handle == INVALID_HANDLE_VALUE {
            let error_code = unsafe { winapi::um::errhandlingapi::GetLastError() };
            return Err(StandardOutputDeviceHandleIsInvalid{error_code});
        }
        if output_device_handle == (0 as HANDLE) {
            return Err(StandardOutputDeviceDoesNotExist);
        }

        let input_device_handle = unsafe {
            let standard_device_handle = STD_INPUT_HANDLE;
            winapi::um::processenv::GetStdHandle(standard_device_handle) as HANDLE
        };
        if input_device_handle == INVALID_HANDLE_VALUE {
            let error_code = unsafe { winapi::um::errhandlingapi::GetLastError() };
            return Err(StandardInputDeviceHandleIsInvalid{error_code});
        }
        if input_device_handle == (0 as HANDLE) {
            return Err(StandardInputDeviceDoesNotExist);
        }
        
        let console_screen_buffer_info = {
            let mut info = unsafe{std::mem::uninitialized()};
            let result = unsafe{winapi::um::wincon::GetConsoleScreenBufferInfo(output_device_handle, &mut info)};
            if result == 0 {
                let error_code = unsafe {winapi::um::errhandlingapi::GetLastError()};
                if error_code == winapi::shared::winerror::ERROR_INVALID_HANDLE {
                    assert!(output_device_handle != INVALID_HANDLE_VALUE);
                    assert!(output_device_handle != 0 as HANDLE);
                    // It seems that this can happen even if output device handle is valid.
                    // GetConsoleScreenBufferInfo mentions that the output device must have GENERIC_READ access
                    // rights. I believe this error happens when that's not fulfilled, but I could not find it
                    // mentioned in the documentation.
                    // A more descriptive error message could be given if it could be determined that we do not have
                    // GENERIC_READ access, but that seems pretty complicated to do.
                    error::report_fatal_error_and_exit(
                        &String::from("Unable to access screen buffer info for the console's standard output device.")
                    );
                }
                log::log_get_last_error_code_and_exit(file!(), line!(), column!());
            }
            info
        };        

        // Record in order to restore before exit
        let initial_console_mode = {
            let mut mode = 0;
            let result = unsafe{ GetConsoleMode(input_device_handle, &mut mode) };
            if result == 0 {
                log::log_get_last_error_code_and_exit(file!(), line!(), column!());
            }
            mode
        };

        {
            let mode = winapi::um::wincon::ENABLE_WINDOW_INPUT;
            let result = unsafe { SetConsoleMode(input_device_handle, mode) };
            if result == 0 {
                log::log_get_last_error_code_and_exit(file!(), line!(), column!());
            }
        }
        
        Ok(
            ConsoleContext {
                output_device_handle: output_device_handle,
                input_device_handle: input_device_handle,
                buffer_width: console_screen_buffer_info.dwSize.X as u32,
                buffer_height: console_screen_buffer_info.dwSize.Y as u32,
                character_attributes: console_screen_buffer_info.wAttributes,
                initial_console_mode: initial_console_mode,
            }
        )
    }

    pub fn try_deinitialize(&self) -> Option<ConsoleContextDeinitializationError> {
        use winapi::um::consoleapi::*;
        use self::ConsoleContextDeinitializationError::*;
        // Restore the initial console mode.
        {
            let result = unsafe { SetConsoleMode(self.input_device_handle, self.initial_console_mode) };
            if result == 0 {
                let error_code = unsafe { winapi::um::errhandlingapi::GetLastError() };
                return Some(FailedToResetConsoleMode{error_code});
            }
        }

        return None;
    }
    
    #[cfg(windows)]
    fn clear(&self) {
        use winapi::um::wincon::*;
        let top_left_cell_coordinates = COORD {X: 0, Y: 0};
        let fill_length = (self.buffer_width) * (self.buffer_height);
        
        {
            let character = ' ' as u16;
            let mut written_characters_count = 0;
            let result = unsafe {
                winapi::um::wincon::FillConsoleOutputCharacterW(
                    self.output_device_handle,
                    character,
                    fill_length,
                    top_left_cell_coordinates,
                    &mut written_characters_count
                )
            };
            if result == 0 {
                log::log_get_last_error_code_and_exit(file!(), line!(), column!());            
            }
            assert!(written_characters_count == fill_length);
        }

        {
            let mut written_attributes_count = unsafe{std::mem::uninitialized()};
            let result = unsafe {
                winapi::um::wincon::FillConsoleOutputAttribute(
                    self.output_device_handle,
                    self.character_attributes,
                    fill_length,
                    top_left_cell_coordinates,
                    &mut written_attributes_count
                )
            };
            assert!(written_attributes_count == fill_length);
            if result == 0 {
                log::log_get_last_error_code_and_exit(file!(), line!(), column!());
            }
        }

        {
            let result = unsafe {
                winapi::um::wincon::SetConsoleCursorPosition(self.output_device_handle, top_left_cell_coordinates)
            };
            if result == 0 {
                log::log_get_last_error_code_and_exit(file!(), line!(), column!());
            }
        }
    }
    
    
}

fn print_diagram(side_alias: &mut FnMut(cube::Side) -> String, cube: &cube::Cube) {
    use cube::Side::*;
    let column_count = 2*4;
    let row_count = 2*3;
    for row_idx in 0..row_count {
        for column_idx in 0..column_count {
            let optional_side = {
                match (row_idx/2, column_idx/2) {
                    (0, 1) => Some(U),
                    (1, 0) => Some(L),
                    (1, 1) => Some(F),
                    (1, 2) => Some(R),
                    (1, 3) => Some(B),
                    (2, 1) => Some(D),
                    _ => None,
                }
            };
            let c = {
                let (i, j) = (row_idx & 1, column_idx & 1);
                if let Some(side) = optional_side {
                    let position: [i32; 3] = 
                        match side {
                            L => [-1, 1 - 2*i, -1 + 2*j],
                            R => [ 1, 1 - 2*i,  1 - 2*j],
                            
                            D => [-1 + 2*j, -1,  1 - 2*i],
                            U => [-1 + 2*j,  1, -1 + 2*i],
                            
                            B => [ 1 - 2*j, 1 - 2*i, -1],
                            F => [-1 + 2*j, 1 - 2*i,  1],
                        };
                    assert!(position[0].abs() == 1);
                    assert!(position[1].abs() == 1);
                    assert!(position[2].abs() == 1);
                    let target_normal = cube.transform(position).inverse().apply(&cube::normal(side));
                    format!("{}", side_alias(cube::normal_side(target_normal)))
                } else {
                    " ".to_string()
                }
            };
            print!("{}", c);
            if column_idx < column_count - 1 {
                print!(" ");
            }
        }
        println!("");
    }
}

