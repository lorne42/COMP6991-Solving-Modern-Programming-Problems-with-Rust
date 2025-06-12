use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::sync::{
    mpsc::{channel, Sender},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

use rsheet_lib::cell_expr::{CellArgument, CellExpr};
use rsheet_lib::cell_value::CellValue;
use rsheet_lib::command::{CellIdentifier, Command};
use rsheet_lib::connect::{Connection, Manager, Reader, Writer};
use rsheet_lib::replies::Reply;

/// Converts column number to Excel-style letter format (e.g., 0 -> A, 25 -> Z, 26 -> AA)
fn col_number_to_letters(mut col: u32) -> String {
    let mut result = String::new();
    col += 1;
    while col > 0 {
        let rem = ((col - 1) % 26) as u8;
        result.insert(0, (b'A' + rem) as char);
        col = (col - 1) / 26;
    }
    result
}

/// Converts a `CellIdentifier` to a string key (e.g., "A1", "B2")
fn cell_id_to_string(id: &CellIdentifier) -> String {
    let col_str = col_number_to_letters(id.col);
    let row_str = (id.row + 1).to_string();
    format!("{}{}", col_str, row_str)
}

/// Parses a string like "A1" into a `CellIdentifier`
fn parse_cell_id(s: &str) -> Option<CellIdentifier> {
    let (col_str, row_str) = s
        .trim()
        .chars()
        .partition::<String, _>(|c| c.is_ascii_alphabetic());
    let col = col_str
        .chars()
        .fold(0u32, |acc, c| {
            acc * 26 + (c.to_ascii_uppercase() as u32 - 'A' as u32 + 1)
        })
        .checked_sub(1)?;
    let row = row_str.parse::<u32>().ok()?.checked_sub(1)?;
    Some(CellIdentifier { col, row })
}

/// Extracts variable data (scalar, vector, or matrix) from the sheet based on the variable name
fn extract_variable(sheet: &HashMap<String, CellValue>, name: &str) -> Option<CellArgument> {
    if name.contains('_') {
        let parts: Vec<_> = name.split('_').collect();
        if parts.len() != 2 {
            return None;
        }

        let start_id = parse_cell_id(parts[0])?;
        let end_id = parse_cell_id(parts[1])?;

        if start_id.row == end_id.row {
            // Extract horizontal vector
            let mut values = vec![];
            for col in start_id.col..=end_id.col {
                let key = cell_id_to_string(&CellIdentifier {
                    col,
                    row: start_id.row,
                });
                values.push(sheet.get(&key).cloned().unwrap_or(CellValue::None));
            }
            Some(CellArgument::Vector(values))
        } else if start_id.col == end_id.col {
            // Extract vertical vector
            let mut values = vec![];
            for row in start_id.row..=end_id.row {
                let key = cell_id_to_string(&CellIdentifier {
                    col: start_id.col,
                    row,
                });
                values.push(sheet.get(&key).cloned().unwrap_or(CellValue::None));
            }
            Some(CellArgument::Vector(values))
        } else {
            // Extract 2D matrix
            let mut matrix = vec![];
            for row in start_id.row..=end_id.row {
                let mut row_vec = vec![];
                for col in start_id.col..=end_id.col {
                    let key = cell_id_to_string(&CellIdentifier { col, row });
                    row_vec.push(sheet.get(&key).cloned().unwrap_or(CellValue::None));
                }
                matrix.push(row_vec);
            }
            Some(CellArgument::Matrix(matrix))
        }
    } else {
        // Single value
        Some(CellArgument::Value(
            sheet.get(name).cloned().unwrap_or(CellValue::None),
        ))
    }
}

/// Starts the rsheet server with asynchronous dependency tracking
pub fn start_server<M>(mut manager: M) -> Result<(), Box<dyn Error>>
where
    M: Manager + Send + 'static,
{
    let sheet = Arc::new(Mutex::new(HashMap::new()));
    let exprs = Arc::new(Mutex::new(HashMap::new()));
    let status = Arc::new(Mutex::new(HashMap::new()));
    let deps = Arc::new(Mutex::new(HashMap::<String, Vec<String>>::new()));
    let (tx, rx) = channel::<String>();

    // Background worker thread: prioritizes non-blocking expressions
    {
        let sheet = Arc::clone(&sheet);
        let exprs = Arc::clone(&exprs);
        let status = Arc::clone(&status);
        let deps = Arc::clone(&deps);
        let tx = tx.clone();

        thread::spawn(move || {
            let mut normal_queue: VecDeque<String> = VecDeque::new();
            let mut delayed_queue: VecDeque<String> = VecDeque::new();

            loop {
                if let Some(key) = normal_queue.pop_front() {
                    evaluate_and_store(&key, &sheet, &exprs, &status, &deps, &tx);
                    continue;
                }

                if let Some(key) = delayed_queue.pop_front() {
                    evaluate_and_store(&key, &sheet, &exprs, &status, &deps, &tx);
                    continue;
                }

                match rx.recv() {
                    Ok(key) => {
                        let expr_str: String = {
                            let exprs_guard = exprs.lock().unwrap();
                            exprs_guard.get(key.as_str()).cloned().unwrap_or_default()
                        };
                        if expr_str.contains("sleep_then") {
                            delayed_queue.push_back(key);
                        } else {
                            normal_queue.push_back(key);
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }

    let mut handles = Vec::new();

    // Accept and process client connections
    while let Connection::NewConnection { reader, writer } = manager.accept_new_connection() {
        let sheet_clone = Arc::clone(&sheet);
        let exprs_clone = Arc::clone(&exprs);
        let status_clone = Arc::clone(&status);
        let deps_clone = Arc::clone(&deps);
        let tx_clone = tx.clone();

        let handle = thread::spawn(move || {
            if let Err(e) = handle_connection(
                reader,
                writer,
                sheet_clone,
                exprs_clone,
                status_clone,
                deps_clone,
                tx_clone,
            ) {
                eprintln!("Connection error: {}", e);
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.join();
    }

    Ok(())
}

/// Handles incoming client messages (set/get commands)
fn handle_connection<R, W>(
    mut recv: R,
    mut send: W,
    sheet: Arc<Mutex<HashMap<String, CellValue>>>,
    exprs: Arc<Mutex<HashMap<String, String>>>,
    status: Arc<Mutex<HashMap<String, bool>>>,
    deps: Arc<Mutex<HashMap<String, Vec<String>>>>,
    tx: Sender<String>,
) -> Result<(), Box<dyn Error>>
where
    R: Reader + Send + 'static,
    W: Writer + Send + 'static,
{
    loop {
        match recv.read_message() {
            rsheet_lib::connect::ReadMessageResult::Message(msg) => {
                let reply = match msg.parse::<Command>() {
                    Ok(Command::Get { cell_identifier }) => {
                        let key = cell_id_to_string(&cell_identifier);
                        // Wait until value is ready (status == true)
                        loop {
                            let ready = {
                                let status_guard = status.lock().unwrap();
                                *status_guard.get(&key).unwrap_or(&true)
                            };
                            if ready {
                                break;
                            }
                            thread::sleep(Duration::from_millis(5));
                        }
                        let value = {
                            let sheet_guard = sheet.lock().unwrap();
                            sheet_guard.get(&key).cloned().unwrap_or(CellValue::None)
                        };
                        match value {
                            CellValue::Error(msg) if msg == "Dependency error" => {
                                Reply::Error("Dependency error".to_string())
                            }
                            _ => Reply::Value(key, value),
                        }
                    }
                    Ok(Command::Set {
                        cell_identifier,
                        cell_expr,
                    }) => {
                        let key = cell_id_to_string(&cell_identifier);
                        {
                            let mut exprs_guard = exprs.lock().unwrap();
                            exprs_guard.insert(key.clone(), cell_expr.clone());
                        }
                        {
                            let mut status_guard = status.lock().unwrap();
                            status_guard.insert(key.clone(), false);
                        }
                        {
                            // Extract and expand all dependencies
                            let mut deps_guard = deps.lock().unwrap();
                            let mut expanded_vars = Vec::new();
                            for var in CellExpr::new(&cell_expr).find_variable_names() {
                                if var.contains('_') {
                                    if let Some((start, end)) = var.split_once('_') {
                                        if let (Some(start_id), Some(end_id)) =
                                            (parse_cell_id(start), parse_cell_id(end))
                                        {
                                            if start_id.row == end_id.row {
                                                for col in start_id.col..=end_id.col {
                                                    expanded_vars.push(cell_id_to_string(
                                                        &CellIdentifier {
                                                            col,
                                                            row: start_id.row,
                                                        },
                                                    ));
                                                }
                                            } else if start_id.col == end_id.col {
                                                for row in start_id.row..=end_id.row {
                                                    expanded_vars.push(cell_id_to_string(
                                                        &CellIdentifier {
                                                            col: start_id.col,
                                                            row,
                                                        },
                                                    ));
                                                }
                                            } else {
                                                for row in start_id.row..=end_id.row {
                                                    for col in start_id.col..=end_id.col {
                                                        expanded_vars.push(cell_id_to_string(
                                                            &CellIdentifier { col, row },
                                                        ));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    expanded_vars.push(var);
                                }
                            }
                            deps_guard.insert(key.clone(), expanded_vars);
                        }
                        let _ = tx.send(key);
                        continue;
                    }
                    Err(e) => Reply::Error(e),
                };

                match send.write_message(reply) {
                    rsheet_lib::connect::WriteMessageResult::Ok => {}
                    rsheet_lib::connect::WriteMessageResult::ConnectionClosed => break,
                    rsheet_lib::connect::WriteMessageResult::Err(e) => return Err(Box::new(e)),
                }
            }
            rsheet_lib::connect::ReadMessageResult::ConnectionClosed => break,
            rsheet_lib::connect::ReadMessageResult::Err(e) => return Err(Box::new(e)),
        }
    }
    Ok(())
}

/// Evaluates a cell and stores the result, then re-triggers dependent evaluations
fn evaluate_and_store(
    key: &str,
    sheet: &Arc<Mutex<HashMap<String, CellValue>>>,
    exprs: &Arc<Mutex<HashMap<String, String>>>,
    status: &Arc<Mutex<HashMap<String, bool>>>,
    deps: &Arc<Mutex<HashMap<String, Vec<String>>>>,
    tx: &Sender<String>,
) {
    let expr_str = {
        let exprs_guard = exprs.lock().unwrap();
        exprs_guard.get(key).cloned().unwrap_or_default()
    };
    let expr = CellExpr::new(&expr_str);
    let mut context = HashMap::new();
    {
        let sheet_guard = sheet.lock().unwrap();
        for var in expr.find_variable_names() {
            if let Some(arg) = extract_variable(&sheet_guard, &var) {
                context.insert(var, arg);
            }
        }
    }
    let value = match expr.evaluate(&context) {
        Ok(v) => v,
        Err(_) => CellValue::Error("Dependency error".to_string()),
    };
    {
        let mut sheet_guard = sheet.lock().unwrap();
        sheet_guard.insert(key.to_string(), value);
    }
    {
        let mut status_guard = status.lock().unwrap();
        status_guard.insert(key.to_string(), true);
    }

    // Reverse dependency map: notify all dependent cells
    let reverse_map = {
        let deps_guard = deps.lock().unwrap();
        let mut map = HashMap::<String, Vec<String>>::new();
        for (cell, vars) in deps_guard.iter() {
            for var in vars {
                map.entry(var.clone()).or_default().push(cell.clone());
            }
        }
        map
    };
    if let Some(dependents) = reverse_map.get(key) {
        for dep in dependents {
            let _ = tx.send(dep.clone());
        }
    }
}
